// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[cfg(test)]
thread_local! {
    pub(crate) static FIXTURE_FAULT: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    pub(crate) static FIXTURE_UPDATES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
thread_local! { pub(crate) static INPUT_OWNERSHIP: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) }; }

#[cfg(test)]
thread_local! { pub(crate) static FIXTURE_GATE: std::cell::Cell<(usize, usize, usize)> = const { std::cell::Cell::new((0, 0, 0)) }; }

pub(super) struct PreparedMeasure {
    pub template: Arc<crate::text::prepared::PreparedInlineGroup>,
    pub generation: u64,
    pub group: crate::text::InlineGroupId,
    pub results: Vec<crate::text::ShapeResult>,
}

pub(super) struct TextBatch {
    generation: u64,
    record_timings: bool,
    collect_ns: u64,
    pending: Vec<(AlgorithmNodeId, PreparedMeasure)>,
}
impl TextBatch {
    pub fn new(record_timings: bool) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            generation: NEXT.fetch_add(1, Ordering::Relaxed),
            record_timings,
            collect_ns: 0,
            pending: Vec::new(),
        }
    }
    pub fn collect<D: LayoutDom>(
        &mut self,
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        boxes: &GeneratedBoxTree<D::NodeId>,
        roots: &[BoxId],
        style: &ComputedValues,
    ) -> PreparedMeasure
    where
        D::NodeId: Copy + Eq + Hash,
    {
        let start = self.record_timings.then(std::time::Instant::now);
        let template = Arc::new(crate::text::prepared::PreparedInlineGroup::collect(
            dom, styles, boxes, roots, style,
        ));
        self.collect_ns = self.collect_ns.saturating_add(start.map_or(0, |start| {
            u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
        }));
        PreparedMeasure {
            template,
            generation: self.generation,
            group: crate::text::InlineGroupId(self.pending.len()),
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, node: AlgorithmNodeId, prepared: PreparedMeasure) {
        self.pending.push((node, prepared));
    }
    pub fn flush<D: LayoutDom>(
        &mut self,
        text: &mut TextSystem,
        atomic: &AtomicLayoutPlane,
        tree: &mut AlgorithmTree<InlineMeasure, Vec<BoxId>>,
    ) -> Result<(), LayoutError>
    where
        D::NodeId: Copy + Eq + Hash,
    {
        text.account_collection_ns(std::mem::take(&mut self.collect_ns));
        let mut jobs = Vec::new();
        for (node, measure) in &self.pending {
            let style = &tree
                .context(*node)
                .ok_or_else(|| {
                    LayoutError::retained_state(format!(
                        "inline batch missing measured context for {node:?}"
                    ))
                })?
                .style;
            #[cfg(test)]
            if measure.template.ready::<D::NodeId, _>(atomic) {
                let context_style = &tree.context(*node).expect("measured context").style;
                INPUT_OWNERSHIP.with(|c| {
                    let (total, shared) = c.get();
                    c.set((
                        total + 1,
                        shared + usize::from(std::ptr::eq(style, context_style)),
                    ));
                });
            }
            #[cfg(test)]
            FIXTURE_GATE.with(|c| {
                let (ready, empty, eligible) = c.get();
                let r = measure.template.ready::<D::NodeId, _>(atomic);
                let e = r && measure.template.test_empty_events();
                let q = r && measure.template.can_share_eager_key(style);
                assert_eq!(
                    q,
                    e && crate::text::test_equivalent_overflow(&measure.template, style)
                );
                c.set((
                    ready + usize::from(r),
                    empty + usize::from(e),
                    eligible + usize::from(q),
                ));
            });
            if measure.template.ready::<D::NodeId, _>(atomic) {
                if measure.template.can_share_eager_key(style) {
                    jobs.push(crate::text::ShapeJob {
                        group: measure.group,
                        generation: self.generation,
                        key: text.prepared_min_shape_key_for_ready::<D::NodeId, _>(
                            &measure.template,
                            atomic,
                            style,
                        ),
                    });
                } else {
                    for key in text.prepared_shape_keys_for_ready::<D::NodeId, _>(
                        &measure.template,
                        atomic,
                        style,
                    ) {
                        jobs.push(crate::text::ShapeJob {
                            group: measure.group,
                            generation: self.generation,
                            key,
                        });
                    }
                }
            }
        }
        #[cfg(test)]
        FIXTURE_FAULT.with(|fault| {
            if let Some(index) = fault.replace(None) {
                assert_eq!(
                    jobs.iter().map(|job| job.group.0).collect::<Vec<_>>(),
                    [0, 1, 1, 1, 2]
                );
                jobs[index].generation = self.generation.wrapping_add(1);
            }
        });
        let results = text
            .shape_batch(&jobs, self.generation)
            .map_err(|e| LayoutError::retained_state(format!("inline batch rejected: {e:?}")))?;
        let mut by_group: HashMap<_, Vec<_>> = HashMap::new();
        for result in results {
            by_group.entry(result.group).or_default().push(result);
        }
        let mut updates = HashMap::new();
        for (node, mut measure) in self.pending.drain(..) {
            measure.results = by_group.remove(&measure.group).unwrap_or_default();
            updates.insert(node, measure);
        }
        #[cfg(test)]
        FIXTURE_UPDATES.with(|count| count.set(count.get() + 1));
        tree.update_contexts(|node, context| {
            if let Some(measure) = updates.remove(&node) {
                context.prepared = Some(measure);
            }
        });
        Ok(())
    }
}
pub(super) fn check_errors(
    tree: &AlgorithmTree<InlineMeasure, Vec<BoxId>>,
) -> Result<(), LayoutError> {
    for node in tree.node_ids() {
        if let Some(error) = tree
            .context(node)
            .and_then(|context| context.shape_error.as_ref())
        {
            return Err(error.clone());
        }
    }
    Ok(())
}
