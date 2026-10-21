// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Element scroll geometry over a retained fragment plane: the scrollport's
//! client box, its scrollable overflow, and the scroll offsets that sit above a
//! node. Pure functions of the DOM, the style plane and the fragments, so a
//! scripted host and a test can ask without a document object.

use std::{collections::HashMap, hash::Hash};

use layout_dom_api::LayoutDom;
use livery::values::{Display, Overflow};

use crate::{LiveryLayout, StylePlane, layout::fragment_edges, paint};

/// A scroll container's client box and scrollable overflow, in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollMetrics {
    /// Border widths `[top, right, bottom, left]`; the padding box starts there.
    pub border: [f32; 4],
    /// The padding box size, which is what `clientWidth`/`clientHeight` report
    /// while no scrollbar takes space.
    pub client: (f32, f32),
    /// The scrollable overflow rectangle's size, never smaller than `client`.
    pub scroll: (f32, f32),
}

impl ScrollMetrics {
    /// The farthest `scrollLeft`/`scrollTop` reaches.
    pub fn max_offset(&self) -> (f32, f32) {
        (
            (self.scroll.0 - self.client.0).max(0.0),
            (self.scroll.1 - self.client.1).max(0.0),
        )
    }
}

/// Whether script can scroll the element: any `overflow` other than `visible`
/// and `clip` makes a scroll container, `hidden` included.
pub fn is_scroll_container(style: &livery::ComputedValues) -> bool {
    let scrolls = |overflow| !matches!(overflow, Overflow::Visible | Overflow::Clip);
    scrolls(style.overflow_x) || scrolls(style.overflow_y)
}

fn clips_content(style: &livery::ComputedValues) -> bool {
    style.overflow_x != Overflow::Visible || style.overflow_y != Overflow::Visible
}

/// The sum of the scroll offsets of every ancestor of `id`.
pub fn ancestor_scroll<D>(
    dom: &D,
    offsets: &HashMap<D::NodeId, (f32, f32)>,
    id: D::NodeId,
) -> (f32, f32)
where
    D: LayoutDom,
{
    let mut sum = (0.0, 0.0);
    let mut parent = dom.parent(id);
    while let Some(ancestor) = parent {
        if let Some(offset) = offsets.get(&ancestor) {
            sum.0 += offset.0;
            sum.1 += offset.1;
        }
        parent = dom.parent(ancestor);
    }
    sum
}

/// Client box and scrollable overflow of `node`. `None` without a fragment or a
/// style. A box that is inline and not replaced has no client box.
pub fn scroll_metrics<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    node: D::NodeId,
) -> Option<ScrollMetrics>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let style = styles.get(node)?;
    let fragment = fragments.get(node)?;
    if style.display == Display::None {
        return None;
    }
    if paint::paints_as_inline_box(fragments, node, style) {
        return Some(ScrollMetrics::default());
    }
    let edges = fragment_edges(style, fragment);
    let client = (
        (fragment.width - edges.border[1] - edges.border[3]).max(0.0),
        (fragment.height - edges.border[0] - edges.border[2]).max(0.0),
    );
    let origin = (fragment.x + edges.border[3], fragment.y + edges.border[0]);
    let mut extent = (0.0_f32, 0.0_f32);
    let mut grow = |x: f32, y: f32, width: f32, height: f32| {
        extent.0 = extent.0.max(x + width - origin.0);
        extent.1 = extent.1.max(y + height - origin.1);
    };
    if let Some(lines) = fragments.inline_scroll_bounds(node) {
        grow(lines.x, lines.y, lines.width, lines.height);
    }
    let mut stack: Vec<D::NodeId> = dom.dom_children(node).collect();
    while let Some(id) = stack.pop() {
        let child_style = styles.get(id);
        if child_style.is_some_and(|style| style.display == Display::None) {
            continue;
        }
        for part in fragments.fragments_for_node(id) {
            grow(part.x, part.y, part.width, part.height);
        }
        if child_style.is_some_and(clips_content) {
            continue;
        }
        if let Some(lines) = fragments.inline_scroll_bounds(id) {
            grow(lines.x, lines.y, lines.width, lines.height);
        }
        stack.extend(dom.dom_children(id));
    }
    // In-flow content that overflows keeps the container's end padding inside
    // the scrollable area.
    if extent.0 > 0.0 {
        extent.0 += edges.padding[1];
    }
    if extent.1 > 0.0 {
        extent.1 += edges.padding[2];
    }
    Some(ScrollMetrics {
        border: edges.border,
        client,
        scroll: (extent.0.max(client.0), extent.1.max(client.1)),
    })
}
