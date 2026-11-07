// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One home for the retained pipeline's `ScriptedDom` instantiation.
//!
//! [`LiveryDocument`], `layout`, `emit_paint_list` and the rest are generic
//! over the DOM backend, and every Genet and Mere crate that drives a
//! `LiveryDocument<ScriptedDom>` is a sibling of the others: none is upstream
//! of another, so each one used to monomorphize its own copy of the whole
//! cascade, box-tree, layout and paint pipeline (the Mere web viewer carried
//! two copies, about 419 KB of code, and which crates held a copy changed with
//! the checkout path through the crates' hashes). With shared generics
//! (rustc's default at `opt-level` `s` and `z`) a crate reuses an instance an
//! upstream crate already holds, so this crate, upstream of all of them, holds
//! the instances once and the siblings link to them.
//!
//! The function below exists only to make those instances here. It is never
//! called; the linker drops it, and keeps the instances the siblings use.
//! `every_public_method_is_anchored` fails when a `pub fn` is added to a
//! `LiveryDocument` impl without being listed.

use genet_scripted_dom::ScriptedDom as Dom;

use crate::{
    LiveryDocument, ancestor_scroll, element_geometry, emit_paint_list,
    emit_paint_list_with_text_system, emit_paint_list_with_text_system_scrolled_with_images,
    emit_paint_list_with_text_system_scrolled_with_images_and_document_base,
    emit_paint_list_with_text_system_scrolled_with_images_and_external_textures,
    emit_paint_list_with_text_system_scrolled_with_images_and_external_textures_with_document_base,
    estimated_used_value_context, hit_test, hit_test_with_scroll, inline_boxes, layout,
    layout_with_text_system, rendered_generated_text, rendered_visible,
    resolve_container_query_styles, resolve_container_query_styles_with_images,
    resolve_container_relative_styles, resolve_styles, scroll_metrics,
    used_value_context_with_text_system,
};

type Doc = LiveryDocument<Dom>;

/// Takes the address of every `LiveryDocument<ScriptedDom>` entry point and of
/// the generic free functions, so this crate instantiates them. Never call it.
#[doc(hidden)]
pub fn scripted_dom_instances() -> usize {
    let entries: &[*const ()] = &[
        Doc::new as *const (),
        Doc::dom as *const (),
        Doc::push_validation_notice as *const (),
        Doc::set_text_shaping_options as *const (),
        Doc::text_system as *const (),
        Doc::generation as *const (),
        Doc::has_current_frame as *const (),
        Doc::layout_generation as *const (),
        Doc::interactions_mut as *const (),
        Doc::apply_dom_mutations as *const (),
        Doc::set_preferred_color_scheme as *const (),
        Doc::set_opaque_canvas as *const (),
        Doc::set_system_palette as *const (),
        Doc::last_restyle_stats as *const (),
        Doc::table_shadow_ledger as *const (),
        Doc::insert_author_rule as *const (),
        Doc::delete_author_rule as *const (),
        Doc::style_set as *const (),
        Doc::computed_style as *const (),
        Doc::retained_layout as *const (),
        Doc::generated_text as *const (),
        Doc::rendered_visible as *const (),
        Doc::hit_test as *const (),
        Doc::fragment_rect as *const (),
        Doc::content_rect as *const (),
        Doc::into_dom as *const (),
        Doc::caret_rect as *const (),
        Doc::text_position_at_point as *const (),
        Doc::selection_for_range as *const (),
        Doc::links as *const (),
        Doc::begin_text_selection as *const (),
        Doc::extend_text_selection as *const (),
        Doc::finish_text_selection as *const (),
        Doc::text_selection as *const (),
        Doc::select_text_range as *const (),
        Doc::activate_text_directives as *const (),
        Doc::scroll_to_element_fragment as *const (),
        Doc::links_for_selection as *const (),
        Doc::text_target as *const (),
        Doc::click_at as *const (),
        Doc::invalidate as *const (),
        Doc::last_layout_damage as *const (),
        Doc::frame as *const (),
        Doc::scroll as *const (),
        Doc::scroll_by as *const (),
        Doc::scroll_at as *const (),
        Doc::scroll_accessible_node_into_view as *const (),
        Doc::accessible_pointer_target as *const (),
        Doc::scroll_to as *const (),
        Doc::scroll_line as *const (),
        Doc::scroll_page as *const (),
        Doc::content_height as *const (),
        Doc::element_scroll as *const (),
        Doc::animate_opacity as *const (),
        Doc::pump as *const (),
        Doc::settled as *const (),
        layout::<Dom> as *const (),
        layout_with_text_system::<Dom> as *const (),
        estimated_used_value_context::<Dom> as *const (),
        used_value_context_with_text_system::<Dom> as *const (),
        resolve_container_query_styles::<Dom> as *const (),
        resolve_container_query_styles_with_images::<Dom> as *const (),
        resolve_container_relative_styles::<Dom> as *const (),
        hit_test::<Dom> as *const (),
        hit_test_with_scroll::<Dom> as *const (),
        emit_paint_list::<Dom> as *const (),
        emit_paint_list_with_text_system::<Dom> as *const (),
        emit_paint_list_with_text_system_scrolled_with_images::<Dom> as *const (),
        emit_paint_list_with_text_system_scrolled_with_images_and_document_base::<Dom> as *const (),
        emit_paint_list_with_text_system_scrolled_with_images_and_external_textures::<Dom> as *const (),
        emit_paint_list_with_text_system_scrolled_with_images_and_external_textures_with_document_base::<Dom> as *const (),
        element_geometry::<Dom> as *const (),
        resolve_styles::<Dom> as *const (),
        ancestor_scroll::<Dom> as *const (),
        inline_boxes::<Dom> as *const (),
        scroll_metrics::<Dom> as *const (),
        rendered_visible::<Dom> as *const (),
        rendered_generated_text::<Dom> as *const (),
    ];
    std::hint::black_box(entries).len()
}

#[cfg(test)]
mod tests {
    /// Every `pub fn` in a `LiveryDocument<D>` impl (closure-generic ones
    /// aside) is named in the anchor, so a new entry point cannot start a
    /// second instantiation unnoticed.
    #[test]
    fn every_public_method_is_anchored() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let anchor = std::fs::read_to_string(root.join("scripted_instance.rs")).unwrap();
        let mut missing = Vec::new();
        for file in [
            "document.rs",
            "document/selection.rs",
            "document/frame.rs",
            "document/scrolling.rs",
            "document/animation.rs",
            "document/resources.rs",
        ] {
            let source = std::fs::read_to_string(root.join(file)).unwrap();
            let start = source.find("impl<D> LiveryDocument<D>").unwrap();
            let body = &source[start..];
            let body = match body.find("\n#[cfg(test)]\nmod ") {
                Some(end) => &body[..end],
                None => body,
            };
            let lines: Vec<&str> = body.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                let Some(rest) = line.strip_prefix("    pub fn ") else {
                    continue;
                };
                // Behind a cfg this anchor cannot name (`set_text_shaping_pool`,
                // native-only).
                if index > 0 && lines[index - 1].starts_with("    #[cfg(") {
                    continue;
                }
                let name = rest.split(['(', '<']).next().unwrap();
                // The signature runs to the line that opens the body. Generic
                // over a closure or an argument type (`<R>`, `impl Trait`):
                // instantiated per call site, so there is nothing to anchor.
                let signature = lines[index..]
                    .iter()
                    .take_while(|line| !line.ends_with('{'))
                    .chain(lines[index..].iter().find(|line| line.ends_with('{')))
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                if rest[name.len()..].starts_with('<') || signature.contains("impl ") {
                    continue;
                }
                if !anchor.contains(&format!("Doc::{name} as *const ()")) {
                    missing.push(format!("{file}: {name}"));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "not in scripted_dom_instances: {missing:?}"
        );
    }
}
