// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The event path (DOM "dispatch" steps 5.x, "get the parent") and
//! `composedPath()`.

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind, ShadowRootMode};

use crate::registry::TargetKey;

/// One struct of the event path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PathItem {
    pub target: TargetKey,
    /// The target listeners at this item see as `event.target`; `None` for items
    /// that share the previous item's target (same tree level, and the window).
    pub adjusted: Option<TargetKey>,
    pub root_of_closed_tree: bool,
    pub slot_in_closed_tree: bool,
}

/// A built path plus the post-dispatch `clearTargets` decision.
#[derive(Clone, Debug)]
pub struct EventPath {
    pub items: Vec<PathItem>,
    pub clear_targets: bool,
}

fn node_root(dom: &ScriptedDom, mut node: NodeId) -> NodeId {
    while let Some(parent) = dom.parent(node) {
        node = parent;
    }
    node
}

fn is_closed_shadow_root(dom: &ScriptedDom, node: NodeId) -> bool {
    dom.kind(node) == NodeKind::ShadowRoot
        && dom
            .shadow_init(node)
            .is_some_and(|init| init.mode == ShadowRootMode::Closed)
}

fn is_shadow_including_inclusive_ancestor(dom: &ScriptedDom, ancestor: NodeId, node: NodeId) -> bool {
    let mut current = Some(node);
    while let Some(c) = current {
        if c == ancestor {
            return true;
        }
        current = dom
            .parent(c)
            .or_else(|| (dom.kind(c) == NodeKind::ShadowRoot).then(|| dom.shadow_host_of(c)).flatten());
    }
    false
}

/// DOM "get the parent". `first` is the path's first invocation target.
fn event_parent(
    dom: &ScriptedDom,
    node: NodeId,
    first: NodeId,
    event_type: &str,
    composed: bool,
    window: Option<u64>,
) -> Option<TargetKey> {
    match dom.kind(node) {
        NodeKind::Document => {
            if event_type == "load" {
                None
            } else {
                window.map(TargetKey::Window)
            }
        },
        NodeKind::ShadowRoot => {
            if !composed && node_root(dom, first) == node {
                None
            } else {
                dom.shadow_host_of(node).map(TargetKey::Node)
            }
        },
        _ => dom
            .assigned_slot_of(node)
            .or_else(|| dom.parent(node))
            .map(TargetKey::Node),
    }
}

/// Build the path for an event dispatched at `target`. `window` is the key of
/// the window that terminates a connected path; a non-node target has no tree.
pub fn build_path(
    dom: &ScriptedDom,
    target: TargetKey,
    window: Option<u64>,
    event_type: &str,
    composed: bool,
) -> EventPath {
    let first_node = match target {
        TargetKey::Node(id) => id,
        _ => {
            return EventPath {
                items: vec![PathItem {
                    target,
                    adjusted: Some(target),
                    root_of_closed_tree: false,
                    slot_in_closed_tree: false,
                }],
                clear_targets: false,
            };
        },
    };
    let mut items = vec![PathItem {
        target,
        adjusted: Some(target),
        root_of_closed_tree: is_closed_shadow_root(dom, first_node),
        slot_in_closed_tree: false,
    }];
    let mut slottable = dom.assigned_slot_of(first_node).is_some();
    let mut current_target = first_node;
    let mut last_adjusted = first_node;
    let mut parent = event_parent(dom, first_node, first_node, event_type, composed, window);
    while let Some(p) = parent {
        let mut slot_in_closed = false;
        let TargetKey::Node(node) = p else {
            items.push(PathItem {
                target: p,
                adjusted: None,
                root_of_closed_tree: false,
                slot_in_closed_tree: false,
            });
            break;
        };
        if slottable {
            slottable = false;
            slot_in_closed = is_closed_shadow_root(dom, node_root(dom, node));
        }
        if dom.assigned_slot_of(node).is_some() {
            slottable = true;
        }
        let root_closed = is_closed_shadow_root(dom, node);
        if is_shadow_including_inclusive_ancestor(dom, node_root(dom, current_target), node) {
            items.push(PathItem {
                target: p,
                adjusted: None,
                root_of_closed_tree: root_closed,
                slot_in_closed_tree: slot_in_closed,
            });
        } else {
            current_target = node;
            last_adjusted = node;
            items.push(PathItem {
                target: p,
                adjusted: Some(p),
                root_of_closed_tree: root_closed,
                slot_in_closed_tree: slot_in_closed,
            });
        }
        parent = event_parent(dom, node, first_node, event_type, composed, window);
    }
    let clear_targets = dom.kind(node_root(dom, last_adjusted)) == NodeKind::ShadowRoot;
    EventPath {
        items,
        clear_targets,
    }
}

/// `Event.composedPath()` (DOM) for `path` while `current` is firing.
pub fn composed_path(path: &[PathItem], current: TargetKey) -> Vec<TargetKey> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut result = vec![current];
    let mut current_index = 0usize;
    let mut level = 0i32;
    for index in (0..path.len()).rev() {
        if path[index].root_of_closed_tree {
            level += 1;
        }
        if path[index].target == current {
            current_index = index;
            break;
        }
        if path[index].slot_in_closed_tree {
            level -= 1;
        }
    }
    let (mut hidden, mut max_hidden) = (level, level);
    for index in (0..current_index).rev() {
        if path[index].root_of_closed_tree {
            hidden += 1;
        }
        if hidden <= max_hidden {
            result.insert(0, path[index].target);
        }
        if path[index].slot_in_closed_tree {
            hidden -= 1;
            if hidden < max_hidden {
                max_hidden = hidden;
            }
        }
    }
    let (mut hidden, mut max_hidden) = (level, level);
    for item in &path[current_index + 1..] {
        if item.slot_in_closed_tree {
            hidden += 1;
        }
        if hidden <= max_hidden {
            result.push(item.target);
        }
        if item.root_of_closed_tree {
            hidden -= 1;
            if hidden < max_hidden {
                max_hidden = hidden;
            }
        }
    }
    result
}
