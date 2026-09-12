/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Storage transfer beneath DOM adoption. This module does not run the DOM
//! adoption algorithm or move a JS reflector between realms.

use std::collections::HashSet;

use crate::{NodeId, NodeKind, ScriptedDom};

/// A refused transfer leaves both stores, their queues and epochs unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubtreeTransferError {
    MissingNode,
    AttachedRoot,
    UnsupportedNode,
    AssociatedState,
    PendingSourceWork,
    DestinationCollision,
    InvalidTree,
    EpochExhausted,
}

/// How a member was reached. Children must agree with their recorded parent;
/// a shadow root and a template's contents are parentless side-map edges.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Origin {
    Child,
    ShadowRoot,
    TemplateContents,
}

impl ScriptedDom {
    /// Validate a prospective transfer without detaching anything or consuming
    /// mutation records. An attached root is accepted only when its entire parent
    /// chain is live, acyclic, and agrees with each parent's child list.
    ///
    /// This is not a reservation: callers must finish semantic removal and drain
    /// observer records before the detached transfer revalidates current state.
    /// Pending layout and observer records do not prevent this read-only check.
    ///
    /// The member set is **shadow-including** and template-including: a host
    /// carries its shadow root and that root's own nested hosts, and a
    /// `<template>` carries its contents fragment, because DOM's adopt steps
    /// move a node's shadow-including inclusive descendants and HTML's "adopt
    /// the template's contents" re-homes the fragment with the element.
    pub fn preflight_subtree_transfer_to(
        &self,
        destination: &Self,
        root: NodeId,
    ) -> Result<Vec<NodeId>, SubtreeTransferError> {
        use SubtreeTransferError as Error;
        let root_parent = self
            .nodes
            .get(&root.raw())
            .ok_or(Error::MissingNode)?
            .parent;
        if self.observed_group.is_some() {
            return Err(Error::PendingSourceWork);
        }
        // An attached root still needs a removal epoch before the transfer epoch.
        self.structure_epoch
            .checked_add(if root_parent.is_some() { 2 } else { 1 })
            .ok_or(Error::EpochExhausted)?;
        destination
            .structure_epoch
            .checked_add(1)
            .ok_or(Error::EpochExhausted)?;
        let mut ancestry = HashSet::new();
        let mut cursor = root;
        while let Some(parent) = self
            .nodes
            .get(&cursor.raw())
            .ok_or(Error::MissingNode)?
            .parent
        {
            if !ancestry.insert(cursor) {
                return Err(Error::InvalidTree);
            }
            let parent_node = self.nodes.get(&parent.raw()).ok_or(Error::MissingNode)?;
            if parent_node
                .children
                .iter()
                .filter(|&&child| child == cursor)
                .count()
                != 1
            {
                return Err(Error::InvalidTree);
            }
            cursor = parent;
        }
        // The root itself is an ordinary node: a document, a shadow root or the
        // inert template-contents owner is never the thing being adopted, only
        // something carried along beneath one.
        if matches!(
            self.nodes[&root.raw()].kind,
            NodeKind::Document | NodeKind::ShadowRoot
        ) {
            return Err(Error::UnsupportedNode);
        }
        let mut members = Vec::new();
        let mut seen = HashSet::new();
        let mut pending = vec![(root, root_parent, Origin::Child)];
        while let Some((id, parent, origin)) = pending.pop() {
            let key = id.raw();
            if !seen.insert(key) {
                return Err(Error::InvalidTree);
            }
            let node = self.nodes.get(&key).ok_or(Error::MissingNode)?;
            if node.parent != parent {
                return Err(Error::InvalidTree);
            }
            match origin {
                Origin::Child => {
                    if matches!(node.kind, NodeKind::Document | NodeKind::ShadowRoot) {
                        return Err(Error::UnsupportedNode);
                    }
                },
                Origin::ShadowRoot => {
                    if node.kind != NodeKind::ShadowRoot {
                        return Err(Error::AssociatedState);
                    }
                },
                Origin::TemplateContents => {
                    if node.kind != NodeKind::DocumentFragment {
                        return Err(Error::AssociatedState);
                    }
                },
            }
            // The inert owner document is shared by every template in the
            // source; it is re-homed to the destination's own, never moved.
            if self.template_document == Some(id) {
                return Err(Error::UnsupportedNode);
            }
            if self.parser_holds(id) {
                return Err(Error::PendingSourceWork);
            }
            if destination.nodes.contains_key(&key) {
                return Err(Error::DestinationCollision);
            }
            members.push(id);
            pending.extend(
                node.children
                    .iter()
                    .rev()
                    .map(|&child| (child, Some(id), Origin::Child)),
            );
            if let Some(&shadow) = self.shadow_hosts.get(&key) {
                pending.push((shadow, None, Origin::ShadowRoot));
            }
            if let Some(&contents) = self.template_contents.get(&key) {
                pending.push((contents, None, Origin::TemplateContents));
            }
        }
        // Every associated-state edge must be wholly inside or wholly outside
        // the member set. A shadow root whose host stayed behind, a slot
        // assignment straddling the boundary, or a contents fragment whose
        // template is not moving would arrive in the destination as a dangling
        // reference, so those are still refused.
        let inside = |id: &NodeId| seen.contains(&id.raw());
        for (key, data) in &self.shadow_roots {
            if seen.contains(key) != inside(&data.host) {
                return Err(Error::AssociatedState);
            }
            if seen.contains(key) {
                continue;
            }
            if data
                .manual
                .iter()
                .chain(data.slots.iter())
                .any(|(slot, nodes)| inside(slot) || nodes.iter().any(inside))
            {
                return Err(Error::AssociatedState);
            }
        }
        for (key, host) in &self.shadow_hosts {
            if seen.contains(key) != inside(host) {
                return Err(Error::AssociatedState);
            }
        }
        for (key, slot) in &self.assigned_slots {
            if seen.contains(key) != inside(slot) {
                return Err(Error::AssociatedState);
            }
        }
        for (key, nodes) in &self.slot_assignments {
            let slot_in = seen.contains(key);
            if nodes.iter().any(|id| inside(id) != slot_in) {
                return Err(Error::AssociatedState);
            }
        }
        for (key, contents) in &self.template_contents {
            if seen.contains(key) != inside(contents) {
                return Err(Error::AssociatedState);
            }
        }
        Ok(members)
    }

    /// Transfer after semantic removal while preserving the source's pending
    /// layout queue and its base sequence exactly. Observer records must already
    /// have been consumed; this method does not deliver or discard them.
    ///
    /// Retained layout records can name nodes that are no longer live in the
    /// source after success. Their consumer must invalidate using the recorded
    /// identities and former parents, without reading removed nodes in this store.
    pub fn transfer_detached_subtree_preserving_mutations_to(
        &mut self,
        destination: &mut Self,
        root: NodeId,
    ) -> Result<Vec<NodeId>, SubtreeTransferError> {
        let pending = std::mem::take(&mut self.mutations);
        let result = self.transfer_detached_subtree_to(destination, root);
        self.mutations = pending;
        result
    }

    /// Move a detached ordinary subtree into `destination`, preserving every
    /// node's identity and data. Return its identities in preorder so the host
    /// can relocate its ownership index and retention roots.
    ///
    /// This is a lower-level storage operation, **not** `Document.adoptNode`.
    /// The caller must coordinate wrapper identity, owning-document metadata,
    /// ranges, observer delivery, script state, layout and host pins before
    /// exposing the result or running collection. A pin retained only in this
    /// source arena cannot keep a transferred node alive in the destination.
    /// No callback runs during transfer, and every recoverable refusal happens
    /// before either store is changed.
    ///
    /// The initial boundary admits detached elements, fragments, doctypes and
    /// character data without shadow/template/slot metadata. Documents and
    /// shadow roots are refused. Source mutation queues must have been consumed
    /// by their owners, and parsing or an observer group must not be active. Insertion
    /// is a separate operation through the destination's ordinary mutation
    /// boundary; this function emits no invented DOM mutation record.
    /// Imported IDs retain their birth namespace, so destination capture goes
    /// through the identity pair (`try_capture_node_identity`), which names the
    /// origin arena this call registers on the destination.
    pub fn transfer_detached_subtree_to(
        &mut self,
        destination: &mut Self,
        root: NodeId,
    ) -> Result<Vec<NodeId>, SubtreeTransferError> {
        use SubtreeTransferError as Error;

        let root_node = self.nodes.get(&root.raw()).ok_or(Error::MissingNode)?;
        if root_node.parent.is_some() {
            return Err(Error::AttachedRoot);
        }
        if self.observed_group.is_some() || !self.observed.is_empty() || !self.mutations.is_empty()
        {
            return Err(Error::PendingSourceWork);
        }
        let source_epoch = self
            .structure_epoch
            .checked_add(1)
            .ok_or(Error::EpochExhausted)?;
        let destination_epoch = destination
            .structure_epoch
            .checked_add(1)
            .ok_or(Error::EpochExhausted)?;
        let members = self.preflight_subtree_transfer_to(destination, root)?;

        // Reserve before removing source data. Normal allocation failure has
        // Rust's usual process-level behavior; it cannot leave a recoverable
        // partially transferred result.
        destination.nodes.reserve(members.len());
        for id in &members {
            let node = self
                .nodes
                .remove(&id.raw())
                .expect("preflight checked every member");
            destination.nodes.insert(id.raw(), node);
        }
        self.move_associated_state_to(destination, &members);
        // Register every foreign origin the destination now holds, so capture
        // and replay can translate an imported identity instead of refusing it.
        for id in &members {
            if id.origin_arena_id() != destination.arena_id() {
                destination.record_imported_arena(id.origin_arena_id());
            }
        }
        self.structure_epoch = source_epoch;
        destination.structure_epoch = destination_epoch;
        Ok(members)
    }
}

impl ScriptedDom {
    /// Relocate the side maps that hang off the transferred members: shadow
    /// roots and their slot assignment tables, and template contents together
    /// with their owner edge, which is re-pointed at the **destination's** inert
    /// template-contents owner document (HTML's "adopt the template's
    /// contents"). The nodes themselves have already moved, and the preflight
    /// has already established that no edge straddles the boundary.
    fn move_associated_state_to(&mut self, destination: &mut Self, members: &[NodeId]) {
        let moving: HashSet<u64> = members.iter().map(|id| id.raw()).collect();
        let mut owner = None;
        for id in members {
            let key = id.raw();
            if let Some(data) = self.shadow_roots.remove(&key) {
                destination.shadow_roots.insert(key, data);
            }
            if let Some(root) = self.shadow_hosts.remove(&key) {
                destination.shadow_hosts.insert(key, root);
            }
            if let Some(slot) = self.assigned_slots.remove(&key) {
                destination.assigned_slots.insert(key, slot);
            }
            if let Some(nodes) = self.slot_assignments.remove(&key) {
                destination.slot_assignments.insert(key, nodes);
            }
            if let Some(contents) = self.template_contents.remove(&key) {
                destination.template_contents.insert(key, contents);
            }
            if self.template_content_owners.remove(&key).is_some() {
                let document = *owner.get_or_insert_with(|| destination.template_owner_document());
                destination.template_content_owners.insert(key, document);
            }
        }
        // A pending `slotchange` belongs to the slot, so it is delivered by
        // whichever document the slot now lives in.
        let (moved, kept): (Vec<NodeId>, Vec<NodeId>) = self
            .slot_changes
            .drain(..)
            .partition(|id| moving.contains(&id.raw()));
        self.slot_changes = kept;
        destination.slot_changes.extend(moved);
        // Assignment is not recomputed: no member changed parent in the move,
        // so the tables that came across are already the destination's answer.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{LayoutDom, LayoutDomMut};

    #[test]
    fn preflight_checks_parent_consistency_and_reserves_removal_epoch() {
        let mut source = ScriptedDom::new();
        let target = ScriptedDom::new();
        let node = source.create_text("attached");
        let parent = source.document();
        source.append_child(parent, node);
        source
            .nodes
            .get_mut(&parent.raw())
            .unwrap()
            .children
            .push(node);
        assert_eq!(
            source.preflight_subtree_transfer_to(&target, node),
            Err(SubtreeTransferError::InvalidTree)
        );
        source.nodes.get_mut(&parent.raw()).unwrap().children.pop();
        source.structure_epoch = u64::MAX - 1;
        assert_eq!(
            source.preflight_subtree_transfer_to(&target, node),
            Err(SubtreeTransferError::EpochExhausted)
        );
        assert_eq!(source.parent(node), Some(parent));
        assert_eq!(source.structure_epoch, u64::MAX - 1);
    }
}
