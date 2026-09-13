// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pointer candidates follow the same bounded stacking phases and inverse
//! 2D geometry as paint, including flattened descendants of scroll containers.

use super::*;
use crate::{paint::stacking_level, placement::PlacementState};

/// Topmost pointer-enabled element at a document-space CSS pixel position.
pub fn hit_test<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    x: f32,
    y: f32,
) -> Option<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    hit_test_with_scroll(dom, styles, fragments, &HashMap::new(), x, y)
}

/// Numeric z-index starts a context only on positioned boxes or flex/grid items.
pub(crate) fn z_index_stacking_level<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    id: D::NodeId,
) -> Option<i32>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let style = styles.get(id)?;
    let livery::values::ZIndex::Integer(level) = style.z_index else {
        return None;
    };
    let item = dom
        .parent(id)
        .and_then(|parent| styles.get(parent))
        .is_some_and(|parent| matches!(parent.display, CssDisplay::Flex | CssDisplay::Grid));
    (style.position != CssPosition::Static || item).then_some(level)
}

/// Stable CSS order-modified children shared with paint.
pub(crate) fn order_modified_children<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    parent: D::NodeId,
) -> Vec<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let mut children = dom.flat_children(parent).collect::<Vec<_>>();
    if styles
        .get(parent)
        .is_some_and(|style| matches!(style.display, CssDisplay::Flex | CssDisplay::Grid))
    {
        children.sort_by_key(|child| styles.get(*child).map_or(0, |style| style.order.value()));
    }
    children
}

#[cfg(test)]
pub(in crate::layout) fn stacking_paint_children<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    parent: D::NodeId,
) -> Vec<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let mut children = order_modified_children(dom, styles, parent);
    children.sort_by_key(|child| stacking_level(dom, styles, *child).unwrap_or_default());
    children
}

/// Hit-test a retained fragment plane with per-element content scroll offsets.
/// The caller adds document scroll to viewport pointer coordinates. CSS
/// transforms are inverted exactly; a rotated AABB never admits a hit by itself.
pub fn hit_test_with_scroll<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
    x: f32,
    y: f32,
) -> Option<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let mut state = HitTestState {
        dom,
        styles,
        fragments,
        scroll_offsets,
        x,
        y,
        winner: None,
    };
    state.visit(dom.document(), &PlacementState::default(), None);
    state.winner
}

struct StackingEntry<Id> {
    id: Id,
    level: i32,
    placement: PlacementState,
}

struct HitTestState<'a, D: LayoutDom>
where
    D::NodeId: Copy + Eq + Hash,
{
    dom: &'a D,
    styles: &'a StylePlane<D::NodeId>,
    fragments: &'a LiveryLayout<D::NodeId>,
    scroll_offsets: &'a HashMap<D::NodeId, (f32, f32)>,
    x: f32,
    y: f32,
    winner: Option<D::NodeId>,
}

impl<D: LayoutDom> HitTestState<'_, D>
where
    D::NodeId: Copy + Eq + Hash,
{
    fn visit(
        &mut self,
        id: D::NodeId,
        placement: &PlacementState,
        roots: Option<&HashSet<D::NodeId>>,
    ) {
        if roots.is_some_and(|roots| roots.contains(&id)) {
            return;
        }
        let Some(entered) = placement.enter(
            self.dom,
            self.styles,
            self.fragments,
            self.scroll_offsets,
            id,
        ) else {
            return;
        };
        if self
            .styles
            .get(id)
            .is_some_and(|style| style.pointer_events == livery::values::PointerEvents::Auto)
            && entered
                .geometry
                .is_some_and(|geometry| geometry.hits_border(self.x, self.y))
        {
            self.winner = Some(id);
        }
        if let Some(roots) = roots {
            for child in order_modified_children(self.dom, self.styles, id) {
                self.visit(child, &entered.children, Some(roots));
            }
            return;
        }

        // Paint flattens stacking roots out of non-context ancestors. Keep
        // their inherited placement, but visit them in the owning context's
        // negative / normal / nonnegative phases, not at their DOM depth.
        let mut items = Vec::new();
        self.collect_stacking(id, &entered.children, &mut items);
        items.sort_by_key(|item| item.level);
        let roots = items.iter().map(|item| item.id).collect::<HashSet<_>>();
        for item in items.iter().filter(|item| item.level < 0) {
            self.visit(item.id, &item.placement, None);
        }
        for child in order_modified_children(self.dom, self.styles, id) {
            self.visit(child, &entered.children, Some(&roots));
        }
        for item in items.iter().filter(|item| item.level >= 0) {
            self.visit(item.id, &item.placement, None);
        }
    }

    fn collect_stacking(
        &self,
        parent: D::NodeId,
        placement: &PlacementState,
        items: &mut Vec<StackingEntry<D::NodeId>>,
    ) {
        for child in order_modified_children(self.dom, self.styles, parent) {
            if let Some(level) = stacking_level(self.dom, self.styles, child) {
                items.push(StackingEntry {
                    id: child,
                    level,
                    placement: placement.clone(),
                });
            } else if let Some(entered) = placement.enter(
                self.dom,
                self.styles,
                self.fragments,
                self.scroll_offsets,
                child,
            ) {
                self.collect_stacking(child, &entered.children, items);
            }
        }
    }
}
