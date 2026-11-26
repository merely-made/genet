// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained edits to a generated box tree: removing a subtree, and
//! generating one child in place where the surrounding fixups provably
//! ignore it, so the result equals regenerating the whole tree.

use super::*;

/// Why [`CssBoxTree::insert_generated_child`] refused; the caller
/// regenerates the tree instead.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChildSpliceRefusal {
    /// The parent is not a block-level flow container.
    Parent,
    /// The index is past the parent's children.
    Index,
    /// A child of the parent is in flow, so its fixups read the child list.
    InFlowSibling,
    /// The input does not generate exactly one out-of-flow element box.
    NotOneOutOfFlowBox,
    /// An ancestor is an in-flow inline or sits among table parts, where a
    /// fixup reads its descendants' white space.
    AncestorContext,
    /// A node of the input already has boxes in the tree.
    NodeHasBoxes,
}

impl<Id> CssBoxTree<Id>
where
    Id: Copy + Eq + Hash,
{
    /// Remove `root` and every box under it, keeping the others in storage
    /// order. Containing blocks are ancestors, so no remaining box refers
    /// into the removed subtree. Returns false when `root` is not in the tree.
    pub fn remove_subtree(&mut self, root: BoxId) -> bool {
        if !self.slots.contains_key(&root) {
            return false;
        }
        match self[root].parent {
            Some(parent) => self[parent].children.retain(|child| *child != root),
            None => self.roots.retain(|id| *id != root),
        }
        let mut doomed = HashSet::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            pending.extend_from_slice(&self[id].children);
            if let Some(node) = self[id].origin.node() {
                if let Some(ids) = self.boxes_by_node.get_mut(&node) {
                    ids.retain(|other| *other != id);
                    if ids.is_empty() {
                        self.boxes_by_node.remove(&node);
                    }
                }
                if self.principal_boxes.get(&node) == Some(&id) {
                    self.principal_boxes.remove(&node);
                }
            }
            doomed.insert(id);
        }
        let ids = std::mem::take(&mut self.ids);
        let mut kept = ids.iter().map(|id| !doomed.contains(id));
        self.boxes.retain(|_| kept.next().unwrap_or(false));
        self.ids = ids.into_iter().filter(|id| !doomed.contains(id)).collect();
        self.renumber_slots_from(0);
        #[cfg(any(debug_assertions, test))]
        self.assert_invariants();
        true
    }

    /// Generate `input` as the `index`th child of `parent`, exactly as
    /// [`generate_box_tree`] would among the parent's other children, with
    /// fresh ids above every id in the tree, in tree order, stored where
    /// generation would store them.
    ///
    /// Admitted only where no fixup reads the change: the parent is a
    /// block-level flow container whose children are all out of flow, the
    /// input generates one out-of-flow element box, and no ancestor's
    /// white-space test can see the parent. Anything else is refused with
    /// the tree unchanged.
    pub fn insert_generated_child(
        &mut self,
        parent: BoxId,
        index: usize,
        input: BoxTreeInput<Id>,
    ) -> Result<BoxId, ChildSpliceRefusal> {
        let Some(parent_box) = self.slots.contains_key(&parent).then(|| &self[parent]) else {
            return Err(ChildSpliceRefusal::Parent);
        };
        // Such a parent's flow fixup keeps its children as they are, and an
        // out-of-flow child never makes it a block formatting context.
        let display = parent_box.display;
        if display.generation != BoxGeneration::Normal
            || display.outside != Some(DisplayOutside::Block)
            || !matches!(
                display.inside,
                Some(DisplayInside::Flow | DisplayInside::FlowRoot)
            )
            || display.internal_table.is_some()
            || display.list_item
            || parent_box.replaced
        {
            return Err(ChildSpliceRefusal::Parent);
        }
        if index > parent_box.children.len() {
            return Err(ChildSpliceRefusal::Index);
        }
        if !parent_box
            .children
            .iter()
            .all(|child| is_out_of_flow(self[*child].positioning))
        {
            return Err(ChildSpliceRefusal::InFlowSibling);
        }
        self.check_ancestor_context(parent)?;
        // A block parent gives its children a block float context and, not
        // being flex or grid, does not blockify them as items.
        let mut protos = normalize_input(input, false, FloatContextProvenance::Block);
        let proto = match protos.pop() {
            Some(proto)
                if protos.is_empty()
                    && matches!(proto.origin, BoxOrigin::Element(_))
                    && is_out_of_flow(proto.positioning) =>
            {
                proto
            },
            _ => return Err(ChildSpliceRefusal::NotOneOutOfFlowBox),
        };
        if self.proto_has_boxes(&proto) {
            return Err(ChildSpliceRefusal::NodeHasBoxes);
        }
        let containing_blocks = self.containing_blocks_for_children(parent);
        let mut next = Some(self.ids.iter().map(|id| id.0).max().map_or(0, |id| {
            id.checked_add(1)
                .expect("a CSS box tree exceeded u32::MAX boxes")
        }));
        // Generation stores boxes in tree order: the subtree goes after its
        // previous sibling's last descendant, or straight after the parent.
        let at = match index.checked_sub(1) {
            Some(previous) => self.last_slot_under(self[parent].children[previous]) + 1,
            None => self.slots[&parent] + 1,
        };
        let appended = self.boxes.len();
        let id = materialize(self, proto, Some(parent), containing_blocks, &mut next);
        let children = &mut self[parent].children;
        let last = children.pop();
        debug_assert_eq!(last, Some(id));
        children.insert(index, id);
        let count = self.boxes.len() - appended;
        self.boxes[at..].rotate_right(count);
        self.ids[at..].rotate_right(count);
        self.renumber_slots_from(at);
        #[cfg(any(debug_assertions, test))]
        self.assert_invariants();
        Ok(id)
    }

    /// An inline ancestor joins its parent's inline run, whose white-space
    /// test reads every descendant; table parts discard white-space
    /// siblings. Either could see a change in the parent's children.
    fn check_ancestor_context(&self, parent: BoxId) -> Result<(), ChildSpliceRefusal> {
        let mut current = parent;
        loop {
            let css_box = &self[current];
            if css_box.display.outside == Some(DisplayOutside::Inline)
                && css_box.positioning.is_in_flow()
                && css_box.float == FloatSide::None
            {
                return Err(ChildSpliceRefusal::AncestorContext);
            }
            let Some(up) = css_box.parent else {
                return Ok(());
            };
            let up_box = &self[up];
            if parent_accepts_table_parts(up_box.display)
                || up_box.display.internal_table.is_some()
                || up_box
                    .children
                    .iter()
                    .any(|child| self[*child].display.internal_table.is_some())
            {
                return Err(ChildSpliceRefusal::AncestorContext);
            }
            current = up;
        }
    }

    /// The storage slot of the last box in `root`'s subtree.
    fn last_slot_under(&self, root: BoxId) -> usize {
        let mut last = root;
        while let Some(child) = self[last].children.last() {
            last = *child;
        }
        self.slots[&last]
    }

    /// Point `slots` at the ids from storage slot `from` on.
    fn renumber_slots_from(&mut self, from: usize) {
        if from == 0 {
            self.slots.clear();
        }
        for (slot, id) in self.ids.iter().copied().enumerate().skip(from) {
            self.slots.insert(id, slot);
        }
    }

    fn proto_has_boxes(&self, proto: &ProtoBox<Id>) -> bool {
        proto
            .origin
            .node()
            .is_some_and(|node| self.boxes_by_node.contains_key(&node))
            || proto
                .children
                .iter()
                .any(|child| self.proto_has_boxes(child))
    }

    /// What `materialize` hands `parent`'s children: the nearest
    /// ancestor-or-self establishing each containing block.
    fn containing_blocks_for_children(&self, parent: BoxId) -> ContainingBlockState {
        let mut state = ContainingBlockState::INITIAL;
        let mut current = Some(parent);
        while let Some(id) = current {
            let css_box = &self[id];
            let non_atomic_inline = css_box.display.outside == Some(DisplayOutside::Inline)
                && css_box.display.inside == Some(DisplayInside::Flow)
                && css_box.display.internal_table.is_none()
                && !css_box.replaced;
            if state.normal_flow.is_none() && !non_atomic_inline {
                state.normal_flow = Some(id);
            }
            let establishment = css_box.containing_block_establishment;
            if state.absolute.is_none() && establishment.absolute {
                state.absolute = Some(id);
            }
            if state.fixed.is_none() && establishment.fixed {
                state.fixed = Some(id);
            }
            current = css_box.parent;
        }
        state
    }
}

fn is_out_of_flow(positioning: PositioningScheme) -> bool {
    matches!(
        positioning,
        PositioningScheme::Absolute | PositioningScheme::Fixed
    )
}

#[cfg(test)]
mod tests;
