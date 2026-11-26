/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The bridge's retained host tree, patched by AccessKit partial updates.
//!
//! [`RetainedTree::apply`] follows `accesskit_consumer`'s update rules: a
//! listed node replaces its stored copy whole, unlisted nodes are kept, and a
//! node dropped from its parent's children (with its subtree) is removed unless
//! the same update adopts it elsewhere. It refuses, without change, any update
//! the consumer would panic on or that would leave the store other than one
//! rooted tree (a listed node that ends unreachable, a second parent, a cycle),
//! so a fresh reader served [`RetainedTree::snapshot`] and a live reader fed
//! every accepted update hold the same nodes.

use std::collections::{HashMap, HashSet};

use accesskit::{Node, NodeId, TreeId, TreeInfo, TreeUpdate};

#[cfg(test)]
thread_local! {
    /// Delta computations on this thread, for tests that prove none happen.
    pub(super) static DELTAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// One whole root-host tree with parent links for local validation.
#[derive(Clone, Debug)]
pub(super) struct RetainedTree {
    info: TreeInfo,
    focus: NodeId,
    nodes: HashMap<NodeId, Node>,
    parents: HashMap<NodeId, NodeId>,
}

/// What an accepted update changed.
pub(super) struct Applied {
    listed: Vec<NodeId>,
    tree: bool,
    removed: Vec<NodeId>,
}

impl RetainedTree {
    /// Build from a complete host tree (`tree` set, every node listed).
    pub(super) fn from_full(update: TreeUpdate) -> Result<Self, String> {
        let info = update
            .tree
            .clone()
            .ok_or("a retained tree starts from a complete host tree")?;
        let mut tree = Self {
            focus: info.root,
            info,
            nodes: HashMap::new(),
            parents: HashMap::new(),
        };
        tree.apply(update)?;
        Ok(tree)
    }

    /// Apply one partial update atomically: Err leaves the tree unchanged.
    pub(super) fn apply(&mut self, update: TreeUpdate) -> Result<Applied, String> {
        let plan = self.plan(&update)?;
        for id in &plan.removed {
            self.nodes.remove(id);
            self.parents.remove(id);
        }
        for (child, parent) in plan.adopter {
            self.parents.insert(child, parent);
        }
        self.parents.remove(&plan.root);
        let tree = update.tree.is_some();
        if let Some(info) = update.tree {
            self.info = info;
        }
        let mut listed = Vec::with_capacity(update.nodes.len());
        for (id, node) in update.nodes {
            listed.push(id);
            self.nodes.insert(id, node);
        }
        self.focus = update.focus;
        Ok(Applied {
            listed,
            tree,
            removed: plan.removed,
        })
    }

    // Read-only validation; returns the removals and new parent links.
    fn plan(&self, update: &TreeUpdate) -> Result<Plan, String> {
        if update.tree_id != TreeId::ROOT {
            return Err("partial updates cover only the host root tree".into());
        }
        let old_root = self.info.root;
        let root = update.tree.as_ref().map_or(old_root, |tree| tree.root);
        let mut listed = HashSet::with_capacity(update.nodes.len());
        for (id, node) in &update.nodes {
            if node.tree_id().is_some() {
                return Err(format!("node {id:?} enters a guest tree"));
            }
            if !listed.insert(*id) {
                return Err(format!("node {id:?} is listed twice"));
            }
        }
        let mut adopter = HashMap::new();
        for (id, node) in &update.nodes {
            for child in node.children() {
                if adopter.insert(*child, *id).is_some() {
                    return Err(format!("child {child:?} has two parents in the update"));
                }
                if !listed.contains(child) && !self.nodes.contains_key(child) {
                    return Err(format!("child {child:?} is neither retained nor listed"));
                }
            }
        }
        if !listed.contains(&root) && !self.nodes.contains_key(&root) {
            return Err(format!("root {root:?} is neither retained nor listed"));
        }
        if adopter.contains_key(&root) {
            return Err(format!("root {root:?} is listed as a child"));
        }
        // A listed parent keeps only the children it lists now.
        let parent_of = |id: &NodeId| -> Option<NodeId> {
            if *id == root {
                return None;
            }
            adopter.get(id).copied().or_else(|| {
                self.parents
                    .get(id)
                    .copied()
                    .filter(|parent| !listed.contains(parent))
            })
        };
        // The consumer's removal pass can drop an existing child promoted to
        // root, depending on listing order; a new root must be a new node.
        if self.parents.contains_key(&root) {
            return Err(format!("new root {root:?} is an existing child"));
        }
        let mut reachable = HashSet::from([root]);
        let mut reaches = |start: NodeId| -> bool {
            let mut path = Vec::new();
            let mut on_path = HashSet::new();
            let mut id = start;
            loop {
                if reachable.contains(&id) {
                    reachable.extend(path);
                    return true;
                }
                if !on_path.insert(id) {
                    return false;
                }
                path.push(id);
                match parent_of(&id) {
                    Some(parent) => id = parent,
                    None => return false,
                }
            }
        };
        for (id, _) in &update.nodes {
            if !reaches(*id) {
                return Err(format!("listed node {id:?} would be unreachable"));
            }
        }
        for (child, parent) in &adopter {
            let Some(old) = self.parents.get(child) else {
                continue;
            };
            if old != parent && !listed.contains(old) && reaches(*old) {
                return Err(format!("child {child:?} would keep its old parent {old:?}"));
            }
        }
        if !(listed.contains(&update.focus) || self.nodes.contains_key(&update.focus))
            || !reaches(update.focus)
        {
            return Err(format!("focus {:?} would not be in the tree", update.focus));
        }
        let removed = self.removals(update, old_root, root, &adopter);
        Ok(Plan {
            removed,
            adopter,
            root,
        })
    }

    // Subtrees dropped from listed parents, skipping adopted nodes and the root.
    fn removals(
        &self,
        update: &TreeUpdate,
        old_root: NodeId,
        root: NodeId,
        adopter: &HashMap<NodeId, NodeId>,
    ) -> Vec<NodeId> {
        let mut stack: Vec<NodeId> = update
            .nodes
            .iter()
            .filter_map(|(id, _)| self.nodes.get(id))
            .flat_map(|node| node.children().iter().copied())
            .filter(|child| !adopter.contains_key(child))
            .collect();
        if root != old_root
            && self.nodes.contains_key(&old_root)
            && !adopter.contains_key(&old_root)
        {
            stack.push(old_root);
        }
        let mut removed = Vec::new();
        while let Some(id) = stack.pop() {
            removed.push(id);
            if let Some(node) = self.nodes.get(&id) {
                stack.extend(
                    node.children()
                        .iter()
                        .copied()
                        .filter(|child| !adopter.contains_key(child) && *child != root),
                );
            }
        }
        removed
    }

    /// The whole tree in preorder from the root, with the given focus.
    pub(super) fn snapshot(&self, focus: NodeId) -> TreeUpdate {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![self.info.root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(&id) else {
                continue;
            };
            stack.extend(node.children().iter().rev().copied());
            nodes.push((id, node.clone()));
        }
        debug_assert_eq!(
            nodes.len(),
            self.nodes.len(),
            "retained tree is one rooted tree"
        );
        TreeUpdate {
            nodes,
            tree: Some(self.info.clone()),
            tree_id: TreeId::ROOT,
            focus,
        }
    }

    /// Root identity of the retained tree.
    pub(super) fn root(&self) -> NodeId {
        self.info.root
    }

    /// The changes a reader holding the tree from before `pending` still needs.
    pub(super) fn delta(&self, pending: &Pending) -> TreeUpdate {
        #[cfg(test)]
        DELTAS.with(|count| count.set(count.get() + 1));
        match pending {
            Pending::All => self.snapshot(self.focus),
            Pending::Nodes { ids, tree, .. } => TreeUpdate {
                nodes: ids
                    .iter()
                    .filter_map(|id| self.nodes.get(id).map(|node| (*id, node.clone())))
                    .collect(),
                tree: tree.then(|| self.info.clone()),
                tree_id: TreeId::ROOT,
                focus: self.focus,
            },
        }
    }
}

struct Plan {
    removed: Vec<NodeId>,
    adopter: HashMap<NodeId, NodeId>,
    root: NodeId,
}

/// Changes accepted since the reader's last confirmed state. Coalesced, so an
/// undelivered backlog is bounded by the tree, not by the number of updates.
#[derive(Debug)]
pub(super) enum Pending {
    /// The reader's state is unknown; send the whole tree.
    All,
    Nodes {
        ids: Vec<NodeId>,
        set: HashSet<NodeId>,
        tree: bool,
    },
}

impl Pending {
    pub(super) fn none() -> Self {
        Self::Nodes {
            ids: Vec::new(),
            set: HashSet::new(),
            tree: false,
        }
    }

    pub(super) fn record(&mut self, applied: Applied) {
        let Self::Nodes { ids, set, tree } = self else {
            return;
        };
        *tree |= applied.tree;
        if !applied.removed.is_empty() {
            for id in &applied.removed {
                set.remove(id);
            }
            ids.retain(|id| set.contains(id));
        }
        for id in applied.listed {
            if set.insert(id) {
                ids.push(id);
            }
        }
    }
}
