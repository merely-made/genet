/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A retained accessibility projection (D62). An update patches a batch it
//! can prove local (D63, D67) and rebuilds the whole projection otherwise.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use document_session_api::{DocumentA11yNode, DocumentA11yNodeId, DocumentA11yProjection};
use genet_livery::LiveryLayout;
use layout_dom_api::{DomMutation, LayoutDom, NodeKind};

use super::name::{Names, REFERENCE_ATTRIBUTES};
use super::pass::Pass;
use super::{A11yStyleQueries, AncestorScroll, aria_true, neutral_node_id, projection_from};
use crate::render::ScrollOffsets;

/// What an update did. Node data is read back by id from the session.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum A11yUpdate {
    /// Only these nodes differ from the previous projection.
    Patched {
        added: Vec<DocumentA11yNodeId>,
        changed: Vec<DocumentA11yNodeId>,
        removed: Vec<DocumentA11yNodeId>,
        /// Kept nodes whose bounds alone moved (D69). A host that supplies its
        /// own bounds ignores this list.
        moved: Vec<DocumentA11yNodeId>,
    },
    /// The batch was not provably local; every node was projected again.
    #[default]
    Rebuilt,
}

/// Why an update rebuilt rather than patched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// No restyled set: the style session restyled the whole document.
    NoRestyledSet,
    ScrollOffsetsChanged,
    /// A mutation other than an insertion, removal or attribute write.
    UnsupportedMutation,
    ReferenceAttributeWritten,
    /// A kept reference now resolves differently.
    ReferenceChanged,
    /// An inserted reference resolves outside its inserted subtree.
    ReferenceLeavesSubtree,
    /// A removed reference pointed at a node that remains.
    RemovedReferenceToKeptNode,
    /// A change inside a node some reference reads.
    ChangeInsideReferencedNode,
    /// A change inside a `<label>`, which names its control.
    ChangeInsideLabel,
    ChangeInsideAriaHidden,
    /// A shadow host, slot or slotted node at the change.
    ShadowTree,
    /// A kept node gained or lost its projected node.
    ProjectionChanged,
    /// An inserted node was already projected (a move).
    Reinserted,
}

/// A batch's mutations, sorted for admission.
struct Batch<Id> {
    /// Every node recorded as inserted.
    inserted: HashSet<Id>,
    /// Kept nodes whose subtree or own data the batch changed.
    touched: Vec<Id>,
}

impl<Id: Copy + Eq + Hash> Batch<Id> {
    fn classify<D: LayoutDom<NodeId = Id>>(
        dom: &D,
        mutations: &[DomMutation<Id>],
    ) -> Result<Self, Refusal> {
        let mut batch = Self {
            inserted: HashSet::new(),
            touched: Vec::new(),
        };
        let mut written = Vec::new();
        for mutation in mutations {
            match mutation {
                DomMutation::Inserted { node, .. } => {
                    batch.inserted.insert(*node);
                },
                DomMutation::Removed { former_parent, .. } => batch.touched.push(*former_parent),
                DomMutation::AttributeChanged { node, name, .. } => {
                    written.push((*node, REFERENCE_ATTRIBUTES.contains(&&*name.local)));
                },
                _ => return Err(Refusal::UnsupportedMutation),
            }
        }
        // Writes inside an entering subtree are part of it; a kept node's
        // reference attribute is not patched.
        for (node, reference) in written {
            if !dom.is_live(node) || batch.root_of(dom, node).is_some() {
                continue;
            }
            if reference {
                return Err(Refusal::ReferenceAttributeWritten);
            }
            batch.touched.push(node);
        }
        Ok(batch)
    }

    /// The entering root `node` lies in, if any.
    fn root_of<D: LayoutDom<NodeId = Id>>(&self, dom: &D, node: Id) -> Option<Id> {
        let mut at = node;
        loop {
            let parent = flat_parent(dom, at);
            if self.inserted.contains(&at) && parent.is_none_or(|p| !self.inserted.contains(&p)) {
                return Some(at);
            }
            at = parent?;
        }
    }
}

struct Retained<Id> {
    node: DocumentA11yNode,
    dom: Id,
    scroll: AncestorScroll,
}

/// The last projection of one document, kept to patch the next.
pub struct A11ySession<Id> {
    revision: u64,
    root: DocumentA11yNodeId,
    nodes: HashMap<DocumentA11yNodeId, Retained<Id>>,
    focus: Option<Id>,
    scroll: ScrollOffsets<Id>,
    references: HashMap<Id, Vec<Option<Id>>>,
    refused: Option<Refusal>,
}

fn names<'a, D: LayoutDom>(dom: &'a D, style: &'a A11yStyleQueries<'a, D::NodeId>) -> Names<'a, D>
where
    D::NodeId: Copy + Eq + Hash,
{
    Names::new(dom)
        .with_generated(style.generated)
        .with_rendered(style.rendered)
}

fn flat_parent<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<D::NodeId> {
    if !dom.has_shadow_trees() {
        return dom.parent(node);
    }
    dom.assigned_slot(node)
        .or_else(|| dom.parent(node))
        .or_else(|| dom.shadow_host(node))
}

fn tag<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<&str> {
    dom.element_name(node).map(|name| &*name.local)
}

impl<Id: Copy + Eq + Hash> A11ySession<Id> {
    /// A full projection, as [`super::document_a11y_projection_with_style`].
    pub fn new<D: LayoutDom<NodeId = Id>>(
        dom: &D,
        fragments: &LiveryLayout<Id>,
        focus: Option<Id>,
        revision: u64,
        scroll_offsets: Option<&ScrollOffsets<Id>>,
        style: &A11yStyleQueries<'_, Id>,
    ) -> Self {
        let mut session = Self {
            revision,
            root: neutral_node_id(dom, dom.document()),
            nodes: HashMap::new(),
            focus,
            scroll: ScrollOffsets::new(),
            references: HashMap::new(),
            refused: None,
        };
        let names = names(dom, style);
        session.rebuild(
            &Pass {
                dom,
                fragments,
                scroll_offsets,
                names: &names,
                focus,
            },
            revision,
        );
        session
    }

    /// Bring the projection up to date with `mutations`, the complete batch
    /// since the last update. `restyled` is the style session's restyled set
    /// for the same batch (`IncrementalStyle::restyled_elements`); `None`
    /// rebuilds.
    #[expect(
        clippy::too_many_arguments,
        reason = "the projection's inputs, as the free functions take them, plus the batch"
    )]
    pub fn update<D: LayoutDom<NodeId = Id>>(
        &mut self,
        dom: &D,
        fragments: &LiveryLayout<Id>,
        focus: Option<Id>,
        revision: u64,
        scroll_offsets: Option<&ScrollOffsets<Id>>,
        style: &A11yStyleQueries<'_, Id>,
        restyled: Option<&HashSet<Id>>,
        mutations: &[DomMutation<Id>],
    ) -> A11yUpdate {
        let names = names(dom, style);
        let pass = Pass {
            dom,
            fragments,
            scroll_offsets,
            names: &names,
            focus,
        };
        match self.patch(&pass, restyled, mutations) {
            Ok(update) => {
                self.refused = None;
                self.revision = revision;
                self.focus = focus;
                update
            },
            Err(refusal) => {
                self.refused = Some(refusal);
                self.rebuild(&pass, revision);
                A11yUpdate::Rebuilt
            },
        }
    }

    pub fn node(&self, id: DocumentA11yNodeId) -> Option<&DocumentA11yNode> {
        self.nodes.get(&id).map(|retained| &retained.node)
    }

    pub fn root(&self) -> DocumentA11yNodeId {
        self.root
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The whole projection, nodes in the order the full projection gives.
    pub fn projection(&self) -> DocumentA11yProjection {
        let mut out = Vec::with_capacity(self.nodes.len());
        let mut pending = vec![(self.root, false)];
        while let Some((id, expanded)) = pending.pop() {
            let Some(retained) = self.nodes.get(&id) else {
                continue;
            };
            if expanded {
                out.push(retained.node.clone());
            } else {
                pending.push((id, true));
                pending.extend(retained.node.children.iter().rev().map(|c| (*c, false)));
            }
        }
        projection_from(self.revision, self.root, out)
    }

    /// Why the last update rebuilt, if it did.
    #[cfg(test)]
    pub(crate) fn refusal(&self) -> Option<Refusal> {
        self.refused
    }

    fn rebuild<D: LayoutDom<NodeId = Id>>(&mut self, pass: &Pass<'_, D>, revision: u64) {
        let dom = pass.dom;
        let (mut nodes, mut keys) = (Vec::new(), Vec::new());
        pass.walk(
            dom.document(),
            AncestorScroll::default(),
            None,
            &mut nodes,
            &mut keys,
        );
        self.nodes = nodes
            .into_iter()
            .zip(keys)
            .map(|(node, (dom, scroll))| (node.id, Retained { node, dom, scroll }))
            .collect();
        self.revision = revision;
        self.root = neutral_node_id(dom, dom.document());
        self.focus = pass.focus;
        self.scroll = pass.scroll_offsets.cloned().unwrap_or_default();
        self.references = pass.names.resolved_references();
    }

    fn patch<D: LayoutDom<NodeId = Id>>(
        &mut self,
        pass: &Pass<'_, D>,
        restyled: Option<&HashSet<Id>>,
        mutations: &[DomMutation<Id>],
    ) -> Result<A11yUpdate, Refusal> {
        let dom = pass.dom;
        let restyled = restyled.ok_or(Refusal::NoRestyledSet)?;
        if pass.scroll_offsets.cloned().unwrap_or_default() != self.scroll {
            return Err(Refusal::ScrollOffsetsChanged);
        }
        let mut batch = Batch::classify(dom, mutations)?;
        let references = pass.names.resolved_references();
        self.check_references(dom, &references, &batch)?;
        let roots = self.entering_roots(dom, &mut batch)?;
        self.restyled_kept(pass, &mut batch, restyled)?;
        let targets: HashSet<Id> = references.values().flatten().flatten().copied().collect();
        let mut again = self.chains(pass, &batch, &targets)?;
        for focus in [self.focus, pass.focus].into_iter().flatten() {
            if dom.is_live(focus) {
                again.insert(neutral_node_id(dom, focus));
            }
        }
        let removed: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, retained)| !dom.is_live(retained.dom))
            .map(|(id, _)| *id)
            .collect();
        for id in &removed {
            self.nodes.remove(id);
        }
        let added = self.insert(pass, roots);
        let (changed, moved) = self.refresh(pass, &added, &again)?;
        self.references = references;
        Ok(A11yUpdate::Patched {
            added,
            changed,
            removed,
            moved,
        })
    }

    /// Entering element roots with their parents. A root's parent is touched.
    fn entering_roots<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        batch: &mut Batch<Id>,
    ) -> Result<Vec<(Id, Id)>, Refusal> {
        let mut roots = Vec::new();
        let live: Vec<_> = batch
            .inserted
            .iter()
            .copied()
            .filter(|n| dom.is_live(*n))
            .collect();
        for node in live {
            let Some(parent) = flat_parent(dom, node) else {
                continue;
            };
            if batch.inserted.contains(&parent) {
                continue;
            }
            batch.touched.push(parent);
            if dom.kind(node) != NodeKind::Element {
                continue;
            }
            if self.nodes.contains_key(&neutral_node_id(dom, node)) {
                return Err(Refusal::Reinserted);
            }
            if dom.has_shadow_trees()
                && (dom.shadow_root(parent).is_some()
                    || tag(dom, parent) == Some("slot")
                    || dom.assigned_slot(node).is_some())
            {
                return Err(Refusal::ShadowTree);
            }
            roots.push((node, parent));
        }
        Ok(roots)
    }

    /// Restyled kept elements are touched; one that gained or lost its
    /// projected node refuses.
    fn restyled_kept<D: LayoutDom<NodeId = Id>>(
        &self,
        pass: &Pass<'_, D>,
        batch: &mut Batch<Id>,
        restyled: &HashSet<Id>,
    ) -> Result<(), Refusal> {
        let dom = pass.dom;
        for node in restyled.iter().copied() {
            if !dom.is_live(node)
                || dom.kind(node) != NodeKind::Element
                || batch.root_of(dom, node).is_some()
            {
                continue;
            }
            let retained = self.nodes.contains_key(&neutral_node_id(dom, node));
            if retained != (pass.projected(node) && !self.under_aria_hidden(dom, node)) {
                return Err(Refusal::ProjectionChanged);
            }
            batch.touched.push(node);
        }
        Ok(())
    }

    /// Every kept projected node on a touched node's ancestor chain, to be
    /// projected again. The chain must hold nothing a name reads across.
    fn chains<D: LayoutDom<NodeId = Id>>(
        &self,
        pass: &Pass<'_, D>,
        batch: &Batch<Id>,
        targets: &HashSet<Id>,
    ) -> Result<HashSet<DocumentA11yNodeId>, Refusal> {
        let dom = pass.dom;
        let mut again = HashSet::new();
        let mut seen = HashSet::new();
        for start in batch.touched.iter().copied() {
            if dom.is_live(start) && self.under_aria_hidden(dom, start) {
                return Err(Refusal::ChangeInsideAriaHidden);
            }
            let mut at = Some(start);
            while let Some(node) = at.filter(|n| dom.is_live(*n) && seen.insert(*n)) {
                at = flat_parent(dom, node);
                if batch.root_of(dom, node).is_some() {
                    continue;
                }
                if tag(dom, node) == Some("label") {
                    return Err(Refusal::ChangeInsideLabel);
                }
                if targets.contains(&node) {
                    return Err(Refusal::ChangeInsideReferencedNode);
                }
                let id = neutral_node_id(dom, node);
                if self.nodes.contains_key(&id) {
                    again.insert(id);
                } else if dom.kind(node) == NodeKind::Element && pass.projected(node) {
                    return Err(Refusal::ProjectionChanged);
                }
            }
        }
        Ok(again)
    }

    /// Project each entering subtree under its owner; returns the new ids.
    fn insert<D: LayoutDom<NodeId = Id>>(
        &mut self,
        pass: &Pass<'_, D>,
        roots: Vec<(Id, Id)>,
    ) -> Vec<DocumentA11yNodeId> {
        let mut added = Vec::new();
        for (root, parent) in roots {
            let (owner, scroll) = self.place(pass, parent);
            let (mut nodes, mut keys) = (Vec::new(), Vec::new());
            pass.walk(root, scroll, Some(owner), &mut nodes, &mut keys);
            for (node, (dom, scroll)) in nodes.into_iter().zip(keys) {
                added.push(node.id);
                self.nodes.insert(node.id, Retained { node, dom, scroll });
            }
        }
        added
    }

    /// Project `again` anew and refresh every kept node's bounds (D69);
    /// returns the changed nodes and those whose bounds alone moved.
    fn refresh<D: LayoutDom<NodeId = Id>>(
        &mut self,
        pass: &Pass<'_, D>,
        added: &[DocumentA11yNodeId],
        again: &HashSet<DocumentA11yNodeId>,
    ) -> Result<(Vec<DocumentA11yNodeId>, Vec<DocumentA11yNodeId>), Refusal> {
        let fresh: HashSet<_> = added.iter().copied().collect();
        let (mut changed, mut moved) = (Vec::new(), Vec::new());
        for (id, retained) in &mut self.nodes {
            if fresh.contains(id) {
                continue;
            }
            let bounds = pass.bounds(retained.dom, retained.scroll);
            if bounds.is_none() && pass.dom.kind(retained.dom) != NodeKind::Document {
                return Err(Refusal::ProjectionChanged);
            }
            let mut differs = false;
            if again.contains(id) {
                let mut node = pass.node(
                    retained.dom,
                    *id,
                    retained.scroll,
                    retained.node.parent,
                    pass.children(retained.dom),
                );
                // Compare everything but bounds, which are reported apart.
                node.bounds = retained.node.bounds;
                differs = node != retained.node;
                if differs {
                    changed.push(*id);
                    retained.node = node;
                }
            }
            if bounds != retained.node.bounds {
                retained.node.bounds = bounds;
                if !differs {
                    moved.push(*id);
                }
            }
        }
        Ok((changed, moved))
    }

    /// D67: references are unchanged outside the inserted subtrees, and an
    /// inserted reference reads only its own subtree.
    fn check_references<D: LayoutDom<NodeId = Id>>(
        &self,
        dom: &D,
        references: &HashMap<Id, Vec<Option<Id>>>,
        batch: &Batch<Id>,
    ) -> Result<(), Refusal> {
        for (referrer, targets) in references {
            if let Some(previous) = self.references.get(referrer) {
                if previous != targets {
                    return Err(Refusal::ReferenceChanged);
                }
                continue;
            }
            let root = batch
                .root_of(dom, *referrer)
                .ok_or(Refusal::ReferenceChanged)?;
            if targets
                .iter()
                .flatten()
                .any(|target| batch.root_of(dom, *target) != Some(root))
            {
                return Err(Refusal::ReferenceLeavesSubtree);
            }
        }
        for (referrer, targets) in &self.references {
            if !references.contains_key(referrer)
                && targets.iter().flatten().any(|t| dom.is_live(*t))
            {
                return Err(Refusal::RemovedReferenceToKeptNode);
            }
        }
        Ok(())
    }

    fn under_aria_hidden<D: LayoutDom<NodeId = Id>>(&self, dom: &D, node: Id) -> bool {
        let mut at = Some(node);
        while let Some(node) = at {
            if aria_true(dom, node, "aria-hidden") {
                return true;
            }
            at = flat_parent(dom, node);
        }
        false
    }

    /// The projected owner of a node inserted under `parent`, and the scroll
    /// its ancestors apply.
    fn place<D: LayoutDom<NodeId = Id>>(
        &self,
        pass: &Pass<'_, D>,
        parent: Id,
    ) -> (DocumentA11yNodeId, AncestorScroll) {
        let mut owner = None;
        let mut scroll = AncestorScroll::default();
        let mut at = Some(parent);
        while let Some(node) = at {
            let id = neutral_node_id(pass.dom, node);
            if owner.is_none() && self.nodes.contains_key(&id) {
                owner = Some(id);
            }
            scroll = scroll.including(pass.own_scroll(node));
            at = flat_parent(pass.dom, node);
        }
        (owner.unwrap_or(self.root), scroll)
    }
}
