// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The retained frame: layout damage tracking and the paint pipeline.
//!
//! Damage decides how much of the retained tree a frame may reuse;
//! `frame` runs the pipeline that honours that decision, including the
//! sticky-positioning pass that needs the laid-out scrollport.

use super::*;
use crate::invalidation::{MutationDependency, for_each_mutation_dependency};

impl<D> LiveryDocument<D>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    pub fn invalidate(&mut self) {
        self.invalidate_with_layout_damage(LayoutDamageKind::Device);
    }

    pub(in crate::document) fn invalidate_with_layout_damage(&mut self, kind: LayoutDamageKind) {
        self.record_full_layout_damage(kind);
        self.retain_layout_identity();
        self.style_session.invalidate();
    }

    /// The latest retained-layout damage classification. The current K5g
    /// correctness path still rebuilds a full layout; K5h will replace these
    /// named roots rather than treating `RestyleStats` as geometry proof.
    pub fn last_layout_damage(&self) -> Option<&LayoutDamage<D::NodeId>> {
        self.last_layout_damage.as_ref()
    }

    pub(in crate::document) fn record_full_layout_damage(&mut self, kind: LayoutDamageKind) {
        self.style_only_damage = false;
        self.last_layout_damage = Some(LayoutDamage {
            kind,
            roots: vec![self.dom.document()],
            full_document: true,
        });
    }

    pub(in crate::document) fn record_dom_layout_damage(
        &mut self,
        mutations: &[DomMutation<D::NodeId>],
    ) {
        self.style_only_damage &= mutations.iter().all(|mutation| {
            matches!(mutation, DomMutation::AttributeChanged { name, .. }
                if name.ns == Namespace::from("") && name.local == LocalName::from("style"))
        });
        let mut roots = Vec::new();
        for mutation in mutations {
            for_each_mutation_dependency(&self.dom, mutation, |dependency| {
                let node = match dependency {
                    MutationDependency::Element(node)
                    | MutationDependency::SemanticOwner(node)
                    | MutationDependency::Tree(node) => node,
                    MutationDependency::Structure { boundary, .. } => boundary,
                    MutationDependency::Removed(_) => return,
                };
                if self.dom.is_live(node) {
                    self.insert_damage_root(&mut roots, self.formatting_damage_root(node));
                }
            });
        }
        self.last_layout_damage = Some(LayoutDamage {
            kind: LayoutDamageKind::Dom,
            roots,
            full_document: false,
        });
    }

    pub(in crate::document) fn formatting_damage_root(&self, node: D::NodeId) -> D::NodeId {
        if !self.dom.is_live(node) {
            return self.dom.document();
        }
        let Some(layout) = self.layout.as_ref() else {
            return self.dom.document();
        };
        let mut candidate = Some(node);
        while let Some(current) = candidate {
            if layout
                .fragments
                .boxes()
                .boxes_for_node(current)
                .iter()
                .any(|box_id| {
                    layout.fragments.boxes()[*box_id]
                        .formatting_context
                        .is_some()
                })
            {
                return current;
            }
            candidate = self.dom.parent(current);
        }
        self.dom.document()
    }

    pub(in crate::document) fn insert_damage_root(
        &self,
        roots: &mut Vec<D::NodeId>,
        candidate: D::NodeId,
    ) {
        if roots
            .iter()
            .copied()
            .any(|root| self.is_dom_ancestor(root, candidate))
        {
            return;
        }
        roots.retain(|root| !self.is_dom_ancestor(candidate, *root));
        roots.push(candidate);
    }

    pub(in crate::document) fn is_dom_ancestor(
        &self,
        ancestor: D::NodeId,
        node: D::NodeId,
    ) -> bool {
        let mut current = Some(node);
        while let Some(candidate) = current {
            if candidate == ancestor {
                return true;
            }
            current = self.dom.parent(candidate);
        }
        false
    }

    /// Discard visible derived output while retaining the immediately prior
    /// layout as K5g's identity source. The next frame recomputes geometry and
    /// reconciles only compatible generated boxes and fragments against it.
    pub(in crate::document) fn retain_layout_identity(&mut self) {
        self.mark_layout_dirty();
        if let Some(layout) = self.layout.take() {
            self.identity_source = Some(layout);
        }
    }

    pub(in crate::document) fn mark_layout_dirty(&mut self) {
        self.cached = None;
        self.layout_dirty = true;
    }

    pub fn frame(&mut self, width: u32, height: u32) -> Result<LiveryPaintList, LayoutError> {
        if let Some((viewport, list)) = &self.cached
            && *viewport == (width, height)
        {
            return Ok(list.clone().translated(-self.scroll.0, -self.scroll.1));
        }

        if !self.layout_dirty
            && self
                .layout
                .as_ref()
                .is_some_and(|layout| layout.viewport == (width, height))
        {
            self.generation = self.generation.saturating_add(1);
            return self.paint_active_layout(width, height);
        }

        let viewport_changed = self.viewport != (width, height);
        if viewport_changed {
            self.record_full_layout_damage(LayoutDamageKind::Viewport);
        }
        self.viewport = (width, height);
        self.device.set_viewport_size(width as f32, height as f32);
        self.finish_completed_transitions();
        let style_span = crate::phase::span(crate::phase::Phase::Style);
        self.style_session.update(
            &self.dom,
            &self.style_set,
            &self.device,
            &self.interactions,
            &[],
        );
        let mut styles = resolve_container_query_styles_with_images(
            &self.dom,
            self.style_session.styles(),
            &self.style_set,
            &self.device,
            &self.interactions,
            &crate::DocumentImageSources::new(
                &self.image_sources,
                self.style_set.document_base_url(),
            ),
        )?;
        styles.resolve_ch_lengths(&mut self.text, self.device.viewport_sizes);
        // Animation ownership must be established before any retained
        // paint-only shortcut considers the style delta. Otherwise a
        // transitioning background-color looks like a static repaint and the
        // shortcut drops the transition before its first sample.
        self.schedule_transitions(&styles);
        self.schedule_keyframe_animation(&styles);
        drop(style_span);
        // Opened here so the retained fast paths' geometry work is attributed
        // to layout as well; `take` closes it without moving the binding.
        let mut layout_span = crate::phase::span(crate::phase::Phase::Layout);
        if !viewport_changed
            && self.layout_dirty
            && self.style_only_damage
            && self.transitions.is_empty()
            && self.keyframe_animation.is_none()
            && self
                .layout
                .as_ref()
                .is_some_and(|layout| styles.differs_only_in_positioned_motion(&layout.styles))
        {
            // These nodes already establish transformed containing blocks.
            // Paint and input consume their new transforms/stacking order
            // from styles; their boxes and shaped text remain valid.
            self.layout
                .as_mut()
                .expect("checked retained layout")
                .styles = styles;
            self.identity_source = None;
            self.layout_dirty = false;
            self.generation = self.generation.saturating_add(1);
            let _ = layout_span.take();
            return self.paint_active_layout(width, height);
        }
        if !viewport_changed
            && self.layout_dirty
            && self.transitions.is_empty()
            && self.keyframe_animation.is_none()
            && let Some(node) = self
                .layout
                .as_ref()
                .and_then(|layout| styles.only_positioned_insets_changed(&layout.styles))
            && self.layout.as_mut().is_some_and(|layout| {
                layout.fragments.reposition_stable_positioned_subtree(
                    &self.dom,
                    &styles,
                    &crate::DocumentImageSources::new(
                        &self.image_sources,
                        self.style_set.document_base_url(),
                    ),
                    node,
                    width as f32,
                    height as f32,
                )
            })
        {
            let (content_width, content_height) = self
                .layout
                .as_ref()
                .map(|layout| self.document_content_extent(&styles, &layout.fragments))
                .expect("a repositioned retained layout is still live");
            let layout = self
                .layout
                .as_mut()
                .expect("a repositioned retained layout is still live");
            layout.styles = styles;
            layout.content_width = content_width;
            layout.content_height = content_height;
            self.identity_source = None;
            self.layout_dirty = false;
            self.layout_generation = self.layout_generation.saturating_add(1);
            self.clamp_scroll();
            self.clamp_nested_scroll();
            self.generation = self.generation.saturating_add(1);
            let _ = layout_span.take();
            return self.paint_active_layout(width, height);
        }
        if !viewport_changed
            && self.layout_dirty
            && self.transitions.is_empty()
            && self.keyframe_animation.is_none()
            && let Some(node) = self
                .layout
                .as_ref()
                .and_then(|layout| styles.only_positioned_leaf_geometry_changed(&layout.styles))
            && self.layout.as_mut().is_some_and(|layout| {
                layout.fragments.resize_positioned_leaf(
                    &self.dom,
                    &styles,
                    &crate::DocumentImageSources::new(
                        &self.image_sources,
                        self.style_set.document_base_url(),
                    ),
                    node,
                    width as f32,
                    height as f32,
                )
            })
        {
            let (content_width, content_height) = self
                .layout
                .as_ref()
                .map(|layout| self.document_content_extent(&styles, &layout.fragments))
                .expect("a resized retained layout is still live");
            let layout = self
                .layout
                .as_mut()
                .expect("a resized retained layout is still live");
            layout.styles = styles;
            layout.content_width = content_width;
            layout.content_height = content_height;
            self.identity_source = None;
            self.layout_dirty = false;
            self.layout_generation = self.layout_generation.saturating_add(1);
            self.clamp_scroll();
            self.clamp_nested_scroll();
            self.generation = self.generation.saturating_add(1);
            let _ = layout_span.take();
            return self.paint_active_layout(width, height);
        }
        if !viewport_changed
            && self.layout_dirty
            && self.transitions.is_empty()
            && self.keyframe_animation.is_none()
            && self
                .layout
                .as_ref()
                .is_some_and(|layout| styles.differs_only_in_background_color(&layout.styles))
        {
            self.layout
                .as_mut()
                .expect("a checked retained layout is still live")
                .styles = styles;
            self.identity_source = None;
            self.layout_dirty = false;
            self.generation = self.generation.saturating_add(1);
            let _ = layout_span.take();
            return self.paint_active_layout(width, height);
        }

        let local_root = self.last_layout_damage.as_ref().and_then(|damage| {
            (!viewport_changed
                && damage.kind == LayoutDamageKind::Dom
                && !damage.full_document
                && damage.roots.len() == 1)
                .then(|| damage.roots[0])
        });
        if self.transitions.is_empty()
            && self.keyframe_animation.is_none()
            && !self.style_set.has_container_queries()
            && let (Some(root), Some(previous)) = (local_root, self.layout.as_ref())
        {
            let dom_text_order = text_sources_in_dom_order(&self.dom);
            let mut candidate =
                retained_table_owner(previous.fragments.boxes(), root).unwrap_or(root);
            loop {
                let mut replaced_nodes = nodes_in_subtree(&self.dom, candidate);
                replaced_nodes.extend(previous.fragments.generated_subtree_nodes(candidate));
                match layout_retained_formatting_root(
                    &self.dom,
                    &styles,
                    &previous.styles,
                    &previous.fragments,
                    candidate,
                    &mut self.text,
                    &crate::DocumentImageSources::new(
                        &self.image_sources,
                        self.style_set.document_base_url(),
                    ),
                )? {
                    RetainedRootFormatting::Formatted(mut local) => {
                        local.reconcile_identifiers(&previous.fragments);
                        // Only a formatted candidate gets a copy. Failed and
                        // promoted roots leave the retained layout untouched.
                        let mut fragments = previous.fragments.clone();
                        if fragments.replace_reconciled_local_formatting_subtree_from(
                            &local,
                            candidate,
                            &replaced_nodes,
                            &dom_text_order,
                        ) {
                            let (content_width, content_height) =
                                self.document_content_extent(&styles, &fragments);
                            let layout = self
                                .layout
                                .as_mut()
                                .expect("the retained root formatter had a source layout");
                            layout.styles = styles;
                            layout.fragments = fragments;
                            layout.content_width = content_width;
                            layout.content_height = content_height;
                            self.identity_source = None;
                            self.layout_dirty = false;
                            self.layout_generation = self.layout_generation.saturating_add(1);
                            #[cfg(test)]
                            {
                                self.retained_root_relayout_generation =
                                    self.retained_root_relayout_generation.saturating_add(1);
                            }
                            self.clamp_scroll();
                            self.clamp_nested_scroll();
                            self.generation = self.generation.saturating_add(1);
                            let _ = layout_span.take();
                            return self.paint_active_layout(width, height);
                        }
                        break;
                    },
                    RetainedRootFormatting::PromoteParent => {
                        let Some(parent) = self.dom.parent(candidate) else {
                            break;
                        };
                        candidate = parent;
                    },
                    RetainedRootFormatting::Unsupported => break,
                }
            }
        }

        self.retain_layout_identity();
        self.apply_transitions(&mut styles);
        self.apply_keyframe_animation(&mut styles);
        let (styles, mut fragments) = layout_with_text_system(
            &self.dom,
            &styles,
            width as f32,
            height as f32,
            self.device.viewport_sizes,
            &mut self.text,
            &crate::DocumentImageSources::new(
                &self.image_sources,
                self.style_set.document_base_url(),
            ),
        )?;
        let previous = self.identity_source.take();
        if let Some(previous) = previous.as_ref() {
            fragments.reconcile_identifiers(&previous.fragments);
        }
        let replacement_roots = self.last_layout_damage.as_ref().and_then(|damage| {
            (!damage.full_document
                && damage.kind == LayoutDamageKind::Dom
                && !damage.roots.is_empty())
            .then(|| damage.roots.clone())
        });
        if let (Some(mut previous), Some(roots)) = (previous, replacement_roots)
            && previous
                .fragments
                .replace_reconciled_formatting_subtrees_from(&fragments, &roots)
        {
            fragments = previous.fragments;
        }
        let (content_width, content_height) = self.document_content_extent(&styles, &fragments);
        // Moved, not cloned: nothing reads these locals afterwards, and a
        // whole-document style plane and fragment tree are the two largest
        // allocations a first frame makes.
        self.layout = Some(LayoutState {
            viewport: (width, height),
            styles,
            fragments,
            content_width,
            content_height,
        });
        self.layout_dirty = false;
        self.layout_generation = self.layout_generation.saturating_add(1);
        self.clamp_scroll();
        self.clamp_nested_scroll();
        self.generation = self.generation.saturating_add(1);
        let _ = layout_span.take();
        self.paint_active_layout(width, height)
    }

    pub(in crate::document) fn paint_active_layout(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<LiveryPaintList, LayoutError> {
        let _paint_span = crate::phase::span(crate::phase::Phase::Paint);
        let layout = self
            .layout
            .as_ref()
            .ok_or_else(|| LayoutError::retained_state("no retained layout to paint"))?;
        // The retained geometry stays stable between clock ticks, while
        // paint reads a fresh style sample at the current transition or
        // keyframe time.
        let styles = self.sampled_paint_styles(layout);
        let fragments = self.sticky_layout_with_styles(layout, &styles);
        let list = emit_paint_list_with_text_system_scrolled_with_images_and_document_base(
            &self.dom,
            &styles,
            &fragments,
            DeviceIntSize::new(width as i32, height as i32),
            self.generation,
            &mut self.text,
            &self.nested_scroll,
            &self.image_sources,
            self.style_set.document_base_url(),
        );
        self.retain_active_data_urls(&styles);
        self.cached = Some(((width, height), list.clone()));
        self.style_only_damage = true;
        Ok(list.translated(-self.scroll.0, -self.scroll.1))
    }

    pub(in crate::document) fn sampled_paint_styles<'a>(
        &self,
        layout: &'a LayoutState<D::NodeId>,
    ) -> Cow<'a, StylePlane<D::NodeId>> {
        let mut styles = Cow::Borrowed(&layout.styles);
        if !self.transitions.is_empty() || self.keyframe_animation.is_some() {
            self.apply_transitions(styles.to_mut());
            self.apply_keyframe_animation(styles.to_mut());
        }
        styles
    }

    fn retain_active_data_urls(&self, styles: &StylePlane<D::NodeId>) {
        if !self.image_sources.has_data_urls() {
            return;
        }
        let mut active = styles
            .background_image_urls()
            .filter(|url| crate::image_resources::is_data_url(url))
            .map(str::to_owned)
            .collect::<HashSet<_>>();
        let mut nodes = vec![self.dom.document()];
        while let Some(node) = nodes.pop() {
            nodes.extend(self.dom.flat_children(node));
            if !self
                .dom
                .element_name(node)
                .is_some_and(|name| name.local.as_ref().eq_ignore_ascii_case("img"))
            {
                continue;
            }
            if let Some(source) = self.dom.attributes(node).find_map(|attribute| {
                (attribute.name.ns.as_ref().is_empty()
                    && attribute.name.local.as_ref().eq_ignore_ascii_case("src"))
                .then_some(attribute.value)
            }) {
                let key = crate::image_resources::document_image_key(
                    self.style_set.document_base_url(),
                    source,
                );
                if crate::image_resources::is_data_url(&key) {
                    active.insert(key);
                }
            }
        }
        self.image_sources.retain_data_urls(&active);
    }

    pub(in crate::document) fn sticky_layout<'a>(
        &self,
        layout: &'a LayoutState<D::NodeId>,
    ) -> Cow<'a, LiveryLayout<D::NodeId>> {
        self.sticky_layout_with_styles(layout, &layout.styles)
    }

    fn sticky_layout_with_styles<'a>(
        &self,
        layout: &'a LayoutState<D::NodeId>,
        styles: &StylePlane<D::NodeId>,
    ) -> Cow<'a, LiveryLayout<D::NodeId>> {
        let mut active = Cow::Borrowed(&layout.fragments);
        if layout.fragments.has_sticky_positioning() {
            self.apply_sticky_positioning(active.to_mut(), styles);
        }
        active
    }

    pub(in crate::document) fn apply_sticky_positioning(
        &self,
        fragments: &mut LiveryLayout<D::NodeId>,
        styles: &StylePlane<D::NodeId>,
    ) {
        fragments.apply_sticky_positioning(
            styles,
            self.viewport.0 as f32,
            self.viewport.1 as f32,
            |node, geometry| self.sticky_scrollport(node, styles, geometry),
        );
    }

    pub(in crate::document) fn sticky_scrollport(
        &self,
        node: D::NodeId,
        styles: &StylePlane<D::NodeId>,
        fragments: &LiveryLayout<D::NodeId>,
    ) -> Option<StickyScrollport> {
        let mut ancestor = self.dom.parent(node);
        while let Some(candidate) = ancestor {
            if styles
                .get(candidate)
                .is_some_and(|style| self.is_scroll_container(style))
                && let Some(fragment) = fragments.get(candidate)
            {
                return Some(StickyScrollport {
                    rect: fragment.physical_rect(),
                    offset: self
                        .nested_scroll
                        .get(&candidate)
                        .copied()
                        .map_or(buckram::PhysicalOffset::default(), |(x, y)| {
                            buckram::PhysicalOffset { x, y }
                        }),
                });
            }
            ancestor = self.dom.parent(candidate);
        }
        Some(StickyScrollport {
            rect: buckram::PhysicalRect {
                x: 0.0,
                y: 0.0,
                width: self.viewport.0 as f32,
                height: self.viewport.1 as f32,
            },
            offset: buckram::PhysicalOffset {
                x: self.scroll.0,
                y: self.scroll.1,
            },
        })
    }

    pub(in crate::document) fn has_sticky_positioning(&self) -> bool {
        self.layout
            .as_ref()
            .or(self.identity_source.as_ref())
            .is_some_and(|layout| layout.fragments.has_sticky_positioning())
    }
}
