/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Retained-tree publication: partial updates against a whole-tree model and
//! a real `accesskit_consumer` reader.

use std::collections::HashMap;
use std::sync::Arc;

use accesskit::{ActivationHandler, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use accesskit_consumer::{NodeRef, Tree, TreeChangeHandler};

use super::BridgeState;
use super::retained::RetainedTree;

struct Quiet;
impl TreeChangeHandler for Quiet {
    fn node_added(&mut self, _: &NodeRef<'_>) {}
    fn node_updated(&mut self, _: &NodeRef<'_>, _: &NodeRef<'_>) {}
    fn node_removed(&mut self, _: &NodeRef<'_>) {}
    fn focus_moved(&mut self, _: Option<&NodeRef<'_>>, _: Option<&NodeRef<'_>>) {}
}

/// The whole-tree caller's view: labels and child order per node.
#[derive(Clone)]
struct Model {
    root: NodeId,
    toolkit: String,
    focus: NodeId,
    nodes: HashMap<NodeId, (String, Vec<NodeId>)>,
    next: u64,
}

impl Model {
    fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(NodeId(0), ("root".to_owned(), vec![NodeId(1), NodeId(2)]));
        nodes.insert(NodeId(1), ("a".to_owned(), vec![NodeId(3)]));
        nodes.insert(NodeId(2), ("b".to_owned(), Vec::new()));
        nodes.insert(NodeId(3), ("c".to_owned(), Vec::new()));
        Self {
            root: NodeId(0),
            toolkit: "t0".into(),
            focus: NodeId(3),
            nodes,
            next: 4,
        }
    }
    fn node(&self, id: NodeId) -> Node {
        let (label, children) = &self.nodes[&id];
        let mut node = Node::new(if id == self.root {
            Role::Window
        } else {
            Role::GenericContainer
        });
        node.set_label(label.as_str());
        node.set_children(children.clone());
        node
    }
    fn info(&self) -> TreeInfo {
        let mut info = TreeInfo::new(self.root);
        info.toolkit_name = Some(self.toolkit.clone());
        info
    }
    /// What a whole-tree caller would install: every node, preorder.
    fn full(&self, focus: NodeId) -> TreeUpdate {
        let mut nodes = Vec::new();
        let mut stack = vec![self.root];
        while let Some(id) = stack.pop() {
            stack.extend(self.nodes[&id].1.iter().rev().copied());
            nodes.push((id, self.node(id)));
        }
        TreeUpdate {
            nodes,
            tree: Some(self.info()),
            tree_id: TreeId::ROOT,
            focus,
        }
    }
    fn partial(&self, listed: &[NodeId], tree: bool) -> TreeUpdate {
        TreeUpdate {
            nodes: listed.iter().map(|id| (*id, self.node(*id))).collect(),
            tree: tree.then(|| self.info()),
            tree_id: TreeId::ROOT,
            focus: self.focus,
        }
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|(_, (_, children))| children.contains(&id))
            .map(|(parent, _)| *parent)
    }
    fn subtree(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = vec![id];
        let mut i = 0;
        while i < out.len() {
            out.extend(self.nodes[&out[i]].1.iter().copied());
            i += 1;
        }
        out
    }
    fn sorted_ids(&self) -> Vec<NodeId> {
        let mut ids: Vec<_> = self.nodes.keys().copied().collect();
        ids.sort();
        ids
    }

    /// One random edit; returns the partial update a minimal caller sends.
    fn edit(&mut self, rng: &mut Lcg) -> TreeUpdate {
        let ids = self.sorted_ids();
        let pick = |rng: &mut Lcg| ids[rng.below(ids.len())];
        let non_root: Vec<_> = ids.iter().copied().filter(|id| *id != self.root).collect();
        let listed = match rng.below(6) {
            0 | 1 => {
                let parent = pick(rng);
                let child = NodeId(self.next);
                let grandchild = NodeId(self.next + 1);
                self.next += 2;
                let with_grandchild = rng.below(2) == 0;
                let kids = if with_grandchild {
                    vec![grandchild]
                } else {
                    Vec::new()
                };
                self.nodes.insert(child, (format!("n{}", child.0), kids));
                let mut listed = vec![parent, child];
                if with_grandchild {
                    self.nodes
                        .insert(grandchild, (format!("n{}", grandchild.0), Vec::new()));
                    listed.push(grandchild);
                }
                let at = rng.below(self.nodes[&parent].1.len() + 1);
                self.nodes.get_mut(&parent).unwrap().1.insert(at, child);
                listed
            },
            2 if !non_root.is_empty() => {
                let gone = non_root[rng.below(non_root.len())];
                let parent = self.parent(gone).unwrap();
                self.nodes
                    .get_mut(&parent)
                    .unwrap()
                    .1
                    .retain(|id| *id != gone);
                for id in self.subtree(gone) {
                    self.nodes.remove(&id);
                }
                vec![parent]
            },
            3 if !non_root.is_empty() => {
                let moved = non_root[rng.below(non_root.len())];
                let inside = self.subtree(moved);
                let old = self.parent(moved).unwrap();
                let targets: Vec<_> = ids
                    .iter()
                    .copied()
                    .filter(|id| !inside.contains(id) && *id != old)
                    .collect();
                if targets.is_empty() {
                    vec![]
                } else {
                    let new = targets[rng.below(targets.len())];
                    self.nodes
                        .get_mut(&old)
                        .unwrap()
                        .1
                        .retain(|id| *id != moved);
                    self.nodes.get_mut(&new).unwrap().1.push(moved);
                    if rng.below(2) == 0 {
                        vec![old, new]
                    } else {
                        vec![new, old]
                    }
                }
            },
            4 => {
                let id = pick(rng);
                self.nodes.get_mut(&id).unwrap().0 = format!("l{}", rng.next());
                vec![id]
            },
            _ => vec![],
        };
        let tree = rng.below(20) == 0;
        if tree {
            self.toolkit = format!("t{}", rng.next());
        }
        let ids = self.sorted_ids();
        self.focus = ids[rng.below(ids.len())];
        self.partial(&listed, tree)
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn reader_nodes(tree: &Tree) -> Vec<(NodeId, Node)> {
    let mut out = Vec::new();
    let mut stack = vec![tree.state().root()];
    while let Some(node) = stack.pop() {
        stack.extend(node.children().rev());
        out.push((node.locate().0, node.data().clone()));
    }
    out
}

fn reader_matches(reader: &Tree, model: &Model) {
    assert_eq!(reader_nodes(reader), model.full(model.focus).nodes);
    assert_eq!(reader.state().focus_in_tree().locate().0, model.focus);
}

fn installed(model: &Model) -> BridgeState {
    let mut state = BridgeState::new(Arc::new(|| {}), None);
    assert!(state.replace_host(model.full(model.focus)));
    state
}

#[test]
fn retained_tree_matches_whole_tree_caller_after_random_partials() {
    for seed in 1..=8 {
        let mut rng = Lcg(seed);
        let mut model = Model::new();
        let mut tree = RetainedTree::from_full(model.full(model.focus)).unwrap();
        for _ in 0..400 {
            let update = model.edit(&mut rng);
            tree.apply(update).expect("a minimal partial is accepted");
            assert_eq!(tree.snapshot(model.focus), model.full(model.focus));
        }
    }
}

#[test]
fn readers_hold_the_whole_tree_across_skips_and_activations() {
    for seed in 1..=8 {
        let mut rng = Lcg(seed * 7919);
        let mut model = Model::new();
        let mut state = installed(&model);
        let mut activation = state.activation();
        let mut reader: Option<Tree> = None;
        for _ in 0..400 {
            match rng.below(10) {
                0 => {
                    let initial = activation.request_initial_tree().unwrap();
                    assert_eq!(initial, model.full(model.root), "whole tree, root focus");
                    reader = Some(Tree::new(initial, true));
                },
                1 => reader = None,
                _ => {},
            }
            let in_step =
                state.delivered && !state.replay.load(std::sync::atomic::Ordering::Acquire);
            let update = model.edit(&mut rng);
            state.update_partial(update.clone()).unwrap();
            // An adapter that has taken its initial tree may still drop updates
            // before it reports active; those must reach it later, coalesced.
            let dropping = rng.below(4) == 0;
            let mut supplied = Vec::new();
            let delivered = state.flush(|supply| match reader.as_mut() {
                Some(reader) if !dropping => {
                    let update = supply();
                    supplied.push(update.clone());
                    reader.update_and_process_changes(update, &mut Quiet);
                    true
                },
                _ => false,
            });
            assert_eq!(delivered, reader.is_some() && !dropping);
            if let Some(reader) = reader.as_ref().filter(|_| delivered) {
                reader_matches(reader, &model);
                if in_step {
                    assert_eq!(
                        supplied,
                        vec![update],
                        "an in-sync reader gets the partial alone"
                    );
                }
            }
        }
    }
}

#[test]
fn active_reader_is_handed_only_the_partial() {
    let mut model = Model::new();
    let mut state = installed(&model);
    let initial = state.activation().request_initial_tree().unwrap();
    let mut reader = Tree::new(initial, true);
    let feed = |state: &mut BridgeState, reader: &mut Tree| {
        let mut supplied = Vec::new();
        assert!(state.flush(|supply| {
            let update = supply();
            supplied.push(update.clone());
            reader.update_and_process_changes(update, &mut Quiet);
            true
        }));
        supplied
    };
    assert_eq!(
        feed(&mut state, &mut reader)[0].nodes.len(),
        4,
        "install replays the host tree"
    );
    model.nodes.get_mut(&NodeId(2)).unwrap().0 = "renamed".into();
    let partial = model.partial(&[NodeId(2)], false);
    state.update_partial(partial.clone()).unwrap();
    assert_eq!(feed(&mut state, &mut reader), vec![partial]);
    reader_matches(&reader, &model);
}

#[test]
fn late_reader_gets_the_whole_tree_then_the_current_focus() {
    let mut model = Model::new();
    let mut state = installed(&model);
    let mut rng = Lcg(3);
    for _ in 0..30 {
        let update = model.edit(&mut rng);
        state.update_partial(update).unwrap();
        assert!(!state.flush(|_| false), "no reader is active");
    }
    let initial = state.activation().request_initial_tree().unwrap();
    assert_eq!(initial, model.full(model.root));
    let mut reader = Tree::new(initial, true);
    let mut supplied = Vec::new();
    assert!(state.flush(|supply| {
        let update = supply();
        supplied.push(update.clone());
        reader.update_and_process_changes(update, &mut Quiet);
        true
    }));
    assert_eq!(supplied.len(), 1);
    assert!(supplied[0].nodes.is_empty() && supplied[0].tree.is_none());
    assert_eq!(supplied[0].focus, model.focus);
    reader_matches(&reader, &model);
}

#[test]
fn removing_a_subtree_drops_it_from_the_retained_tree() {
    let mut model = Model::new();
    let mut tree = RetainedTree::from_full(model.full(model.focus)).unwrap();
    model.nodes.get_mut(&NodeId(0)).unwrap().1 = vec![NodeId(2)];
    model.nodes.remove(&NodeId(1));
    model.nodes.remove(&NodeId(3));
    model.focus = NodeId(2);
    tree.apply(model.partial(&[NodeId(0)], false)).unwrap();
    let snapshot = tree.snapshot(NodeId(0));
    let ids: Vec<_> = snapshot.nodes.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![NodeId(0), NodeId(2)]);
    assert_eq!(snapshot, model.full(NodeId(0)));
}

#[test]
fn refused_partials_leave_the_retained_tree_unchanged() {
    let model = Model::new();
    let mut tree = RetainedTree::from_full(model.full(model.focus)).unwrap();
    let before = tree.snapshot(model.focus);
    let with_children = |id: u64, children: Vec<NodeId>| {
        let mut node = model.node(NodeId(id));
        node.set_children(children);
        node
    };
    let update = |nodes: Vec<(NodeId, Node)>| TreeUpdate {
        nodes,
        tree: None,
        tree_id: TreeId::ROOT,
        focus: NodeId(0),
    };
    let mut guest = model.node(NodeId(2));
    guest.set_tree_id(TreeId(accesskit::Uuid::from_u128(5)));
    let refused = [
        (
            "unknown child",
            update(vec![(NodeId(2), with_children(2, vec![NodeId(99)]))]),
        ),
        (
            "missing root",
            TreeUpdate {
                tree: Some(TreeInfo::new(NodeId(99))),
                ..update(vec![])
            },
        ),
        ("guest graft", update(vec![(NodeId(2), guest)])),
        (
            "guest tree",
            TreeUpdate {
                tree_id: TreeId(accesskit::Uuid::from_u128(5)),
                ..update(vec![])
            },
        ),
        (
            "orphaned listed node",
            update(vec![
                (NodeId(1), with_children(1, vec![])),
                (NodeId(3), model.node(NodeId(3))),
            ]),
        ),
        (
            "second parent",
            update(vec![(NodeId(2), with_children(2, vec![NodeId(3)]))]),
        ),
        (
            "cycle",
            update(vec![
                (NodeId(0), with_children(0, vec![NodeId(2)])),
                (NodeId(3), with_children(3, vec![NodeId(1)])),
            ]),
        ),
        (
            "unknown focus",
            TreeUpdate {
                focus: NodeId(77),
                ..update(vec![])
            },
        ),
    ];
    for (why, refused) in refused {
        assert!(tree.apply(refused).is_err(), "{why} is refused");
        assert_eq!(tree.snapshot(model.focus), before, "{why} leaves the tree");
    }
}

#[test]
fn forest_and_guest_publications_refuse_partials() {
    let model = Model::new();
    let partial = model.partial(&[NodeId(2)], false);
    let mut state = BridgeState::new(Arc::new(|| {}), None);
    assert!(
        state.update_partial(partial.clone()).is_err(),
        "nothing installed"
    );
    let safe = model.full(model.root);
    let epoch = state.activation_epoch();
    state
        .replace_batch_at(epoch, safe.clone(), vec![safe.clone()])
        .unwrap();
    assert!(
        state.update_partial(partial.clone()).is_err(),
        "epoch-bound forest"
    );
    let guest_id = TreeId(accesskit::Uuid::from_u128(9));
    let mut host = model.full(model.focus);
    host.nodes.push((NodeId(50), {
        let mut graft = Node::new(Role::GenericContainer);
        graft.set_tree_id(guest_id);
        graft
    }));
    let root = host
        .nodes
        .iter_mut()
        .find(|(id, _)| *id == NodeId(0))
        .unwrap();
    root.1.set_children(vec![NodeId(1), NodeId(2), NodeId(50)]);
    let guest = TreeUpdate {
        nodes: vec![(NodeId(0), Node::new(Role::Document))],
        tree: Some(TreeInfo::new(NodeId(0))),
        tree_id: guest_id,
        focus: NodeId(0),
    };
    state
        .replace_batch(safe.clone(), vec![host, guest])
        .unwrap();
    assert!(
        state.update_partial(partial.clone()).is_err(),
        "guest trees"
    );
    state.replace_batch(safe.clone(), vec![safe]).unwrap();
    assert!(
        state.update_partial(partial).is_ok(),
        "a root-only batch retains"
    );
}

#[test]
fn an_undelivered_full_publish_makes_the_first_partial_whole() {
    let mut model = Model::new();
    let mut state = installed(&model);
    model.nodes.get_mut(&NodeId(2)).unwrap().0 = "renamed".into();
    state
        .update_partial(model.partial(&[NodeId(2)], false))
        .unwrap();
    let mut supplied = Vec::new();
    assert!(state.flush(|supply| {
        supplied.push(supply());
        true
    }));
    assert_eq!(supplied, vec![model.full(model.focus)]);
}

#[test]
fn a_new_root_adopts_the_old_one_and_an_existing_child_cannot_be_promoted() {
    let model = Model::new();
    let mut state = installed(&model);
    let initial = state.activation().request_initial_tree().unwrap();
    let mut reader = Tree::new(initial, true);
    assert!(state.flush(|supply| {
        reader.update_and_process_changes(supply(), &mut Quiet);
        true
    }));
    let promote = TreeUpdate {
        nodes: vec![],
        tree: Some(TreeInfo::new(NodeId(1))),
        tree_id: TreeId::ROOT,
        focus: NodeId(1),
    };
    assert!(state.update_partial(promote).is_err());
    let mut top = Node::new(Role::Window);
    top.set_children(vec![NodeId(0)]);
    let reroot = TreeUpdate {
        nodes: vec![(NodeId(100), top)],
        tree: Some(TreeInfo::new(NodeId(100))),
        tree_id: TreeId::ROOT,
        focus: NodeId(3),
    };
    state.update_partial(reroot).unwrap();
    assert!(state.flush(|supply| {
        reader.update_and_process_changes(supply(), &mut Quiet);
        true
    }));
    let late = state.activation().request_initial_tree().unwrap();
    assert_eq!(late.nodes[0].0, NodeId(100));
    assert_eq!(late.nodes.len(), 5);
    assert_eq!(reader_nodes(&reader), late.nodes);
}
