// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use livery::{
    ComputedValues,
    values::{BackdropFilter, Display, Visibility},
};
use paint_list_api::{BackdropBlurSpec, DeviceIntSize, PaintCmd};

use super::{Fragment, LiveryPaintList, border_radius, bounds, paintable_fragment};

pub(super) fn is_backdrop_blur(command: &PaintCmd) -> bool {
    matches!(command, PaintCmd::DrawBackdropBlur(_))
}

/// Admit one pane in the document root's coordinate space. Inspect each
/// preceding command at most once, and only when we find a candidate pane.
#[derive(Default)]
pub(super) struct BackdropPaint {
    scanned: usize,
    depth: usize,
    emitted: bool,
    has_unresolved_prefix_filter: bool,
}

impl BackdropPaint {
    pub(super) fn draw(
        &mut self,
        list: &mut LiveryPaintList,
        style: &ComputedValues,
        fragment: &Fragment,
    ) {
        if self.emitted || self.has_unresolved_prefix_filter {
            return;
        }
        let Some(spec) = backdrop_blur(style, fragment, list.viewport) else {
            return;
        };
        for command in &list.commands[self.scanned..] {
            match command {
                PaintCmd::PushLayer(layer) if !layer.filters.is_empty() => {
                    // Classic processes backdrops ahead of element filters.
                    // An earlier such layer would contribute the wrong pixels.
                    self.has_unresolved_prefix_filter = true;
                    return;
                },
                PaintCmd::PushLayer(_) | PaintCmd::PushClip(_) | PaintCmd::PushTransform(_) => {
                    self.depth += 1;
                },
                PaintCmd::PopLayer | PaintCmd::PopClip | PaintCmd::PopTransform => {
                    self.depth = self.depth.saturating_sub(1);
                },
                _ => {},
            }
        }
        self.scanned = list.commands.len();
        if self.depth == 0 {
            list.commands.push(PaintCmd::DrawBackdropBlur(spec));
            self.emitted = true;
        }
    }
}

fn backdrop_blur(
    style: &ComputedValues,
    fragment: &Fragment,
    viewport: DeviceIntSize,
) -> Option<BackdropBlurSpec> {
    let BackdropFilter::Blur(sigma) = style.backdrop_filter else {
        return None;
    };
    // The renderer uses a radius parameter of two standard deviations.
    let radius_px = 2.0 * sigma;
    if !radius_px.is_finite()
        || radius_px <= 0.0
        || !paintable_fragment(fragment)
        || matches!(style.display, Display::None | Display::Contents)
        || style.visibility != Visibility::Visible
        || style.opacity.value() != 1.0
        || !style.filter.is_none()
        || !style.clip_path.is_none()
    {
        return None;
    }
    let corners = border_radius(style, fragment);
    if [
        corners.top_left,
        corners.top_right,
        corners.bottom_left,
        corners.bottom_right,
    ]
    .iter()
    .any(|corner| corner.width != 0.0 || corner.height != 0.0)
    {
        return None;
    }
    let bounds = bounds(fragment);
    if [bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y]
        .iter()
        .any(|edge| edge.fract() != 0.0)
        || bounds.min.x < 0.0
        || bounds.min.y < 0.0
        || bounds.max.x > viewport.width as f32
        || bounds.max.y > viewport.height as f32
    {
        return None;
    }
    Some(BackdropBlurSpec { bounds, radius_px })
}
