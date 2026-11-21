/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! One projection pass: the walk the full projection makes, and the single
//! node and child-list projections a retained session repeats.

use std::hash::Hash;

use document_session_api::{
    DocumentA11yBounds, DocumentA11yNode, DocumentA11yNodeId, DocumentA11yRole,
};
use genet_livery::LiveryLayout;
use layout_dom_api::{LayoutDom, NodeKind};

use super::{
    AncestorScroll, aria_number, aria_true, document_role, name, neutral_actions,
    neutral_heading_level, neutral_node_id, neutral_state, text_control_value,
};
use crate::render::ScrollOffsets;

/// What one pass reads besides the node it is projecting.
pub(super) struct Pass<'a, D: LayoutDom> {
    pub(super) dom: &'a D,
    pub(super) fragments: &'a LiveryLayout<D::NodeId>,
    pub(super) scroll_offsets: Option<&'a ScrollOffsets<D::NodeId>>,
    pub(super) names: &'a name::Names<'a, D>,
    pub(super) focus: Option<D::NodeId>,
}

/// A projected node's DOM node and the scroll its ancestors apply.
pub(super) type Key<Id> = (Id, AncestorScroll);

impl<D> Pass<'_, D>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    /// Whether `node` gets its own projected node (when not under `aria-hidden`).
    /// A CSS-hidden node is skipped like a box-less one, but its children are
    /// still walked: a `visibility: visible` child of a hidden parent renders.
    pub(super) fn projected(&self, node: D::NodeId) -> bool {
        self.dom.kind(node) == NodeKind::Document
            || (self.fragments.get(node).is_some() && self.names.rendered(node))
    }

    pub(super) fn own_scroll(&self, node: D::NodeId) -> (f32, f32) {
        self.scroll_offsets
            .and_then(|offsets| offsets.get(&node))
            .copied()
            .unwrap_or_default()
    }

    /// Project `node` and its subtree, pushing nodes in post-order to `out`
    /// and each one's key to `keys`; returns the top projected ids.
    pub(super) fn walk(
        &self,
        node: D::NodeId,
        inherited: AncestorScroll,
        parent: Option<DocumentA11yNodeId>,
        out: &mut Vec<DocumentA11yNode>,
        keys: &mut Vec<Key<D::NodeId>>,
    ) -> Vec<DocumentA11yNodeId> {
        if aria_true(self.dom, node, "aria-hidden") {
            return Vec::new();
        }
        let id = self
            .projected(node)
            .then(|| neutral_node_id(self.dom, node));
        let child_scroll = inherited.including(self.own_scroll(node));
        let children = self
            .dom
            .flat_children(node)
            .filter(|child| self.dom.kind(*child) == NodeKind::Element)
            .flat_map(|child| self.walk(child, child_scroll, id.or(parent), out, keys))
            .collect::<Vec<_>>();
        let Some(id) = id else {
            return children;
        };
        out.push(self.node(node, id, inherited, parent, children));
        keys.push((node, inherited));
        vec![id]
    }

    /// The projected ids directly below `node`, through unprojected elements.
    pub(super) fn children(&self, node: D::NodeId) -> Vec<DocumentA11yNodeId> {
        let mut out = Vec::new();
        for child in self.dom.flat_children(node) {
            if self.dom.kind(child) != NodeKind::Element
                || aria_true(self.dom, child, "aria-hidden")
            {
                continue;
            }
            if self.projected(child) {
                out.push(neutral_node_id(self.dom, child));
            } else {
                out.extend(self.children(child));
            }
        }
        out
    }

    pub(super) fn bounds(
        &self,
        node: D::NodeId,
        inherited: AncestorScroll,
    ) -> Option<DocumentA11yBounds> {
        self.fragments.get(node).map(|fragment| {
            let (scroll_x, scroll_y) = inherited.offset;
            DocumentA11yBounds {
                x: fragment.x - scroll_x,
                y: fragment.y - scroll_y,
                width: fragment.width,
                height: fragment.height,
            }
        })
    }

    /// One projected node, given its place in the tree.
    pub(super) fn node(
        &self,
        node: D::NodeId,
        id: DocumentA11yNodeId,
        inherited: AncestorScroll,
        parent: Option<DocumentA11yNodeId>,
        children: Vec<DocumentA11yNodeId>,
    ) -> DocumentA11yNode {
        let dom = self.dom;
        let role = match document_role(dom, node) {
            DocumentA11yRole::Heading { level: 0 } => DocumentA11yRole::Heading {
                level: neutral_heading_level(dom, node),
            },
            role => role,
        };
        let state = neutral_state(dom, node, self.focus);
        DocumentA11yNode {
            id,
            parent,
            children,
            role,
            name: self.names.name(node),
            description: self.names.description(node),
            value: text_control_value(dom, node),
            numeric_value: aria_number(dom, node, "aria-valuenow"),
            numeric_minimum: aria_number(dom, node, "aria-valuemin"),
            numeric_maximum: aria_number(dom, node, "aria-valuemax"),
            bounds: self.bounds(node, inherited),
            state,
            actions: neutral_actions(dom, node, role, state, inherited.active),
        }
    }
}
