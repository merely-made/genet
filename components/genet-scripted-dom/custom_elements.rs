// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custom elements at the arena's mutation point.
//!
//! HTML gives every element a custom element state. It lives here, on the node,
//! so selector matching and the Forms model can read it without asking script.
//! While any definition exists, the tree and attribute mutators also append
//! [`ReactionCandidate`]s: the connected, disconnected and attribute-changed
//! facts the bootstrap's custom element reactions stack turns into callbacks.
//! Every caller that mutates the arena — script, parser, native host edits —
//! produces them, because they are written by the same statements that write the
//! tree. Adoption is the one exception: the owning document is not an arena
//! relation yet, so the bootstrap's adoption step records `adopted` through
//! [`ScriptedDom::record_adopted`].

use crate::{NodeId, NodeKind, QualName, ScriptedDom};

/// HTML's custom element state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CustomElementState {
    Undefined,
    Failed,
    #[default]
    Uncustomized,
    Precustomized,
    Custom,
}

impl CustomElementState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Failed => "failed",
            Self::Uncustomized => "uncustomized",
            Self::Precustomized => "precustomized",
            Self::Custom => "custom",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "undefined" => Self::Undefined,
            "failed" => Self::Failed,
            "uncustomized" => Self::Uncustomized,
            "precustomized" => Self::Precustomized,
            "custom" => Self::Custom,
            _ => return None,
        })
    }
}

/// A fact the custom element reactions stack may owe a callback for. Only
/// elements whose state is `custom` produce one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReactionCandidate {
    Connected(NodeId),
    Disconnected(NodeId),
    Adopted {
        node: NodeId,
        old_document: NodeId,
        new_document: NodeId,
    },
    AttributeChanged {
        node: NodeId,
        name: QualName,
        old_value: Option<String>,
        new_value: Option<String>,
    },
}

impl ReactionCandidate {
    pub fn node(&self) -> NodeId {
        match self {
            Self::Connected(node) | Self::Disconnected(node) => *node,
            Self::Adopted { node, .. } | Self::AttributeChanged { node, .. } => *node,
        }
    }
}

#[derive(Clone, Copy)]
enum TreeFact {
    Connected,
    Disconnected,
}

impl ScriptedDom {
    pub fn custom_element_state(&self, id: NodeId) -> CustomElementState {
        self.nodes
            .get(&id.raw())
            .map(|node| node.custom_state)
            .unwrap_or_default()
    }

    /// Set `id`'s custom element state. False for a node this arena does not
    /// hold or that is not an element.
    pub fn set_custom_element_state(&mut self, id: NodeId, state: CustomElementState) -> bool {
        match self.nodes.get_mut(&id.raw()) {
            Some(node) if node.kind == NodeKind::Element => {
                node.custom_state = state;
                true
            },
            _ => false,
        }
    }

    /// Start or stop recording reaction candidates. The bootstrap turns this on
    /// with the first definition, so a document without custom elements keeps
    /// its old cost exactly.
    pub fn set_reaction_recording(&mut self, on: bool) {
        self.reaction_recording = on;
        if !on {
            self.reaction_candidates.clear();
        }
    }

    pub fn is_recording_reactions(&self) -> bool {
        self.reaction_recording
    }

    pub fn has_reaction_candidates(&self) -> bool {
        !self.reaction_candidates.is_empty()
    }

    /// Drain the pending candidates in the order they were recorded.
    pub fn take_reaction_candidates(&mut self) -> Vec<ReactionCandidate> {
        std::mem::take(&mut self.reaction_candidates)
    }

    /// DOM's *connected*: the shadow-including root is a document.
    pub fn is_connected(&self, id: NodeId) -> bool {
        let mut cursor = id;
        loop {
            let Some(node) = self.nodes.get(&cursor.raw()) else {
                return false;
            };
            match node.parent {
                Some(parent) => cursor = parent,
                None => match self.shadow_host_of(cursor) {
                    Some(host) => cursor = host,
                    None => return node.kind == NodeKind::Document,
                },
            }
        }
    }

    /// Record `adopted` for every custom element in `root`'s shadow-including
    /// inclusive descendants. The bootstrap's adoption step calls this: it is
    /// the one holder of the owning-document relation today.
    pub fn record_adopted(&mut self, root: NodeId, old_document: NodeId, new_document: NodeId) {
        if !self.reaction_recording || old_document == new_document {
            return;
        }
        for node in self.custom_inclusive_descendants(root) {
            self.reaction_candidates.push(ReactionCandidate::Adopted {
                node,
                old_document,
                new_document,
            });
        }
    }

    pub(crate) fn record_connected(&mut self, root: NodeId) {
        self.record_tree_fact(root, TreeFact::Connected);
    }

    pub(crate) fn record_disconnected(&mut self, root: NodeId) {
        self.record_tree_fact(root, TreeFact::Disconnected);
    }

    fn record_tree_fact(&mut self, root: NodeId, fact: TreeFact) {
        if !self.reaction_recording {
            return;
        }
        for node in self.custom_inclusive_descendants(root) {
            self.reaction_candidates.push(match fact {
                TreeFact::Connected => ReactionCandidate::Connected(node),
                TreeFact::Disconnected => ReactionCandidate::Disconnected(node),
            });
        }
    }

    pub(crate) fn record_attribute_reaction(
        &mut self,
        node: NodeId,
        name: &QualName,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        if !self.reaction_recording || self.custom_element_state(node) != CustomElementState::Custom {
            return;
        }
        self.reaction_candidates.push(ReactionCandidate::AttributeChanged {
            node,
            name: name.clone(),
            old_value,
            new_value,
        });
    }

    /// Custom elements among `root`'s shadow-including inclusive descendants, in
    /// shadow-including tree order: a host's shadow tree before its children.
    fn custom_inclusive_descendants(&self, root: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(&id.raw()) else {
                continue;
            };
            if node.custom_state == CustomElementState::Custom {
                out.push(id);
            }
            stack.extend(node.children.iter().rev().copied());
            if let Some(shadow) = self.shadow_root_of(id) {
                stack.push(shadow);
            }
        }
        out
    }

    /// Hand the candidates naming `members` to `destination` with the nodes.
    pub(crate) fn move_reaction_candidates_to(
        &mut self,
        destination: &mut Self,
        members: &std::collections::HashSet<u64>,
    ) {
        if self.reaction_candidates.is_empty() {
            return;
        }
        let (moved, kept): (Vec<_>, Vec<_>) = self
            .reaction_candidates
            .drain(..)
            .partition(|c| members.contains(&c.node().raw()));
        self.reaction_candidates = kept;
        destination.reaction_candidates.extend(moved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{
        LayoutDom, LayoutDomMut, ShadowRootInit, ShadowRootMode, SlotAssignmentMode,
    };
    use markup5ever::{LocalName, Namespace};

    fn qual(local: &str) -> QualName {
        QualName::new(None, Namespace::from(""), LocalName::from(local))
    }

    fn custom(dom: &mut ScriptedDom, local: &str) -> NodeId {
        let id = dom.create_element(qual(local));
        assert!(dom.set_custom_element_state(id, CustomElementState::Custom));
        id
    }

    fn open_root(dom: &mut ScriptedDom, host: NodeId) -> NodeId {
        dom.attach_shadow_unchecked(
            host,
            ShadowRootInit {
                mode: ShadowRootMode::Open,
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: SlotAssignmentMode::Named,
            },
            false,
        )
    }

    #[test]
    fn nothing_is_recorded_until_a_definition_turns_recording_on() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let el = custom(&mut dom, "x-a");
        dom.append_child(root, el);
        dom.set_attribute(el, qual("id"), "one");
        assert!(!dom.has_reaction_candidates());
    }

    #[test]
    fn a_move_between_connected_parents_is_disconnected_then_connected() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let root = dom.document();
        let first = dom.create_element(qual("div"));
        let second = dom.create_element(qual("div"));
        dom.append_child(root, first);
        dom.append_child(root, second);
        let el = custom(&mut dom, "x-a");
        dom.append_child(first, el);
        dom.insert_before(second, el, None);
        assert_eq!(
            dom.take_reaction_candidates(),
            vec![
                ReactionCandidate::Connected(el),
                ReactionCandidate::Disconnected(el),
                ReactionCandidate::Connected(el),
            ]
        );
    }

    #[test]
    fn detached_trees_record_no_connection() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let parent = dom.create_element(qual("div"));
        let el = custom(&mut dom, "x-a");
        dom.append_child(parent, el);
        dom.remove_child(el);
        assert!(dom.take_reaction_candidates().is_empty());
    }

    #[test]
    fn connection_walks_shadow_including_tree_order() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let root = dom.document();
        let host = custom(&mut dom, "x-host");
        let shadow = open_root(&mut dom, host);
        let inner = custom(&mut dom, "x-inner");
        dom.append_child(shadow, inner);
        let child = custom(&mut dom, "x-child");
        dom.append_child(host, child);
        assert!(dom.take_reaction_candidates().is_empty());
        dom.append_child(root, host);
        assert_eq!(
            dom.take_reaction_candidates(),
            vec![
                ReactionCandidate::Connected(host),
                ReactionCandidate::Connected(inner),
                ReactionCandidate::Connected(child),
            ]
        );
    }

    #[test]
    fn attribute_changes_carry_old_and_new_values_for_custom_elements_only() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let plain = dom.create_element(qual("div"));
        dom.set_attribute(plain, qual("id"), "x");
        let el = custom(&mut dom, "x-a");
        dom.set_attribute(el, qual("id"), "one");
        dom.set_attribute(el, qual("id"), "one");
        dom.remove_attribute(el, qual("id"));
        let changed = |old: Option<&str>, new: Option<&str>| ReactionCandidate::AttributeChanged {
            node: el,
            name: qual("id"),
            old_value: old.map(str::to_owned),
            new_value: new.map(str::to_owned),
        };
        assert_eq!(
            dom.take_reaction_candidates(),
            vec![
                changed(None, Some("one")),
                changed(Some("one"), Some("one")),
                changed(Some("one"), None),
            ]
        );
    }

    /// G5 lifetime: a removal that leaves a disconnectedCallback pending must
    /// not free the element, and collection must spare it until the candidate
    /// is drained.
    #[test]
    fn a_pending_candidate_keeps_its_removed_element_alive() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let root = dom.document();
        let el = custom(&mut dom, "x-a");
        dom.append_child(root, el);
        dom.take_reaction_candidates();
        LayoutDomMut::remove(&mut dom, el);
        assert!(dom.is_live(el), "removal freed a node a reaction still names");
        dom.collect(std::iter::empty());
        assert!(dom.is_live(el), "collection freed a node a reaction still names");
        assert_eq!(
            dom.take_reaction_candidates(),
            vec![ReactionCandidate::Disconnected(el)]
        );
        dom.collect(std::iter::empty());
        assert!(!dom.is_live(el));
    }

    #[test]
    fn replacing_children_records_disconnection_of_the_old_ones() {
        let mut dom = ScriptedDom::new();
        dom.set_reaction_recording(true);
        let root = dom.document();
        let parent = dom.create_element(qual("div"));
        dom.append_child(root, parent);
        let el = custom(&mut dom, "x-a");
        dom.append_child(parent, el);
        dom.take_reaction_candidates();
        dom.set_inner_html(parent, "<p>new</p>");
        assert_eq!(
            dom.take_reaction_candidates(),
            vec![ReactionCandidate::Disconnected(el)]
        );
        assert!(dom.is_live(el));
    }
}
