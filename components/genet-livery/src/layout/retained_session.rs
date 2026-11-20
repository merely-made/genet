// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use layout_dom_api::DomMutation;

impl<Id: Copy + Eq + Hash> LiveryLayout<Id> {
    /// Update retained fragments using a borrowed live DOM and its current
    /// cascade. The caller owns the DOM, styles, resources, and scroll state.
    /// Call with the complete batch since this layout and the same device,
    /// text and image resources. A refusal leaves the published layout intact.
    #[expect(
        clippy::too_many_arguments,
        reason = "borrowed layout and its caller-owned inputs"
    )]
    pub fn relayout_borrowed_dom<D: LayoutDom<NodeId = Id>>(
        &mut self,
        dom: &D,
        style_set: &StyleSet,
        cascade: &StylePlane<Id>,
        previous_styles: &StylePlane<Id>,
        viewport: ViewportSizes,
        mutations: &[DomMutation<Id>],
        text: &mut TextSystem,
        images: &dyn crate::ImageSourceLookup,
    ) -> Result<bool, LayoutError> {
        if style_set.has_container_queries()
            || self.viewport != (viewport.dynamic.width, viewport.dynamic.height)
        {
            return Ok(false);
        }
        let Some(owner) = text_mutation_owner(dom, mutations) else {
            return Ok(false);
        };
        // The complete cascade includes selectors, generated content and
        // inheritance outside the edited subtree. Keep the published plane
        // only if those results and the resolved lengths agree.
        if cascade != previous_styles {
            let mut resolved = cascade.clone();
            resolved.resolve_ch_lengths(text, viewport);
            if resolved != *previous_styles {
                return Ok(false);
            }
        }
        taffy_style::reset_calc_scratch();
        let root = formatting_damage_root(dom, self, owner);
        reformat_retained_root(
            dom,
            previous_styles,
            previous_styles,
            self,
            root,
            text,
            images,
        )
    }
}

fn text_mutation_owner<D: LayoutDom>(
    dom: &D,
    mutations: &[DomMutation<D::NodeId>],
) -> Option<D::NodeId>
where
    D::NodeId: Copy + Eq + Hash,
{
    let mut owner = None;
    let mut text = false;
    for mutation in mutations {
        let node = match mutation {
            DomMutation::CharacterDataChanged { node }
                if dom.is_live(*node)
                    && dom.kind(*node) == NodeKind::Text
                    && dom.assigned_slot(*node).is_none() =>
            {
                text = true;
                dom.parent(*node)?
            },
            DomMutation::AttributeChanged { node, name, .. }
                if name.ns == Namespace::from("") && name.local == LocalName::from("title") =>
            {
                *node
            },
            _ => return None,
        };
        if owner.is_some_and(|owner| owner != node) {
            return None;
        }
        owner = Some(node);
    }
    let owner = owner.filter(|owner| text && dom.is_live(*owner))?;
    let mut ancestor = owner;
    loop {
        // Do not infer a composed-tree formatting root through slotting.
        if dom.assigned_slot(ancestor).is_some() || dom.shadow_root(ancestor).is_some() {
            return None;
        }
        if ancestor == dom.document() {
            break;
        }
        ancestor = dom.parent(ancestor)?;
    }
    Some(owner)
}

pub(crate) fn formatting_damage_root<D>(
    dom: &D,
    previous: &LiveryLayout<D::NodeId>,
    node: D::NodeId,
) -> D::NodeId
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let boxes = previous.boxes();
    let mut candidate = Some(node);
    while let Some(current) = candidate {
        if boxes
            .boxes_for_node(current)
            .iter()
            .any(|id| boxes[*id].formatting_context.is_some())
        {
            return current;
        }
        candidate = dom.parent(current);
    }
    dom.document()
}

pub(crate) fn reformat_retained_root<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    previous_styles: &StylePlane<D::NodeId>,
    previous: &mut LiveryLayout<D::NodeId>,
    root: D::NodeId,
    text: &mut TextSystem,
    images: &dyn crate::ImageSourceLookup,
) -> Result<bool, LayoutError>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    // Promotions share the box tree for this DOM and cascade.
    let mut boxes = None;
    let mut root = retained_table_owner(previous.boxes(), root).unwrap_or(root);
    loop {
        match layout_retained_formatting_root(
            dom,
            styles,
            previous_styles,
            previous,
            &mut boxes,
            root,
            text,
            images,
        )? {
            RetainedRootFormatting::Formatted(mut local) => {
                let mut replaced = HashSet::new();
                let mut pending = vec![root];
                while let Some(node) = pending.pop() {
                    replaced.insert(node);
                    pending.extend(dom.flat_children(node));
                }
                replaced.extend(previous.generated_subtree_nodes(root));
                local.reconcile_identifiers(previous);
                let sources = text_sources_in_dom_order(dom);
                return Ok(previous.replace_reconciled_local_formatting_subtree_from(
                    &local, root, &replaced, &sources,
                ));
            },
            RetainedRootFormatting::PromoteParent => {
                let Some(parent) = dom.parent(root) else {
                    return Ok(false);
                };
                root = parent;
            },
            RetainedRootFormatting::Unsupported => return Ok(false),
        }
    }
}

fn text_sources_in_dom_order<D: LayoutDom>(dom: &D) -> Vec<D::NodeId> {
    fn visit<D: LayoutDom>(dom: &D, node: D::NodeId, sources: &mut Vec<D::NodeId>) {
        // Native controls retain text under the element's own ID.
        if dom.kind(node) == NodeKind::Text || dom.form_control_state(node).is_some() {
            sources.push(node);
        }
        for child in dom.dom_children(node) {
            visit(dom, child, sources);
        }
    }
    let mut sources = Vec::new();
    visit(dom, dom.document(), &mut sources);
    sources
}
