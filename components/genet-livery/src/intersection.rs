// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Intersection Observer geometry: how much of a target shows through the
//! clips between it and a root. A pure function of the retained fragments, the
//! style plane and the scroll offsets, so it is testable without script.
//!
//! Everything is in client (viewport) coordinates. Transforms are honoured for
//! the target's own box and ignored on the containers that clip it.

use std::{collections::HashMap, hash::Hash};

use layout_dom_api::{LayoutDom, NodeKind};
use livery::values::{Display, Overflow, Position};

use crate::{
    LiveryLayout, StylePlane, ancestor_scroll, element_geometry, is_scroll_container,
    layout::fragment_edges,
};

/// One side of `rootMargin`/`scrollMargin`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarginValue {
    Px(f32),
    Percent(f32),
}

impl MarginValue {
    fn resolve(self, basis: f32) -> f32 {
        match self {
            Self::Px(px) => px,
            Self::Percent(percent) => basis * percent / 100.0,
        }
    }
}

/// Top, right, bottom, left.
pub type Margins = [MarginValue; 4];

pub const ZERO_MARGINS: Margins = [MarginValue::Px(0.0); 4];

/// A rectangle as `[left, top, right, bottom]`.
type Edges = [f32; 4];

/// What the observer needs for one target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intersection {
    /// The target's border box `[x, y, width, height]`; zeros when it has none.
    pub target_rect: [f32; 4],
    /// The visible part `[x, y, width, height]`; zeros when nothing shows.
    pub intersection_rect: [f32; 4],
    /// The root's intersection rectangle, margin included.
    pub root_bounds: Option<[f32; 4]>,
    /// Whether target and root intersect, edge-adjacent counting.
    pub is_intersecting: bool,
}

/// The retained frame an intersection is computed against.
pub struct IntersectionFrame<'a, D: LayoutDom>
where
    D::NodeId: Copy + Eq + Hash,
{
    pub dom: &'a D,
    pub styles: &'a StylePlane<D::NodeId>,
    pub fragments: &'a LiveryLayout<D::NodeId>,
    pub element_scroll: &'a HashMap<D::NodeId, (f32, f32)>,
    pub viewport: (f32, f32),
    pub viewport_scroll: (f32, f32),
}

fn to_rect(edges: Edges) -> [f32; 4] {
    [
        edges[0],
        edges[1],
        edges[2] - edges[0],
        edges[3] - edges[1],
    ]
}

/// Inclusive: boxes that only touch still intersect, with zero area.
fn clip(rect: Edges, by: Edges) -> Option<Edges> {
    let next = [
        rect[0].max(by[0]),
        rect[1].max(by[1]),
        rect[2].min(by[2]),
        rect[3].min(by[3]),
    ];
    (next[0] <= next[2] && next[1] <= next[3]).then_some(next)
}

fn expand(rect: Edges, margins: &Margins) -> Edges {
    let width = rect[2] - rect[0];
    let height = rect[3] - rect[1];
    [
        rect[0] - margins[3].resolve(width),
        rect[1] - margins[0].resolve(height),
        rect[2] + margins[1].resolve(width),
        rect[3] + margins[2].resolve(height),
    ]
}

impl<D: LayoutDom> IntersectionFrame<'_, D>
where
    D::NodeId: Copy + Eq + Hash,
{
    /// A node's fragment parts in client coordinates, scrolls above it removed.
    fn client_parts(&self, node: D::NodeId) -> Vec<Edges> {
        let above = ancestor_scroll(self.dom, self.element_scroll, node);
        let (dx, dy) = (
            self.viewport_scroll.0 + above.0,
            self.viewport_scroll.1 + above.1,
        );
        self.fragments
            .fragments_for_node(node)
            .map(|part| {
                [
                    part.x - dx,
                    part.y - dy,
                    part.x + part.width - dx,
                    part.y + part.height - dy,
                ]
            })
            .collect()
    }

    fn target_rect(&self, target: D::NodeId) -> Option<Edges> {
        if let Some(bounds) = element_geometry(
            self.dom,
            self.styles,
            self.fragments,
            self.element_scroll,
            target,
        )
        .and_then(|geometry| geometry.border_bounds())
        {
            let (sx, sy) = self.viewport_scroll;
            return Some([
                bounds[0] - sx,
                bounds[1] - sy,
                bounds[0] + bounds[2] - sx,
                bounds[1] + bounds[3] - sy,
            ]);
        }
        let parts = self.client_parts(target);
        let first = *parts.first()?;
        Some(parts.iter().fold(first, |all, part| {
            [
                all[0].min(part[0]),
                all[1].min(part[1]),
                all[2].max(part[2]),
                all[3].max(part[3]),
            ]
        }))
    }

    fn padding_box(&self, node: D::NodeId) -> Option<Edges> {
        let style = self.styles.get(node)?;
        let outer = *self.client_parts(node).first()?;
        let fragment = self.fragments.get(node)?;
        let border = fragment_edges(style, fragment).border;
        Some([
            outer[0] + border[3],
            outer[1] + border[0],
            outer[2] - border[1],
            outer[3] - border[2],
        ])
    }

    fn establishes_containing_block(&self, ancestor: D::NodeId, position: Position) -> bool {
        let Some(style) = self.styles.get(ancestor) else {
            return false;
        };
        if style.display == Display::None || self.fragments.get(ancestor).is_none() {
            return false;
        }
        let transformed = !style.transform.is_none();
        match position {
            Position::Fixed => transformed,
            Position::Absolute => transformed || style.position != Position::Static,
            _ => !matches!(style.display, Display::Inline),
        }
    }

    fn root_bounds(&self, root: Option<D::NodeId>, margins: &Margins) -> Option<Edges> {
        let base = match root {
            None => [0.0, 0.0, self.viewport.0, self.viewport.1],
            Some(root) => {
                let style = self.styles.get(root)?;
                if style.overflow_x != Overflow::Visible || style.overflow_y != Overflow::Visible {
                    self.padding_box(root)?
                } else {
                    self.target_rect(root)?
                }
            },
        };
        Some(expand(base, margins))
    }

    /// The `<body>` whose overflow reaches the viewport instead of clipping,
    /// which is the case while `<html>` leaves its own overflow visible.
    fn propagating_body(&self, html: Option<D::NodeId>) -> Option<D::NodeId> {
        let html = html?;
        let visible = self.styles.get(html).is_some_and(|style| {
            style.overflow_x == Overflow::Visible && style.overflow_y == Overflow::Visible
        });
        if !visible {
            return None;
        }
        self.dom.dom_children(html).find(|&id| {
            self.dom
                .element_name(id)
                .is_some_and(|name| &*name.local == "body")
        })
    }

    /// Clip `rect` by every container between `target` and the root, following
    /// the containing-block chain. The outer `None` means the root is not on
    /// the chain at all; the inner one that the clips left nothing.
    fn clip_through_chain(
        &self,
        target: D::NodeId,
        root: Option<D::NodeId>,
        root_bounds: Edges,
        scroll_margin: &Margins,
        mut rect: Edges,
    ) -> Option<Option<Edges>> {
        let html = self
            .dom
            .dom_children(self.dom.document())
            .find(|&id| self.dom.kind(id) == NodeKind::Element);
        let body = self.propagating_body(html);
        let mut current = target;
        let mut position = self.styles.get(target)?.position;
        loop {
            let mut ancestor = self.dom.parent(current);
            let mut container = None;
            while let Some(candidate) = ancestor {
                if self.dom.kind(candidate) != NodeKind::Element {
                    break;
                }
                if self.establishes_containing_block(candidate, position) {
                    container = Some(candidate);
                    break;
                }
                ancestor = self.dom.parent(candidate);
            }
            let Some(container) = container else {
                // Reached the viewport.
                return match root {
                    None => Some(clip(rect, root_bounds)),
                    Some(_) => None,
                };
            };
            if Some(container) == root {
                return Some(clip(rect, root_bounds));
            }
            let style = self.styles.get(container)?;
            let clips = (style.overflow_x != Overflow::Visible
                || style.overflow_y != Overflow::Visible)
                && Some(container) != html
                && Some(container) != body;
            if clips {
                let Some(mut area) = self.padding_box(container) else {
                    return Some(None);
                };
                if is_scroll_container(style) {
                    area = expand(area, scroll_margin);
                }
                match clip(rect, area) {
                    Some(next) => rect = next,
                    None => return Some(None),
                }
            }
            current = container;
            position = style.position;
        }
    }

    /// Compute the intersection of `target` with `root` (`None` is the
    /// viewport) under the two margins.
    pub fn intersection(
        &self,
        target: D::NodeId,
        root: Option<D::NodeId>,
        root_margin: &Margins,
        scroll_margin: &Margins,
    ) -> Intersection {
        let root_edges = self.root_bounds(root, root_margin);
        let miss = |target_rect: [f32; 4]| Intersection {
            target_rect,
            intersection_rect: [0.0; 4],
            root_bounds: root_edges.map(to_rect),
            is_intersecting: false,
        };
        let target_edges = self
            .dom
            .is_live(target)
            .then(|| self.target_rect(target))
            .flatten();
        let Some(target_edges) = target_edges else {
            return miss([0.0; 4]);
        };
        let target_rect = to_rect(target_edges);
        let Some(root_bounds) = root_edges else {
            return miss(target_rect);
        };
        match self.clip_through_chain(target, root, root_bounds, scroll_margin, target_edges) {
            Some(Some(visible)) => Intersection {
                target_rect,
                intersection_rect: to_rect(visible),
                root_bounds: Some(to_rect(root_bounds)),
                is_intersecting: true,
            },
            _ => miss(target_rect),
        }
    }
}
