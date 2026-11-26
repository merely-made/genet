// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A scroll batch's box tree by splice: a copy of the kept tree with the
//! entering roots' boxes generated in place and the leaving roots' removed.
//! Kept elements keep their boxes because the style patch admits only paint
//! and inset changes on them, which box generation does not read. Buckram
//! refuses any insertion it cannot prove equal to a regeneration.

use buckram::{ChildSpliceRefusal, CssBoxTree, PseudoElement};

use super::scroll_batch::ScrollBatch;
use super::scroll_batch_admit::box_subtree;
use super::*;

/// Why a batch's box tree was regenerated rather than spliced.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BoxSpliceRefusal {
    /// The parent has no principal box in the kept tree.
    ParentBox,
    /// A leaving root's boxes are not one subtree directly under the parent.
    LeavingRoot,
    /// An entering root is not an element child generating one input.
    EnteringRoot,
    /// A box under the parent is not the parent's own child or pseudo.
    ForeignChild,
    Buckram(ChildSpliceRefusal),
}

impl<Id: Copy + Eq + Hash> LiveryLayout<Id> {
    /// The batch's final box tree spliced from the kept one, or why not.
    /// Nothing retained changes either way.
    pub(in crate::layout) fn splice_scroll_batch_boxes<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        styles: &StylePlane<Id>,
        batch: &ScrollBatch<Id>,
        parent: Id,
    ) -> Result<GeneratedBoxTree<Id>, BoxSpliceRefusal> {
        let previous = self.buckram.boxes();
        let parent_box = previous
            .principal_box(parent)
            .ok_or(BoxSpliceRefusal::ParentBox)?;
        let mut leaving = HashSet::new();
        for root in &batch.removed {
            let root_box = previous
                .principal_box(*root)
                .filter(|root_box| previous[*root_box].parent() == Some(parent_box))
                .ok_or(BoxSpliceRefusal::LeavingRoot)?;
            let subtree = box_subtree(previous, root_box);
            if !previous
                .boxes_for_node(*root)
                .iter()
                .all(|id| subtree.contains(id))
            {
                return Err(BoxSpliceRefusal::LeavingRoot);
            }
            leaving.insert(root_box);
        }
        let order = dom
            .flat_children(parent)
            .enumerate()
            .map(|(index, child)| (child, index))
            .collect::<HashMap<_, _>>();
        let mut tree = previous.clone();
        // Entering first, so fresh ids start above the leaving ones as a
        // regeneration's reconcile allocates them.
        for root in &batch.inserted {
            let position = order
                .get(root)
                .copied()
                .filter(|_| dom.kind(*root) == NodeKind::Element)
                .ok_or(BoxSpliceRefusal::EnteringRoot)?;
            let index = insertion_index(&tree, parent_box, parent, &order, &leaving, position)?;
            let input = GeneratedBoxTree::child_input(dom, styles, *root, parent)
                .ok_or(BoxSpliceRefusal::EnteringRoot)?;
            tree.insert_generated_child(parent_box, index, input)
                .map_err(BoxSpliceRefusal::Buckram)?;
        }
        for root_box in leaving {
            tree.remove_subtree(root_box);
        }
        #[cfg(debug_assertions)]
        assert_regenerates(dom, styles, previous, &tree);
        Ok(GeneratedBoxTree::from_tree(tree))
    }
}

/// Where a root at DOM `position` among `parent`'s children goes among its
/// parent box's children: after `::before` and every earlier kept child.
fn insertion_index<Id: Copy + Eq + Hash>(
    tree: &CssBoxTree<Id>,
    parent_box: BoxId,
    parent: Id,
    order: &HashMap<Id, usize>,
    leaving: &HashSet<BoxId>,
    position: usize,
) -> Result<usize, BoxSpliceRefusal> {
    let mut index = 0;
    for (slot, child) in tree[parent_box].children().iter().enumerate() {
        if leaving.contains(child) {
            continue;
        }
        let precedes = match tree[*child].origin {
            BoxOrigin::Pseudo { owner, pseudo } if owner == parent => {
                pseudo == PseudoElement::Before
            },
            BoxOrigin::Element(node) => {
                *order.get(&node).ok_or(BoxSpliceRefusal::ForeignChild)? < position
            },
            _ => return Err(BoxSpliceRefusal::ForeignChild),
        };
        if precedes {
            index = slot + 1;
        }
    }
    Ok(index)
}

/// Debug builds check every splice against a full regeneration.
#[cfg(debug_assertions)]
fn assert_regenerates<D: LayoutDom>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    previous: &CssBoxTree<D::NodeId>,
    spliced: &CssBoxTree<D::NodeId>,
) where
    D::NodeId: Copy + Eq + Hash,
{
    let mut regenerated = GeneratedBoxTree::from_dom(dom, styles);
    regenerated.reconcile_identifiers(previous);
    assert!(
        same_box_tree(spliced, &regenerated),
        "a spliced box tree equals its regeneration"
    );
}

/// Equal boxes under equal ids in equal storage order, with equal
/// provenance.
pub(crate) fn same_box_tree<Id: Copy + Eq + Hash>(
    left: &CssBoxTree<Id>,
    right: &CssBoxTree<Id>,
) -> bool {
    left.roots() == right.roots()
        && left.iter().eq(right.iter())
        && left
            .iter()
            .filter_map(|(_, css_box)| css_box.origin.node())
            .all(|node| {
                left.principal_box(node) == right.principal_box(node)
                    && left.boxes_for_node(node) == right.boxes_for_node(node)
            })
}

#[cfg(test)]
thread_local! {
    static OUTCOMES: std::cell::RefCell<Vec<Result<(), BoxSpliceRefusal>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Count one batch's box-tree outcome, for tests.
pub(in crate::layout) fn note_box_splice(_outcome: Result<(), BoxSpliceRefusal>) {
    #[cfg(test)]
    OUTCOMES.with(|outcomes| outcomes.borrow_mut().push(_outcome));
}

/// The box-tree outcomes since the last call: `Ok` spliced, `Err` why not.
#[cfg(test)]
pub(crate) fn take_box_splices() -> Vec<Result<(), BoxSpliceRefusal>> {
    OUTCOMES.with(|outcomes| std::mem::take(&mut *outcomes.borrow_mut()))
}
