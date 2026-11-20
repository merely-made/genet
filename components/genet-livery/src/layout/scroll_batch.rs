// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The retained route for a scroll batch: one absolute element translated,
//! absolutely positioned subtrees entering and leaving one parent, attribute
//! writes that leave layout alone. Everything else is refused, and a refusal
//! leaves the published layout untouched.

use layout_dom_api::DomMutation;

use super::query::Reposition;
use super::scroll_batch_admit::definite_px_size;
use super::scroll_batch_format::format_positioned_root;
use super::scroll_batch_shape::scroll_batch_shape;
use super::*;

/// Time one phase of a batch for the breakdown receipt; free outside tests.
macro_rules! phase {
    ($name:literal, $body:expr) => {{
        #[cfg(test)]
        let started = std::time::Instant::now();
        let value = $body;
        #[cfg(test)]
        record_phase($name, started.elapsed());
        value
    }};
}

/// The structural half of a scroll batch.
pub(in crate::layout) struct ScrollBatch<Id> {
    /// The element every entering and leaving root is a child of.
    pub(in crate::layout) parent: Option<Id>,
    /// Entering roots, in document order.
    pub(in crate::layout) inserted: Vec<Id>,
    pub(in crate::layout) removed: Vec<Id>,
    /// Every node under an entering root, roots included.
    pub(in crate::layout) entered: HashSet<Id>,
    /// Every node under a leaving root that the previous layout or the DOM
    /// still knows, roots included.
    pub(in crate::layout) left: HashSet<Id>,
}

/// An entering root formatted on its own, ready to insert.
struct EnteringRoot<Id> {
    node: Id,
    root_box: BoxId,
    local: FragmentTree,
    local_root: FragmentId,
    frame: TextFrame<Id>,
    static_position: StaticPosition,
}

/// Every change a batch makes, worked out before the first one is made.
struct BatchPlan<Id> {
    /// The final box tree, when structure changed.
    boxes: Option<GeneratedBoxTree<Id>>,
    removed: Vec<FragmentId>,
    reposition: Option<Reposition<Id>>,
    entering: Vec<EnteringRoot<Id>>,
}

impl<Id: Copy + Eq + Hash> LiveryLayout<Id> {
    /// Apply an admitted scroll batch in place. Every check that can refuse
    /// runs first; after the first change none can. On success
    /// `previous_styles` holds the resolved plane for `cascade`, patched only
    /// where `restyled` says the cascade changed.
    #[expect(
        clippy::too_many_arguments,
        reason = "borrowed layout and its caller-owned inputs"
    )]
    pub(in crate::layout) fn relayout_scroll_batch<D: LayoutDom<NodeId = Id>>(
        &mut self,
        dom: &D,
        cascade: &StylePlane<Id>,
        previous_styles: &mut StylePlane<Id>,
        restyled: Option<&HashSet<Id>>,
        viewport: ViewportSizes,
        mutations: &[DomMutation<Id>],
        text: &mut TextSystem,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<bool, LayoutError> {
        let Some(restyled) = restyled else {
            return Ok(false);
        };
        let Some(batch) = phase!("shape", scroll_batch_shape(dom, self, mutations)) else {
            return Ok(false);
        };
        // A leaving root was absolute with a definite size; only the plane
        // before the patch still says so.
        if !batch.removed.iter().all(|root| {
            previous_styles
                .get(*root)
                .is_some_and(|style| matches!(style.position, CssPosition::Absolute))
                && previous_styles.get(*root).is_some_and(definite_px_size)
        }) {
            return Ok(false);
        }
        let Some(mut patch) = phase!(
            "style patch",
            previous_styles.scroll_batch_patch(
                cascade,
                restyled,
                &batch.entered,
                &batch.left,
                dom,
                text,
                viewport,
            )
        ) else {
            return Ok(false);
        };
        // The layout checks read the final plane; swapping back undoes it.
        phase!("style swap", previous_styles.swap_patch(&mut patch));
        let plan =
            match self.plan_scroll_batch(dom, previous_styles, &batch, patch.moved, text, images) {
                Ok(Some(plan)) => plan,
                other => {
                    previous_styles.swap_patch(&mut patch);
                    return other.map(|_| false);
                },
            };
        self.apply_scroll_batch(dom, previous_styles, &batch, plan, images)?;
        #[cfg(test)]
        note_scroll_batch();
        Ok(true)
    }

    /// Every check of the batch, with no change made: `None` refuses.
    fn plan_scroll_batch<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        styles: &StylePlane<Id>,
        batch: &ScrollBatch<Id>,
        moved: Option<Id>,
        text: &mut TextSystem,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<Option<BatchPlan<Id>>, LayoutError> {
        let (width, height) = self.viewport;
        // Leaving rows are not the moved element's ancestors or containing
        // block, so removing them first cannot change this plan.
        let reposition = match moved {
            None => None,
            Some(node) => match phase!(
                "reposition plan",
                self.plan_reposition(dom, styles, images, node, width, height)
            ) {
                Some(plan) => plan,
                None => return Ok(None),
            },
        };
        let Some(parent) = batch.parent else {
            // No structure changed, so the box tree stands.
            return Ok(Some(BatchPlan {
                boxes: None,
                removed: Vec::new(),
                reposition,
                entering: Vec::new(),
            }));
        };
        let mut boxes = phase!("box tree", GeneratedBoxTree::from_dom(dom, styles));
        phase!(
            "box reconcile",
            boxes.reconcile_identifiers(self.buckram.boxes())
        );
        let Some(roots) = phase!(
            "admit structure",
            self.admit_structure(dom, styles, &boxes, batch, parent)
        ) else {
            return Ok(None);
        };
        if !phase!(
            "removal check",
            roots
                .removed
                .iter()
                .all(|root| self.buckram.fragments().can_remove_subtree(*root))
        ) {
            return Ok(None);
        }
        taffy_style::reset_calc_scratch();
        let mut entering = Vec::new();
        for (node, root_box) in batch.inserted.iter().zip(&roots.inserted) {
            let Some(root) =
                self.prepare_entering_root(dom, styles, &boxes, *node, *root_box, text, images)?
            else {
                return Ok(None);
            };
            entering.push(root);
        }
        Ok(Some(BatchPlan {
            boxes: Some(boxes),
            removed: roots.removed,
            reposition,
            entering,
        }))
    }

    /// Format one entering root against its parent's content box and check
    /// everything its insertion and placement need.
    #[expect(
        clippy::too_many_arguments,
        reason = "one formatting pass borrows the batch's layout inputs"
    )]
    fn prepare_entering_root<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        styles: &StylePlane<Id>,
        boxes: &GeneratedBoxTree<Id>,
        node: Id,
        root_box: BoxId,
        text: &mut TextSystem,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<Option<EnteringRoot<Id>>, LayoutError> {
        let fragments = self.buckram.fragments();
        let Some((parent_fragment, parent_style)) = self.entering_parent(styles, boxes, root_box)
        else {
            return Ok(None);
        };
        // The parent's size is final: only a size-stable move is admitted.
        let (_, _, content_width, content_height) = content_box_rect(
            parent_style,
            fragments
                .get(parent_fragment)
                .expect("a checked parent fragment"),
        );
        let Some((local, frame, local_root)) = phase!(
            "format root",
            format_positioned_root(
                dom,
                styles,
                boxes,
                root_box,
                (content_width, content_height),
                self.viewport,
                text,
                images,
            )?
        ) else {
            return Ok(None);
        };
        let Some(static_position) = local.static_position_for_box(root_box).copied() else {
            return Ok(None);
        };
        // Placement reads the containing block's fragment, an ancestor's.
        let containing_laid_out = match boxes[root_box].containing_block {
            ContainingBlock::Initial => true,
            ContainingBlock::Box(containing) => {
                !fragments.fragment_ids_for_box(containing).is_empty()
            },
        };
        if !containing_laid_out || !local.is_self_contained_subtree(local_root) {
            return Ok(None);
        }
        Ok(Some(EnteringRoot {
            node,
            root_box,
            local,
            local_root,
            frame,
            static_position,
        }))
    }

    /// The entering root's parent fragment, which must be unique, and style.
    fn entering_parent<'a>(
        &self,
        styles: &'a StylePlane<Id>,
        boxes: &GeneratedBoxTree<Id>,
        root_box: BoxId,
    ) -> Option<(FragmentId, &'a ComputedValues)> {
        let parent_box = boxes[root_box].parent()?;
        let [parent_fragment] = *self.buckram.fragments().fragment_ids_for_box(parent_box) else {
            return None;
        };
        Some((parent_fragment, styles.get(boxes.origin_node(parent_box)?)?))
    }

    /// Make the planned changes. Each was checked by `plan_scroll_batch`, so
    /// a failure here is a broken invariant, reported as an error.
    fn apply_scroll_batch<D: LayoutDom<NodeId = Id>>(
        &mut self,
        dom: &D,
        styles: &StylePlane<Id>,
        batch: &ScrollBatch<Id>,
        plan: BatchPlan<Id>,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<(), LayoutError> {
        for root in plan.removed {
            if !phase!(
                "remove_subtree",
                self.buckram.fragments_mut().remove_subtree(root)
            ) {
                return Err(checked_step_failed("removing a leaving root"));
            }
        }
        if let Some(reposition) = &plan.reposition {
            phase!("reposition", self.apply_reposition(dom, reposition));
        }
        if let Some(boxes) = plan.boxes {
            let mut frame = self.text_frame.take().unwrap_or_default();
            let mut replaced = batch.left.clone();
            if plan.entering.is_empty() {
                phase!(
                    "text splice",
                    frame.splice_subtree_from(None, &replaced, dom)
                );
            }
            for root in plan.entering {
                let node = root.node;
                let local = self.insert_entering_root(dom, styles, &boxes, root, images)?;
                collect_subtree(dom, node, &mut replaced);
                phase!(
                    "text splice",
                    frame.splice_subtree_from(Some((&local, node)), &replaced, dom)
                );
                replaced.clear();
            }
            self.text_frame = Some(frame);
            self.buckram.replace_box_tree(boxes.into_tree());
        }
        phase!(
            "flush overflow",
            self.buckram.fragments_mut().flush_overflow()
        );
        Ok(())
    }

    /// Insert one prepared root under its parent's fragment and place it as
    /// complete layout's positioned pass would. Returns its text, already in
    /// document coordinates.
    fn insert_entering_root<D: LayoutDom<NodeId = Id>>(
        &mut self,
        dom: &D,
        styles: &StylePlane<Id>,
        boxes: &GeneratedBoxTree<Id>,
        root: EnteringRoot<Id>,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<TextFrame<Id>, LayoutError> {
        let EnteringRoot {
            node,
            root_box,
            mut local,
            local_root,
            mut frame,
            mut static_position,
        } = root;
        let (width, height) = self.viewport;
        let (parent_fragment, parent_style) = self
            .entering_parent(styles, boxes, root_box)
            .ok_or_else(|| checked_step_failed("finding the parent fragment"))?;
        let parent_rect = self
            .buckram
            .fragments()
            .get(parent_fragment)
            .expect("a found parent fragment");
        // The parent may have moved since the root was formatted.
        let (content_x, content_y, _, _) = content_box_rect(parent_style, parent_rect);
        let border_box = parent_rect.physical_rect();
        local.translate_subtree(
            local_root,
            PhysicalOffset {
                x: content_x,
                y: content_y,
            },
        );
        let parent_box = boxes[root_box].parent().expect("a found parent box");
        let after = boxes[parent_box]
            .children()
            .iter()
            .take_while(|sibling| **sibling != root_box)
            .filter_map(|sibling| {
                self.buckram
                    .fragments()
                    .fragment_ids_for_box(*sibling)
                    .iter()
                    .copied()
                    .find(|id| {
                        self.buckram
                            .fragments()
                            .get(*id)
                            .and_then(TreeFragment::parent)
                            == Some(parent_fragment)
                    })
            })
            .last();
        let fragments = self.buckram.fragments_mut();
        let inserted = phase!(
            "insert_subtree",
            fragments.insert_subtree(
                Some(parent_fragment),
                after,
                Some(parent_fragment),
                &local,
                local_root,
            )
        )
        .ok_or_else(|| checked_step_failed("inserting an entering root"))?;
        // Complete layout records the static position against the parent's
        // border box; the local one was taken against its content box.
        static_position.source = StaticPositionSource::Fragment(parent_fragment);
        static_position.logical_rect.inline_start += content_x - border_box.x;
        static_position.logical_rect.block_start += content_y - border_box.y;
        fragments.reconcile_static_position(static_position);
        let placement = phase!(
            "placement",
            positioned_placement_for_box(
                self.buckram.fragments(),
                boxes,
                styles,
                dom,
                images,
                width,
                height,
                root_box,
            )
        )
        .ok_or_else(|| checked_step_failed("placing an entering root"))?;
        let target = placement.target_rect();
        let offset = PhysicalOffset {
            x: placement.containing_rect.x + target.x - placement.current.x,
            y: placement.containing_rect.y + target.y - placement.current.y,
        };
        let fragments = self.buckram.fragments_mut();
        fragments.resize_leaf(
            inserted,
            PhysicalSize {
                width: target.width,
                height: target.height,
            },
        );
        fragments.translate_subtree(inserted, offset);
        fragments.set_containing_fragment(inserted, placement.containing_fragment);
        phase!(
            "text translate",
            frame.translate_subtree(dom, node, (content_x + offset.x, content_y + offset.y))
        );
        Ok(frame)
    }
}

/// A step its checks made infallible failed anyway: the layout may already
/// be partly changed, so this is an error rather than a refusal.
fn checked_step_failed(step: &str) -> LayoutError {
    debug_assert!(false, "{step} was checked before the first change");
    LayoutError::retained_state(format!("scroll batch: {step} failed after its check"))
}

fn collect_subtree<D: LayoutDom>(dom: &D, root: D::NodeId, nodes: &mut HashSet<D::NodeId>)
where
    D::NodeId: Copy + Eq + Hash,
{
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        nodes.insert(node);
        pending.extend(dom.dom_children(node));
    }
}

#[cfg(test)]
thread_local! {
    static SCROLL_BATCHES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Scroll batches this thread has published, for receipts that must prove
/// the route ran rather than fell back.
#[cfg(test)]
pub(crate) fn scroll_batches() -> usize {
    SCROLL_BATCHES.with(std::cell::Cell::get)
}

#[cfg(test)]
thread_local! {
    static PHASES: std::cell::RefCell<Vec<(&'static str, std::time::Duration)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn record_phase(name: &'static str, elapsed: std::time::Duration) {
    PHASES.with(|phases| phases.borrow_mut().push((name, elapsed)));
}

/// The phases timed since the last call, in the order they ran.
#[cfg(test)]
pub(crate) fn take_phases() -> Vec<(&'static str, std::time::Duration)> {
    PHASES.with(|phases| std::mem::take(&mut *phases.borrow_mut()))
}

#[cfg(test)]
fn note_scroll_batch() {
    SCROLL_BATCHES.with(|count| count.set(count.get() + 1));
}
