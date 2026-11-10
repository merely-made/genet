// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A `::before` / `::after` whose `content` is one `url(...)` image: a
//! replaced atomic inline box (css-content-3 section 1.2). It is sized as an
//! `<img>` is, from the pseudo's own computed width and height and the
//! image's natural size, and joins the atomic plane like any replaced root.

use super::*;
pub(super) use crate::generated_image::is_generated_image;

/// The natural size of the image registered under `url`: a vector image
/// first, then decoded raster bytes.
pub(super) fn natural_size(
    image_sources: &dyn crate::ImageSourceLookup,
    url: &str,
) -> Option<(f32, f32)> {
    if let Some(vector) = image_sources.vector(url) {
        return Some(vector.intrinsic_size());
    }
    let image = image_sources.decoded(url)?;
    Some((image.width as f32, image.height as f32))
}

impl<D> BuildState<'_, D>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    /// The replaced leaf for a generated image. An image nobody registered
    /// (or one that has not loaded) is a zero-size leaf that paints nothing.
    pub(in crate::layout) fn build_generated_image(
        &mut self,
        box_id: BoxId,
        owner: D::NodeId,
        pseudo: buckram::PseudoElement,
        parent_font_size: f32,
        containing_size: (Option<f32>, Option<f32>),
    ) -> Result<Option<AlgorithmNodeId>, LayoutError> {
        let Some(computed) = self.styles.generated_style(owner, pseudo).cloned() else {
            return Ok(None);
        };
        let font_size = font_size_px(&computed.font_size, parent_font_size);
        let natural = self
            .styles
            .generated_image(owner, pseudo)
            .and_then(|url| natural_size(self.image_sources, url));
        let block_style = to_block_style(self.boxes, self.styles, box_id, &computed, font_size);
        let (taffy_style, width, height) = if let Some(natural) = natural {
            let mut style = to_taffy_style(&computed, font_size);
            style.size.width = dimension_with_basis(computed.width, font_size, containing_size.0);
            style.size.height =
                dimension_with_basis(computed.height, font_size, containing_size.1);
            style.min_size.width =
                size_limit_with_basis(computed.min_width, font_size, containing_size.0);
            style.min_size.height =
                size_limit_with_basis(computed.min_height, font_size, containing_size.1);
            style.max_size.width =
                size_limit_with_basis(computed.max_width, font_size, containing_size.0);
            style.max_size.height =
                size_limit_with_basis(computed.max_height, font_size, containing_size.1);
            let (width, height) = apply_natural_replaced_size(
                &mut style,
                Some(natural),
                &computed,
                font_size,
                false,
                containing_size.0.unwrap_or(0.0),
            )
            .unwrap_or(natural);
            style.item_is_replaced = true;
            (style, width, height)
        } else {
            let style = Style {
                size: Size {
                    width: Dimension::length(0.0),
                    height: Dimension::length(0.0),
                },
                ..Style::default()
            };
            (style, 0.0, 0.0)
        };
        Ok(Some(self.tree.new_leaf_with_context_and_block_style(
            block_style,
            taffy_style,
            TextMeasure {
                min_width: width,
                max_width: width,
                height,
                baselines: None,
                wrap: None,
            },
            Some(box_id),
        )))
    }
}
