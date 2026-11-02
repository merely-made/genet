// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

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
    pending: Vec<(AlgorithmNodeId, PreparedMeasure, ComputedValues)>,
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
    pub fn push(
        &mut self,
        node: AlgorithmNodeId,
        prepared: PreparedMeasure,
        style: ComputedValues,
    ) {
        self.pending.push((node, prepared, style));
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
        for (_, measure, style) in &self.pending {
            if measure.template.ready::<D::NodeId, _>(atomic) {
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
        let results = text
            .shape_batch(&jobs, self.generation)
            .map_err(|e| LayoutError::retained_state(format!("inline batch rejected: {e:?}")))?;
        let mut by_group: HashMap<_, Vec<_>> = HashMap::new();
        for result in results {
            by_group.entry(result.group).or_default().push(result);
        }
        let mut updates = HashMap::new();
        for (node, mut measure, _) in self.pending.drain(..) {
            measure.results = by_group.remove(&measure.group).unwrap_or_default();
            updates.insert(node, measure);
        }
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
