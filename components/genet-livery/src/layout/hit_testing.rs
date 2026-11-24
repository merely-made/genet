// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pointer candidates follow the same bounded stacking phases and inverse
//! 2D geometry as paint, including flattened descendants of scroll containers.

use super::*;
use crate::{
    paint::{StackingOrder, stacking_order},
    placement::PlacementState,
};

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
pub(crate) fn z_index_stacking_level<Id>(
    styles: &StylePlane<Id>,
    id: Id,
    flex_grid_item: bool,
) -> Option<i32>
where
    Id: Copy + Eq + Hash,
{
    let style = styles.get(id)?;
    let livery::values::ZIndex::Integer(level) = style.z_index else {
        return None;
    };
    (style.position != CssPosition::Static || flex_grid_item).then_some(level)
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
    if styles.get(parent).is_some_and(|style| {
        matches!(
            style.display,
            CssDisplay::Flex | CssDisplay::Grid | CssDisplay::InlineGrid
        )
    }) {
        children.sort_by_key(|child| styles.get(*child).map_or(0, |style| style.order.value()));
    }
    children
}

#[cfg(test)]
pub(in crate::layout) fn stacking_paint_children<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    parent: D::NodeId,
) -> Vec<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let mut children = order_modified_children(dom, styles, parent);
    children.sort_by_key(|child| {
        stacking_order(
            dom,
            styles,
            *child,
            crate::paint::is_blockified_item(fragments, *child),
        )
        .map_or(0, StackingOrder::level)
    });
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
    let mut winner = None;
    visit_hits(dom, styles, fragments, scroll_offsets, x, y, &mut |id| {
        winner = Some(id)
    });
    winner
}

/// All pointer-enabled elements at a document-space point in front-to-back
/// paint order. Adjacent boxes for one element contribute one entry. A
/// different element can be painted between an element and its pseudo box.
pub fn hit_test_all_with_scroll<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
    x: f32,
    y: f32,
) -> Vec<D::NodeId>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let mut hits = Vec::new();
    visit_hits(dom, styles, fragments, scroll_offsets, x, y, &mut |id| {
        if hits.last() != Some(&id) {
            hits.push(id);
        }
    });
    hits.reverse();
    hits
}

fn visit_hits<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
    x: f32,
    y: f32,
    record: &mut dyn FnMut(D::NodeId),
) where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    if x.is_finite() && y.is_finite() {
        HitTestState {
            dom,
            styles,
            fragments,
            scroll_offsets,
            x,
            y,
            record,
        }
        .visit(dom.document(), &PlacementState::default(), None);
    }
}

struct StackingEntry<Id> {
    id: Id,
    order: StackingOrder,
    placement: PlacementState,
    pseudo: bool,
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
    record: &'a mut dyn FnMut(D::NodeId),
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
            (self.record)(id);
        }
        if let Some(roots) = roots {
            self.visit_in_flow_children(id, &entered.children, roots);
            return;
        }

        // Paint flattens stacking roots out of non-context ancestors. Keep
        // their inherited placement, but visit them in the owning context's
        // negative / normal / nonnegative phases, not at their DOM depth.
        let mut items = Vec::new();
        self.collect_stacking(id, &entered.children, &mut items);
        items.sort_by_key(|item| item.order.level());
        let roots = items
            .iter()
            .filter(|item| !item.pseudo)
            .map(|item| item.id)
            .collect::<HashSet<_>>();
        for item in items.iter().filter(|item| item.order.level() < 0) {
            self.visit_entry(item, None);
        }
        self.visit_in_flow_children(id, &entered.children, &roots);
        for item in items.iter().filter(|item| item.order.level() >= 0) {
            let excluded = (item.order == StackingOrder::Positioned).then_some(&roots);
            self.visit_entry(item, excluded);
        }
    }

    fn visit_entry(
        &mut self,
        entry: &StackingEntry<D::NodeId>,
        roots: Option<&HashSet<D::NodeId>>,
    ) {
        if entry.pseudo {
            (self.record)(entry.id);
        } else {
            self.visit(entry.id, &entry.placement, roots);
        }
    }

    fn visit_in_flow_children(
        &mut self,
        parent: D::NodeId,
        placement: &PlacementState,
        roots: &HashSet<D::NodeId>,
    ) {
        for child in order_modified_children(self.dom, self.styles, parent) {
            if !roots.contains(&child) {
                self.visit(child, placement, Some(roots));
            }
        }
    }

    fn collect_stacking(
        &self,
        parent: D::NodeId,
        placement: &PlacementState,
        items: &mut Vec<StackingEntry<D::NodeId>>,
    ) {
        self.collect_pseudo(parent, buckram::PseudoElement::Before, placement, items);
        for child in order_modified_children(self.dom, self.styles, parent) {
            if let Some(order) = stacking_order(
                self.dom,
                self.styles,
                child,
                crate::paint::is_blockified_item(self.fragments, child),
            ) {
                items.push(StackingEntry {
                    id: child,
                    pseudo: false,
                    order,
                    placement: placement.clone(),
                });
                if order != StackingOrder::Positioned {
                    continue;
                }
            }
            if let Some(entered) = placement.enter(
                self.dom,
                self.styles,
                self.fragments,
                self.scroll_offsets,
                child,
            ) {
                self.collect_stacking(child, &entered.children, items);
            }
        }
        self.collect_pseudo(parent, buckram::PseudoElement::After, placement, items);
    }

    fn collect_pseudo(
        &self,
        owner: D::NodeId,
        pseudo: buckram::PseudoElement,
        placement: &PlacementState,
        items: &mut Vec<StackingEntry<D::NodeId>>,
    ) {
        let Some((style, fragment)) = self.fragments.positioned_pseudo(self.styles, owner, pseudo)
        else {
            return;
        };
        if style.pointer_events == livery::values::PointerEvents::Auto
            && placement.hits_pseudo(style, fragment, self.x, self.y)
        {
            items.push(StackingEntry {
                id: owner,
                order: crate::paint::positioned_pseudo_order(style),
                placement: placement.clone(),
                pseudo: true,
            });
        }
    }
}
