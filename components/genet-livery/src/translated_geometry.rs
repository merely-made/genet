// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Batched retained geometry for hosts whose placements use CSS translations.

use std::collections::HashMap;

use layout_dom_api::{LayoutDom, NodeKind};
use livery::values::{Display, Visibility};

use crate::{LiveryLayout, StylePlane, document::visible_generated_text};

#[cfg(test)]
mod tests;

/// One node's retained geometry and style facts, in CSS pixels.
///
/// `border_rect` applies the translation terms of each ancestor's and this
/// node's `transform` matrix, then removes ancestor element scroll. This is
/// the translate-only host convention: it does not project rotated, scaled,
/// perspective, or individual transform properties. Use [`crate::element_geometry`]
/// for the full supported 2D paint mapping. Viewport scroll and physical-pixel
/// scaling belong to the host and are not applied here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TranslatedGeometry<'a> {
    pub border_rect: Option<(f32, f32, f32, f32)>,
    /// Content box, with the same translation and ancestor-scroll adjustment
    /// as `border_rect`. Borders and padding use the existing layout query's
    /// fragment-width percentage-padding basis.
    pub content_rect: Option<(f32, f32, f32, f32)>,
    /// Inclusive accumulated `transform` matrix translation terms.
    pub translation: (f32, f32),
    /// Ancestor offsets only: a node's own scroll moves its descendants.
    pub ancestor_scroll: (f32, f32),
    /// No `display: none` ancestor and computed visible rendering. A visible
    /// child of a `visibility: hidden` parent can be rendered.
    pub rendered: bool,
    /// Every element from the scope root through this node has a style and
    /// visible, non-`none` rendering. This uses `element_geometry`'s style
    /// admission rule on the walked flat tree, including rejection of hidden
    /// ancestors even when a descendant overrides visibility. It is distinct
    /// from `rendered`.
    pub placement_visible: bool,
    /// Admitted visible inline before/after text, borrowed from the style
    /// owner. Suppressed and replaced owners contribute no generated text.
    pub generated_text: (&'a str, &'a str),
}

#[derive(Clone, Copy, Default)]
struct Inherited {
    translation: (f32, f32),
    scroll: (f32, f32),
    display_none: bool,
    placement_hidden: bool,
    parent_element_rendered: Option<bool>,
}

/// Visit retained geometry in flat-tree preorder with one descent and no
/// ancestor queries. The supplied DOM's `document()` is the scope boundary:
/// ancestor styles, transforms and scroll outside a subtree view do not leak
/// into its local layout space. Missing fragments do not prevent descent.
///
/// No geometry cache is retained. Every invocation reads the caller's current
/// fragments, styles and scroll offsets, including restyles without layout.
pub fn walk_translated_geometry<'a, D: LayoutDom>(
    dom: &D,
    styles: &'a StylePlane<D::NodeId>,
    fragments: &LiveryLayout<D::NodeId>,
    scroll_offsets: &HashMap<D::NodeId, (f32, f32)>,
    mut visit: impl FnMut(D::NodeId, TranslatedGeometry<'a>),
) {
    let mut pending = vec![(dom.document(), Inherited::default())];
    while let Some((node, inherited)) = pending.pop() {
        let style = styles.get(node);
        let fragment = fragments.get(node);
        let own_translation = style
            .zip(fragment)
            .and_then(|(style, fragment)| {
                style
                    .transform
                    .to_matrix(16.0, (fragment.width, fragment.height))
            })
            .map_or((0.0, 0.0), |matrix| (matrix.e, matrix.f));
        let translation = (
            inherited.translation.0 + own_translation.0,
            inherited.translation.1 + own_translation.1,
        );
        let display_none =
            inherited.display_none || style.is_some_and(|style| style.display == Display::None);
        let kind = dom.kind(node);
        let rendered = match kind {
            NodeKind::Document => true,
            NodeKind::Element => {
                !display_none && style.is_some_and(|style| style.visibility == Visibility::Visible)
            },
            _ => !display_none && inherited.parent_element_rendered.unwrap_or(true),
        };
        let placement_visible = !inherited.placement_hidden
            && (kind != NodeKind::Element
                || style.is_some_and(|style| {
                    style.display != Display::None && style.visibility == Visibility::Visible
                }));
        let generated_text = if display_none || crate::box_tree::is_replaced_element(dom, node) {
            ("", "")
        } else {
            visible_generated_text(styles, node)
        };
        visit(
            node,
            TranslatedGeometry {
                border_rect: fragment.map(|fragment| {
                    (
                        fragment.x + translation.0 - inherited.scroll.0,
                        fragment.y + translation.1 - inherited.scroll.1,
                        fragment.width,
                        fragment.height,
                    )
                }),
                content_rect: style.zip(fragment).map(|(style, fragment)| {
                    let (x, y, width, height) = crate::layout::content_box_rect(style, fragment);
                    (
                        x + translation.0 - inherited.scroll.0,
                        y + translation.1 - inherited.scroll.1,
                        width,
                        height,
                    )
                }),
                translation,
                ancestor_scroll: inherited.scroll,
                rendered,
                placement_visible,
                generated_text,
            },
        );
        let own_scroll = scroll_offsets.get(&node).copied().unwrap_or_default();
        let children = Inherited {
            translation,
            scroll: (
                inherited.scroll.0 + own_scroll.0,
                inherited.scroll.1 + own_scroll.1,
            ),
            display_none,
            placement_hidden: !placement_visible,
            parent_element_rendered: if kind == NodeKind::Element {
                Some(rendered)
            } else {
                inherited.parent_element_rendered
            },
        };
        let nodes: Vec<_> = dom.flat_children(node).collect();
        pending.extend(nodes.into_iter().rev().map(|node| (node, children)));
    }
}
