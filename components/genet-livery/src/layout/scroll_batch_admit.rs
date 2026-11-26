// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The structural admission of a scroll batch: which parent, which entering
//! and leaving roots, and what may sit inside an entering root.

use super::scroll_batch::ScrollBatch;
use super::*;

/// Entering and leaving roots resolved to the boxes and fragments they own.
pub(in crate::layout) struct AdmittedRoots {
    /// Principal boxes of the entering roots, in batch order.
    pub(in crate::layout) inserted: Vec<BoxId>,
    /// Retained fragments of the leaving roots.
    pub(in crate::layout) removed: Vec<FragmentId>,
}

impl<Id: Copy + Eq + Hash> LiveryLayout<Id> {
    /// Conditions 2 and 4 of the scroll batch, against the reconciled box
    /// tree `boxes` for the final DOM. `self` is still the previous layout.
    pub(in crate::layout) fn admit_structure<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        styles: &StylePlane<Id>,
        boxes: &GeneratedBoxTree<Id>,
        batch: &ScrollBatch<Id>,
        parent: Id,
    ) -> Option<AdmittedRoots> {
        let previous = self.buckram.boxes();
        let parent_box = previous.principal_box(parent)?;
        if boxes.principal_box(parent) != Some(parent_box)
            || !admits_parent(&previous[parent_box], styles.get(parent)?)
            || self
                .buckram
                .fragments()
                .fragment_ids_for_box(parent_box)
                .len()
                != 1
            || !boxes[parent_box]
                .children()
                .iter()
                .all(|child| boxes[*child].positioning == PositioningScheme::Absolute)
        {
            return None;
        }
        let mut removed = Vec::new();
        let mut removed_boxes = 0;
        for root in &batch.removed {
            let root_box = previous.principal_box(*root)?;
            if previous[root_box].parent() != Some(parent_box)
                || previous[root_box].positioning != PositioningScheme::Absolute
            {
                return None;
            }
            let [fragment] = *self.buckram.fragments().fragment_ids_for_box(root_box) else {
                return None;
            };
            removed.push(fragment);
            removed_boxes += box_subtree(previous, root_box).len();
        }
        let mut inserted = Vec::new();
        let mut inserted_boxes = HashSet::new();
        for root in &batch.inserted {
            let root_box = boxes.principal_box(*root)?;
            let subtree = box_subtree(boxes, root_box);
            if boxes[root_box].parent() != Some(parent_box)
                || boxes[root_box].positioning != PositioningScheme::Absolute
                || boxes[root_box].flow != FlowAxes::HORIZONTAL_LTR
                || !definite_px_size(styles.get(*root)?)
                || !subtree
                    .iter()
                    .all(|box_id| admits_inserted_box(dom, styles, &boxes[*box_id]))
            {
                return None;
            }
            inserted.push(root_box);
            inserted_boxes.extend(subtree);
        }
        // The reconciled tree must be the previous one less the leaving
        // subtrees plus the entering ones: every other box kept its identity.
        let newest = previous.iter().map(|(id, _)| id.index()).max()?;
        let fresh_only_where_entered = boxes
            .iter()
            .all(|(id, _)| (id.index() > newest) == inserted_boxes.contains(&id));
        (fresh_only_where_entered
            && boxes.len() + removed_boxes == previous.len() + inserted_boxes.len())
        .then_some(AdmittedRoots { inserted, removed })
    }
}

/// The parent keeps its size whatever its absolute children do, and its
/// content box is exact: a block container in horizontal flow with no
/// percentage padding.
fn admits_parent<Id>(css_box: &buckram::CssBox<Id>, style: &ComputedValues) -> bool {
    css_box.display.outside == Some(DisplayOutside::Block)
        && matches!(
            css_box.display.inside,
            Some(DisplayInside::Flow | DisplayInside::FlowRoot)
        )
        && css_box.display.internal_table.is_none()
        && matches!(
            css_box.formatting_context,
            Some(FormattingContextKind::Block | FormattingContextKind::Inline)
        )
        && css_box.flow == FlowAxes::HORIZONTAL_LTR
        && [
            style.padding_top.0,
            style.padding_right.0,
            style.padding_bottom.0,
            style.padding_left.0,
        ]
        .iter()
        .all(|padding| !padding.has_percentage())
}

/// Condition 4: what the local formatter reproduces exactly. Fixed, relative
/// and sticky boxes, floats, table parts, atomic inlines, replaced elements,
/// form controls, generated content, and positioned boxes sized by content
/// stay with complete layout.
fn admits_inserted_box<D: LayoutDom>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    css_box: &buckram::CssBox<D::NodeId>,
) -> bool
where
    D::NodeId: Copy + Eq + Hash,
{
    let node = match css_box.origin {
        BoxOrigin::Element(node) => Some(node),
        BoxOrigin::Text(_) | BoxOrigin::Anonymous { .. } => None,
        BoxOrigin::Pseudo { .. } => return false,
    };
    let positioned_sized = css_box.positioning != PositioningScheme::Absolute
        || node.and_then(|node| styles.get(node)).is_some_and(|style| {
            matches!(style.width, CssSize::Value(_)) && matches!(style.height, CssSize::Value(_))
        });
    matches!(
        css_box.positioning,
        PositioningScheme::Static | PositioningScheme::Absolute
    ) && positioned_sized
        && css_box.float == FloatSide::None
        && css_box.display.internal_table.is_none()
        && !css_box.replaced
        && !generated_image::is_generated_image(css_box)
        && (css_box.display.outside != Some(DisplayOutside::Inline)
            || css_box.display.inside == Some(DisplayInside::Flow))
        && node.is_none_or(|node| {
            dom.form_control_state(node).is_none()
                && dom.shadow_root(node).is_none()
                && dom.assigned_slot(node).is_none()
        })
}

/// A definite pixel width and height: the box's border box is known before
/// its containing block is.
pub(in crate::layout) fn definite_px_size(style: &ComputedValues) -> bool {
    [style.width, style.height]
        .iter()
        .all(|size| matches!(size, CssSize::Value(value) if !value.has_percentage()))
}

/// `root` and every generated descendant.
pub(in crate::layout) fn box_subtree<Id>(
    boxes: &buckram::CssBoxTree<Id>,
    root: BoxId,
) -> Vec<BoxId> {
    let mut collected = vec![root];
    let mut index = 0;
    while let Some(box_id) = collected.get(index).copied() {
        collected.extend_from_slice(boxes[box_id].children());
        index += 1;
    }
    collected
}
