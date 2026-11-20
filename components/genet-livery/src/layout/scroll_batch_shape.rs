// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading a mutation batch as a scroll batch: which subtrees entered and
//! left which parent, and whether every other record leaves layout to the
//! style planes.

use layout_dom_api::DomMutation;

use super::scroll_batch::ScrollBatch;
use super::*;

/// The scroll-batch reading of `mutations`, or `None` for any other shape.
/// Records inside an entering or leaving subtree are absorbed by it. Outside
/// them only attribute writes are admitted, and only on elements whose layout
/// reads no attribute directly; their style effect is judged from the planes.
pub(in crate::layout) fn scroll_batch_shape<D: LayoutDom>(
    dom: &D,
    previous: &LiveryLayout<D::NodeId>,
    mutations: &[DomMutation<D::NodeId>],
) -> Option<ScrollBatch<D::NodeId>>
where
    D::NodeId: Copy + Eq + Hash,
{
    let mut inserted_all = HashSet::new();
    let mut removed_all = HashSet::new();
    for mutation in mutations {
        match mutation {
            DomMutation::Inserted { node, .. } => {
                inserted_all.insert(*node);
            },
            DomMutation::Removed { node, .. } => {
                removed_all.insert(*node);
            },
            _ => {},
        }
    }
    if !inserted_all.is_disjoint(&removed_all) {
        return None;
    }
    let under = |set: &HashSet<D::NodeId>, node: D::NodeId| {
        // A released node has no parent to ask.
        std::iter::successors(Some(node), |node| {
            dom.is_live(*node).then(|| dom.parent(*node)).flatten()
        })
        .any(|node| set.contains(&node))
    };

    let mut parent = None;
    let mut same_parent = |candidate: D::NodeId| *parent.get_or_insert(candidate) == candidate;
    let mut inserted = Vec::new();
    let mut removed = Vec::new();
    for mutation in mutations {
        match mutation {
            DomMutation::Inserted { node, parent: to } => {
                if !dom.is_live(*node) || dom.parent(*node) != Some(*to) {
                    return None;
                }
                if !under(&inserted_all, *to) {
                    if !same_parent(*to) {
                        return None;
                    }
                    inserted.push(*node);
                }
            },
            DomMutation::Removed {
                node,
                former_parent,
            } => {
                if dom.is_live(*node) && dom.parent(*node).is_some() {
                    return None;
                }
                if !removed_all.contains(former_parent) {
                    if !same_parent(*former_parent) {
                        return None;
                    }
                    removed.push(*node);
                }
            },
            DomMutation::CharacterDataChanged { node } => {
                if !under(&inserted_all, *node) && !under(&removed_all, *node) {
                    return None;
                }
            },
            DomMutation::AttributeChanged { node, .. } => {
                if !under(&inserted_all, *node)
                    && !under(&removed_all, *node)
                    && (!dom.is_live(*node) || reads_attributes_in_layout(dom, *node))
                {
                    return None;
                }
            },
            _ => return None,
        }
    }

    if let Some(parent) = parent
        && (!dom.is_live(parent)
            || dom.kind(parent) != NodeKind::Element
            || under(&inserted_all, parent)
            || under(&removed_all, parent)
            || composed_through_slots(dom, parent))
    {
        return None;
    }
    // Document order, so each root is inserted after its previous sibling.
    inserted.sort_by_key(|node| {
        parent.and_then(|parent| dom.dom_children(parent).position(|child| child == *node))
    });
    let mut entered = HashSet::new();
    for root in &inserted {
        let mut pending = vec![*root];
        while let Some(node) = pending.pop() {
            entered.insert(node);
            pending.extend(dom.dom_children(node));
        }
    }
    let mut left = HashSet::new();
    for root in &removed {
        left.extend(previous.generated_subtree_nodes(*root));
        left.insert(*root);
        if dom.is_live(*root) {
            let mut pending = vec![*root];
            while let Some(node) = pending.pop() {
                left.insert(node);
                pending.extend(dom.dom_children(node));
            }
        }
    }
    Some(ScrollBatch {
        parent,
        inserted,
        removed,
        entered,
        left,
    })
}

/// Elements whose layout reads attributes besides through the cascade:
/// replaced content, form controls, tables, lists and slots.
fn reads_attributes_in_layout<D: LayoutDom>(dom: &D, node: D::NodeId) -> bool
where
    D::NodeId: Copy + Eq + Hash,
{
    if dom.kind(node) != NodeKind::Element {
        return true;
    }
    if dom.form_control_state(node).is_some() || crate::box_tree::is_replaced_element(dom, node) {
        return true;
    }
    dom.element_name(node).is_none_or(|name| {
        [
            "img", "picture", "source", "video", "audio", "canvas", "iframe", "object", "embed",
            "svg", "input", "textarea", "select", "option", "button", "table", "td", "th", "tr",
            "col", "colgroup", "ol", "ul", "li", "slot",
        ]
        .iter()
        .any(|local| (*name.local).eq_ignore_ascii_case(local))
    })
}

/// Whether `node` or an ancestor takes part in a shadow tree's composition.
fn composed_through_slots<D: LayoutDom>(dom: &D, node: D::NodeId) -> bool
where
    D::NodeId: Copy + Eq + Hash,
{
    std::iter::successors(Some(node), |node| dom.parent(*node))
        .any(|node| dom.assigned_slot(node).is_some() || dom.shadow_root(node).is_some())
}
