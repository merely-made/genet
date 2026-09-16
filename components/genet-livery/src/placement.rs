// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bounded 2D paint placement shared by host content and pointer queries.

use std::{collections::HashMap, hash::Hash};

use layout_dom_api::{LayoutDom, NodeKind};
use livery::values::{Display, Matrix2D, Visibility};
use paint_list_api::{ClipKind, ClipSpec, DeviceIntSize, PathCommand, TransformSpec};

use crate::{LiveryLayout, StylePlane, layout::content_box_rect, paint};

/// A painted element's untransformed boxes and document-to-local mapping.
///
/// Coordinates are CSS pixels in the same document space as
/// [`crate::hit_test_with_scroll`]. Hosts add their document scroll offset to
/// viewport pointer coordinates before querying. The content origin excludes
/// borders and padding. Producer resolution follows this untransformed size;
/// CSS scale remains a compositor transform.
///
/// This uses the existing layout content-box query. Its percentage-padding
/// basis remains the fragment width; pixel padding and borders are exact.
/// A singular or non-finite transform has a size but cannot map input.
#[derive(Clone, Debug)]
pub struct ElementGeometry {
    pub content_rect: (f32, f32, f32, f32),
    border_rect: (f32, f32, f32, f32),
    inverse: Option<Matrix2D>,
    clips: Vec<GeometryClip>,
    border_clip_count: usize,
}

impl ElementGeometry {
    pub fn content_size(&self) -> (f32, f32) {
        (self.content_rect.2, self.content_rect.3)
    }

    pub fn border_size(&self) -> (f32, f32) {
        (self.border_rect.2, self.border_rect.3)
    }

    /// Inverse paint transform into content-local coordinates, including
    /// positions outside the element. Intended for an existing pointer capture.
    pub fn map_to_local(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let (x, y) = map_point(self.inverse?, x, y)?;
        Some((x - self.content_rect.0, y - self.content_rect.1))
    }

    /// Inverse-only border-local coordinates for ordinary DOM controls.
    pub fn map_to_border_local(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let (x, y) = map_point(self.inverse?, x, y)?;
        Some((x - self.border_rect.0, y - self.border_rect.1))
    }

    /// Map a visible content point. Ancestor clips, this element's clip path,
    /// overflow clip, and its content boundary all apply. DOM occlusion is
    /// separate: route ordinary hit testing first so overlay controls win.
    pub fn map_to_content(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if !self.clips.iter().all(|clip| clip.contains(x, y)) {
            return None;
        }
        let local = self.map_to_local(x, y)?;
        contains((0.0, 0.0, self.content_rect.2, self.content_rect.3), local).then_some(local)
    }

    pub(crate) fn hits_border(&self, x: f32, y: f32) -> bool {
        self.clips[..self.border_clip_count]
            .iter()
            .all(|clip| clip.contains(x, y))
            && self.map_to_border_local(x, y).is_some_and(|(x, y)| {
                border_axis_contains(self.border_rect.2, x)
                    && border_axis_contains(self.border_rect.3, y)
            })
    }
}

/// Half-open like painted coverage, except that an axis without extent holds
/// only its own line: Livery gives an unsized form control a zero-extent box,
/// and pointers aimed at its centre land exactly there.
fn border_axis_contains(extent: f32, offset: f32) -> bool {
    if extent == 0.0 {
        offset == 0.0
    } else {
        extent > 0.0 && offset >= 0.0 && offset < extent
    }
}

/// Query a currently painted element using the same transform-origin,
/// inherited 2D transforms, and scroll/clip scopes as paint. Hidden ancestors
/// and missing fragments return `None`. No layout or producer work is done.
pub fn element_geometry<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
    node: D::NodeId,
) -> Option<ElementGeometry>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let mut ancestors = Vec::new();
    let mut current = Some(node);
    while let Some(id) = current {
        ancestors.push(id);
        current = dom.parent(id);
    }
    let mut placement = PlacementState::default();
    for id in ancestors.into_iter().rev() {
        let entered = placement.enter(dom, styles, fragments, scroll_offsets, id)?;
        if id == node {
            return entered.geometry;
        }
        placement = entered.children;
    }
    None
}

#[derive(Clone, Debug)]
struct GeometryClip {
    inverse: Option<Matrix2D>,
    clip: ClipSpec,
}

impl GeometryClip {
    fn contains(&self, x: f32, y: f32) -> bool {
        let Some((x, y)) = self.inverse.and_then(|inverse| map_point(inverse, x, y)) else {
            return false;
        };
        match &self.clip.kind {
            ClipKind::Rect(rect) => contains(
                (rect.min.x, rect.min.y, rect.width(), rect.height()),
                (x, y),
            ),
            ClipKind::Path(path) => {
                // Livery's admitted clip-path emits one straight-edge polygon.
                let points = path
                    .commands
                    .iter()
                    .filter_map(|command| match command {
                        PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                            Some((point.x, point.y))
                        },
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                point_in_polygon(&points, x, y)
            },
            _ => false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PlacementState {
    matrix: Matrix2D,
    clips: Vec<GeometryClip>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            matrix: Matrix2D::IDENTITY,
            clips: Vec::new(),
        }
    }
}

pub(crate) struct EnteredPlacement {
    pub geometry: Option<ElementGeometry>,
    pub children: PlacementState,
}

impl PlacementState {
    pub(crate) fn enter<D>(
        &self,
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        fragments: &LiveryLayout<D::NodeId>,
        scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
        id: D::NodeId,
    ) -> Option<EnteredPlacement>
    where
        D: LayoutDom,
        D::NodeId: Copy + Eq + Hash,
    {
        let mut next = self.clone();
        let mut geometry = None;
        if dom.kind(id) == NodeKind::Element {
            let style = styles.get(id)?;
            if style.display == Display::None || style.visibility != Visibility::Visible {
                return None;
            }
            if let Some(fragment) = fragments.get(id) {
                let parent = paint::transform_parent(dom, styles, fragments, id);
                if let Some(transform) = paint::transform_spec(style, fragment, parent) {
                    next.matrix = next.matrix.multiply(spec_matrix(&transform));
                }
                let inverse = inverse(next.matrix);
                if let Some(clip) = paint::polygon_clip(style, fragment) {
                    next.clips.push(GeometryClip { inverse, clip });
                }
                let border_clip_count = next.clips.len();
                let paints_as_inline =
                    matches!(style.display, Display::Inline | Display::InlineBlock)
                        && !paint::is_blockified_item(fragments, id);
                if !paints_as_inline {
                    let viewport = DeviceIntSize::new(
                        fragments.viewport.0 as i32,
                        fragments.viewport.1 as i32,
                    );
                    if let Some(clip) = paint::descendant_clip(style, fragment, viewport) {
                        next.clips.push(GeometryClip { inverse, clip });
                    }
                }
                geometry = Some(ElementGeometry {
                    content_rect: content_box_rect(style, fragment),
                    border_rect: (fragment.x, fragment.y, fragment.width, fragment.height),
                    inverse,
                    clips: next.clips.clone(),
                    border_clip_count,
                });
            }
        }
        // Own replaced content remains stationary within its content box;
        // only DOM descendants receive the element's scroll offset.
        if let Some(&(x, y)) = scroll_offsets.get(&id) {
            next.matrix = next
                .matrix
                .multiply(Matrix2D::new(1.0, 0.0, 0.0, 1.0, -x, -y));
        }
        Some(EnteredPlacement {
            geometry,
            children: next,
        })
    }
}

/// TransformSpec applies its matrix, then its placement origin. This reads
/// paint's exact resolved matrix instead of maintaining a second CSS parser.
fn spec_matrix(spec: &TransformSpec) -> Matrix2D {
    let matrix = &spec.transform;
    Matrix2D::new(
        matrix.m11,
        matrix.m12,
        matrix.m21,
        matrix.m22,
        matrix.m41 + spec.origin.x,
        matrix.m42 + spec.origin.y,
    )
}

fn inverse(matrix: Matrix2D) -> Option<Matrix2D> {
    if !matrix.is_finite() {
        return None;
    }
    let determinant = matrix.a * matrix.d - matrix.b * matrix.c;
    if !determinant.is_finite() || determinant == 0.0 {
        return None;
    }
    let inverse = Matrix2D::new(
        matrix.d / determinant,
        -matrix.b / determinant,
        -matrix.c / determinant,
        matrix.a / determinant,
        (matrix.c * matrix.f - matrix.d * matrix.e) / determinant,
        (matrix.b * matrix.e - matrix.a * matrix.f) / determinant,
    );
    inverse.is_finite().then_some(inverse)
}

fn map_point(matrix: Matrix2D, x: f32, y: f32) -> Option<(f32, f32)> {
    let point = (
        matrix.a * x + matrix.c * y + matrix.e,
        matrix.b * x + matrix.d * y + matrix.f,
    );
    (point.0.is_finite() && point.1.is_finite()).then_some(point)
}

fn contains(rect: (f32, f32, f32, f32), point: (f32, f32)) -> bool {
    rect.2 > 0.0
        && rect.3 > 0.0
        && point.0 >= rect.0
        && point.1 >= rect.1
        && point.0 < rect.0 + rect.2
        && point.1 < rect.1 + rect.3
}

fn point_in_polygon(points: &[(f32, f32)], x: f32, y: f32) -> bool {
    let Some(&last) = points.last() else {
        return false;
    };
    let mut previous = last;
    let mut winding = 0_i32;
    for &(cx, cy) in points {
        let (px, py) = previous;
        let side = (cx - px) * (y - py) - (x - px) * (cy - py);
        if side.abs() <= f32::EPSILON
            && x >= px.min(cx)
            && x <= px.max(cx)
            && y >= py.min(cy)
            && y <= py.max(cy)
        {
            return true;
        }
        if py <= y {
            if cy > y && side > 0.0 {
                winding += 1;
            }
        } else if cy <= y && side < 0.0 {
            winding -= 1;
        }
        previous = (cx, cy);
    }
    winding != 0
}
