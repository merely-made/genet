// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A `::before` / `::after` whose `content` is a single `url(...)` image is a
//! replaced atomic inline box (css-content-3 section 1.2).
//!
//! Box generation marks it a replaced pseudo box; layout sizes it as an
//! `<img>` and lays it in the line as an atom (`layout/generated_image.rs`,
//! `text.rs`); this module owns what is shared and its paint. The image is
//! registered by the embedder under the declared URL (resolved against the
//! declaring sheet), as a vector image or as raster bytes. An unregistered
//! URL generates a zero-size box that paints nothing.
//!
//! Limits: `content` mixing strings and images keeps its text and ignores the
//! images, and an owner inside an inline-block or inline-table, or one that is
//! not a block or inline box, generates no image (see `box_tree.rs`).

use std::hash::Hash;

use buckram::{BoxOrigin, BoxId, CssBox, CssBoxTree, PseudoElement};
use layout_dom_api::LayoutDom;
use livery::values::Visibility;

use crate::{LiveryLayout, StylePlane, paint::PaintBuilder};

/// A box generated for a pseudo's replaced image (never a marker).
pub(crate) fn is_generated_image<Id>(css_box: &CssBox<Id>) -> bool {
    css_box.replaced
        && matches!(
            css_box.origin,
            BoxOrigin::Pseudo { pseudo, .. } if pseudo != PseudoElement::Marker
        )
}

/// The node an inline placement is recorded against. A generated image owns
/// no DOM node: attributing its rectangle to the owner would stretch the
/// owner's own inline fragments, so it has none and paints from its box.
pub(crate) fn inline_node<Id: Copy + Eq + Hash>(
    boxes: &CssBoxTree<Id>,
    box_id: BoxId,
) -> Option<Id> {
    if is_generated_image(&boxes[box_id]) {
        None
    } else {
        boxes.origin_node(box_id)
    }
}

/// Paint `owner`'s generated images where they were placed in its lines.
pub(crate) fn paint<D>(
    styles: &StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    owner: D::NodeId,
    list: &mut PaintBuilder<'_>,
) where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let boxes = fragments.boxes();
    for &box_id in boxes.boxes_for_node(owner) {
        let css_box = &boxes[box_id];
        let BoxOrigin::Pseudo { pseudo, .. } = css_box.origin else {
            continue;
        };
        if !is_generated_image(css_box)
            || styles
                .generated_style(owner, pseudo)
                .is_none_or(|style| style.visibility != Visibility::Visible)
        {
            continue;
        }
        let Some(source) = styles
            .generated_image(owner, pseudo)
            .and_then(|url| list.replaced_for(url))
        else {
            continue;
        };
        for &id in fragments.fragments().fragment_ids_for_box(box_id) {
            if let Some(fragment) = fragments.fragments().get(id) {
                let rect = fragment.physical_rect();
                if rect.width > 0.0 && rect.height > 0.0 {
                    list.push_replaced(&source, crate::paint::bounds(&rect));
                }
            }
        }
    }
}
