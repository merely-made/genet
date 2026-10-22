// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Neutral incremental style invalidation for Livery.
//!
//! The mutable DOM reports facts as `DomMutation`; this module owns the style
//! snapshots, conservative restyle hints, and retained computed plane. It uses
//! selector dependency summaries only to widen a scope. Ambiguity therefore
//! costs work, never correctness.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use layout_dom_api::{DomMutation, LayoutDom, NodeKind, QualName};
use livery::{ComputedValues, custom::CustomProperties, media::Device};

use crate::style::{SubtreeStyleContext, resolve_subtree};
use crate::{InteractionStates, SelectorTree, StylePlane, StyleSet, resolve_styles};

/// One pre-mutation attribute value retained for invalidation diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeSnapshot {
    pub name: QualName,
    pub old_value: Option<String>,
}

/// Coalesced change facts for one element. Repeated writes to one attribute
/// retain the first old value, the state before the whole mutation batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElementSnapshot<Id> {
    pub node: Id,
    pub changed_attributes: Vec<AttributeSnapshot>,
    pub state_changed: bool,
}

/// Work performed by the latest incremental style update.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RestyleStats {
    /// Elements carrying a coalesced attribute or interaction snapshot.
    pub snapshots: usize,
    /// Disjoint subtree roots after ancestor coalescing.
    pub hints: usize,
    /// Elements whose cascade was recomputed.
    pub restyled_elements: usize,
    /// Elements whose identity was needed while selector matching.
    pub selector_elements: usize,
    /// Attached elements retained after this pass.
    pub total_elements: usize,
    /// True whenever this pass recomputed every attached element.
    pub full_document: bool,
    /// The style-set generation changed since the prior pass.
    pub stylesheet_invalidated: bool,
    /// The media device changed since the prior pass.
    pub device_invalidated: bool,
}

/// A computed style plane retained across DOM, interaction, and stylesheet
/// changes. Callers supply the mutation batch without transferring ownership;
/// this lets scripting and layout observe the same DOM log independently.
pub struct IncrementalStyle<Id> {
    plane: StylePlane<Id>,
    initialized: bool,
    force_full: bool,
    stylesheet_generation: u64,
    interaction_generation: u64,
    device: Option<Device>,
    snapshots: Vec<ElementSnapshot<Id>>,
    last_stats: RestyleStats,
}

impl<Id> Default for IncrementalStyle<Id> {
    fn default() -> Self {
        Self {
            plane: StylePlane::default(),
            initialized: false,
            force_full: false,
            stylesheet_generation: 0,
            interaction_generation: 0,
            device: None,
            snapshots: Vec::new(),
            last_stats: RestyleStats::default(),
        }
    }
}

impl<Id> IncrementalStyle<Id>
where
    Id: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn styles(&self) -> &StylePlane<Id> {
        &self.plane
    }

    pub fn snapshots(&self) -> &[ElementSnapshot<Id>] {
        &self.snapshots
    }

    pub fn last_stats(&self) -> RestyleStats {
        self.last_stats
    }

    /// Force the next update through the full-document correctness path.
    pub fn invalidate(&mut self) {
        self.force_full = true;
    }

    pub fn update<D>(
        &mut self,
        dom: &D,
        style_set: &StyleSet,
        device: &Device,
        states: &InteractionStates<Id>,
        mutations: &[DomMutation<Id>],
    ) -> RestyleStats
    where
        D: LayoutDom<NodeId = Id>,
    {
        let state_nodes: Vec<_> = states
            .changed_since(self.interaction_generation)
            .filter(|id| dom.is_live(*id))
            .collect();
        self.snapshots = build_snapshots(mutations, &state_nodes);

        let generation = style_set.generation();
        let stylesheet_invalidated = self.initialized && generation != self.stylesheet_generation;
        let device_invalidated = self.initialized && self.device != Some(*device);
        if !self.initialized || self.force_full || stylesheet_invalidated || device_invalidated {
            self.plane = resolve_styles(dom, style_set, device, states);
            self.initialized = true;
            self.force_full = false;
            self.stylesheet_generation = generation;
            self.interaction_generation = states.generation();
            self.device = Some(*device);
            let total = self.plane.len();
            self.last_stats = RestyleStats {
                snapshots: self.snapshots.len(),
                hints: usize::from(total > 0),
                restyled_elements: total,
                selector_elements: total,
                total_elements: total,
                full_document: total > 0,
                stylesheet_invalidated,
                device_invalidated,
            };
            return self.last_stats;
        }

        let sibling_dependencies = style_set.has_sibling_dependencies();
        let structural_dependencies = style_set.has_structural_dependencies();
        let mut roots = Vec::new();

        for mutation in mutations {
            for_each_mutation_dependency(dom, mutation, |dependency| match dependency {
                MutationDependency::Element(node) => {
                    push_element_hint(dom, &mut roots, node, sibling_dependencies);
                },
                MutationDependency::SemanticOwner(node) | MutationDependency::Tree(node) => {
                    push_root(dom, &mut roots, node);
                },
                MutationDependency::Structure {
                    kind,
                    node,
                    boundary,
                } => {
                    if structural_dependencies {
                        // A text mutation only broadens a structural selector
                        // when it still has a parent in the live tree.
                        if kind != StructureKind::Text || dom.parent(node).is_some() {
                            push_element_hint(dom, &mut roots, boundary, sibling_dependencies);
                        }
                    } else if matches!(
                        kind,
                        StructureKind::Inserted | StructureKind::Replaced | StructureKind::Moved
                    ) {
                        push_root(dom, &mut roots, node);
                    }
                },
                MutationDependency::Removed(node) => remove_subtree(dom, &mut self.plane, node),
            });
        }
        for node in state_nodes {
            push_element_hint(dom, &mut roots, node, sibling_dependencies);
        }

        // innerHTML can retire old ids before the mutation is observed.
        self.plane.retain(|id| dom.is_live(id));
        roots.retain(|id| dom.is_live(*id));
        coalesce_roots(dom, &mut roots);
        let selector_tree = SelectorTree::for_roots(
            dom,
            states,
            &roots,
            sibling_dependencies || structural_dependencies,
        );
        let selector_elements = selector_tree.len();
        let context = (!roots.is_empty()).then(|| SubtreeStyleContext::new(dom, style_set, &roots));
        let mut restyled_elements = 0;
        for root in roots.iter().copied() {
            let (parent, parent_custom) = inherited_parent(dom, &self.plane, root);
            remove_subtree(dom, &mut self.plane, root);
            restyled_elements += resolve_subtree(
                &selector_tree,
                style_set,
                device,
                root,
                parent.as_ref(),
                parent_custom.as_ref(),
                context.as_ref().expect("nonempty restyle roots"),
                &mut self.plane,
            );
        }

        crate::style::resolve_counters(dom, &mut self.plane);
        self.stylesheet_generation = generation;
        self.interaction_generation = states.generation();
        self.device = Some(*device);
        let total = self.plane.len();
        self.last_stats = RestyleStats {
            snapshots: self.snapshots.len(),
            hints: roots.len(),
            restyled_elements,
            selector_elements,
            total_elements: total,
            full_document: total > 0 && restyled_elements >= total,
            stylesheet_invalidated: false,
            device_invalidated: false,
        };
        self.last_stats
    }
}

/// Mutation facts shared by style and retained layout. The consumers choose
/// their own restyle and formatting roots from these affected DOM owners.
pub(crate) enum MutationDependency<Id> {
    Element(Id),
    SemanticOwner(Id),
    Tree(Id),
    Structure {
        kind: StructureKind,
        node: Id,
        boundary: Id,
    },
    Removed(Id),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StructureKind {
    Inserted,
    Removed,
    Text,
    Replaced,
    Moved,
}

/// Classify one mutation after it has been applied to the DOM. Keep both the
/// former and current parent of a move, since neither can be reconstructed
/// from the post-mutation tree alone.
pub(crate) fn for_each_mutation_dependency<D: LayoutDom>(
    dom: &D,
    mutation: &DomMutation<D::NodeId>,
    mut affected: impl FnMut(MutationDependency<D::NodeId>),
) {
    match mutation {
        DomMutation::AttributeChanged {
            node,
            name,
            old_value,
        } => {
            if !dom.is_live(*node) {
                return;
            }
            affected(MutationDependency::Element(*node));
            emit_validity_owners(dom, *node, &mut affected);
            // Ownership changes reach controls outside this subtree. An ID
            // on any element can also block an external form association.
            if form_association_attribute_changed(dom, *node, name) {
                emit_form_tree(dom, *node, &mut affected);
            }
            // The first old value matters when a radio changes type.
            if radio_group_attribute_changed(dom, *node, name, old_value.as_deref())
                && let Some(root) = radio_tree_root(dom, *node)
            {
                affected(MutationDependency::Tree(root));
            }
        },
        DomMutation::FormControlStateChanged { node } => {
            if !dom.is_live(*node) {
                return;
            }
            affected(MutationDependency::Element(*node));
            emit_validity_owners(dom, *node, &mut affected);
            if is_radio_control(dom, *node)
                && let Some(root) = radio_tree_root(dom, *node)
            {
                affected(MutationDependency::Tree(root));
            }
        },
        DomMutation::FormControlInteractionStateChanged { node }
        | DomMutation::FormControlCustomValidityChanged { node }
        | DomMutation::OptionStateChanged { node } => {
            if !dom.is_live(*node) {
                return;
            }
            affected(MutationDependency::Element(*node));
            emit_validity_owners(dom, *node, &mut affected);
        },
        DomMutation::Inserted { node, parent } => {
            emit_form_tree(dom, *parent, &mut affected);
            emit_select_owners(dom, *parent, &mut affected);
            affected(MutationDependency::Structure {
                kind: StructureKind::Inserted,
                node: *node,
                boundary: *parent,
            });
        },
        DomMutation::Removed {
            node,
            former_parent,
        } => {
            affected(MutationDependency::Removed(*node));
            emit_form_tree(dom, *former_parent, &mut affected);
            emit_select_owners(dom, *former_parent, &mut affected);
            affected(MutationDependency::Structure {
                kind: StructureKind::Removed,
                node: *node,
                boundary: *former_parent,
            });
        },
        DomMutation::CharacterDataChanged { node } => {
            if !dom.is_live(*node) {
                return;
            }
            emit_form_tree(dom, *node, &mut affected);
            emit_select_owners(dom, *node, &mut affected);
            affected(MutationDependency::Structure {
                kind: StructureKind::Text,
                node: *node,
                boundary: dom.parent(*node).unwrap_or(*node),
            });
        },
        DomMutation::SubtreeReplaced { node } => {
            emit_form_tree(dom, *node, &mut affected);
            emit_select_owners(dom, *node, &mut affected);
            affected(MutationDependency::Structure {
                kind: StructureKind::Replaced,
                node: *node,
                boundary: *node,
            });
        },
        DomMutation::Moved {
            node,
            from_parent,
            to_parent,
        } => {
            for parent in [*from_parent, *to_parent] {
                emit_form_tree(dom, parent, &mut affected);
                emit_select_owners(dom, parent, &mut affected);
                affected(MutationDependency::Structure {
                    kind: StructureKind::Moved,
                    node: *node,
                    boundary: parent,
                });
            }
        },
    }
}

fn emit_validity_owners<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    affected: &mut impl FnMut(MutationDependency<D::NodeId>),
) {
    for_each_validity_dependent(dom, node, |id| {
        affected(MutationDependency::SemanticOwner(id));
    });
}

fn emit_select_owners<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    affected: &mut impl FnMut(MutationDependency<D::NodeId>),
) {
    for_each_select_validity_dependent(dom, node, |id| {
        affected(MutationDependency::SemanticOwner(id));
    });
}

fn emit_form_tree<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    affected: &mut impl FnMut(MutationDependency<D::NodeId>),
) {
    if let Some(root) = form_validity_tree_root(dom, node) {
        affected(MutationDependency::Tree(root));
    }
}

fn build_snapshots<Id>(
    mutations: &[DomMutation<Id>],
    state_nodes: &[Id],
) -> Vec<ElementSnapshot<Id>>
where
    Id: Copy + Eq + Hash,
{
    let mut snapshots = HashMap::<Id, ElementSnapshot<Id>>::new();
    for mutation in mutations {
        if let DomMutation::AttributeChanged {
            node,
            name,
            old_value,
        } = mutation
        {
            let snapshot = snapshots.entry(*node).or_insert_with(|| ElementSnapshot {
                node: *node,
                changed_attributes: Vec::new(),
                state_changed: false,
            });
            if !snapshot
                .changed_attributes
                .iter()
                .any(|attribute| attribute.name == *name)
            {
                snapshot.changed_attributes.push(AttributeSnapshot {
                    name: name.clone(),
                    old_value: old_value.clone(),
                });
            }
        }
    }
    for node in state_nodes {
        snapshots
            .entry(*node)
            .and_modify(|snapshot| snapshot.state_changed = true)
            .or_insert_with(|| ElementSnapshot {
                node: *node,
                changed_attributes: Vec::new(),
                state_changed: true,
            });
    }
    snapshots.into_values().collect()
}

fn push_element_hint<D>(
    dom: &D,
    roots: &mut Vec<D::NodeId>,
    node: D::NodeId,
    sibling_dependencies: bool,
) where
    D: LayoutDom,
{
    if !dom.is_live(node) {
        return;
    }
    let root = if sibling_dependencies {
        dom.parent(node).unwrap_or(node)
    } else {
        node
    };
    push_root(dom, roots, root);
}

/// Semantic owners affected by a control or option's validity change. The
/// caller decides how each owner maps to its invalidation plane.
fn for_each_validity_dependent<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    mut affected: impl FnMut(D::NodeId),
) {
    if !dom.is_live(node) {
        return;
    }
    let mut parent = dom.parent(node);
    while let Some(id) = parent {
        if dom.element_name(id).is_some_and(|name| {
            name.ns.as_str() == "http://www.w3.org/1999/xhtml"
                && (name.local.as_str().eq_ignore_ascii_case("fieldset")
                    || name.local.as_str().eq_ignore_ascii_case("select"))
        }) {
            affected(id);
            if let Some(form) = dom.form_control_form_owner(id) {
                affected(form);
            }
        }
        parent = dom.parent(id);
    }
    if let Some(form) = dom.form_control_form_owner(node) {
        affected(form);
    }
}

/// Option membership and tree position can change a select's value-missing
/// state without changing any selectedness. Include the mutation boundary so
/// direct select-child changes reach its external form owner too.
fn for_each_select_validity_dependent<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    mut affected: impl FnMut(D::NodeId),
) {
    if !dom.is_live(node) {
        return;
    }
    let mut current = Some(node);
    while let Some(id) = current {
        if dom.element_name(id).is_some_and(|name| {
            name.ns.as_str() == "http://www.w3.org/1999/xhtml"
                && name.local.as_str().eq_ignore_ascii_case("select")
        }) {
            affected(id);
            if let Some(form) = dom.form_control_form_owner(id) {
                affected(form);
            }
        }
        current = dom.parent(id);
    }
}

fn is_radio_control<D: LayoutDom>(dom: &D, node: D::NodeId) -> bool {
    if !dom.is_live(node) {
        return false;
    }
    dom.element_name(node).is_some_and(|name| {
        name.ns.as_str() == "http://www.w3.org/1999/xhtml"
            && name.local.as_str().eq_ignore_ascii_case("input")
    }) && dom
        .attribute(
            node,
            &layout_dom_api::Namespace::from(""),
            &layout_dom_api::LocalName::from("type"),
        )
        .is_some_and(|kind| kind.eq_ignore_ascii_case("radio"))
}

fn radio_group_attribute_changed<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    name: &layout_dom_api::QualName,
    old_value: Option<&str>,
) -> bool {
    if !dom.is_live(node)
        || name.ns != layout_dom_api::Namespace::from("")
        || !dom.element_name(node).is_some_and(|element| {
            element.ns.as_str() == "http://www.w3.org/1999/xhtml"
                && element.local.as_str().eq_ignore_ascii_case("input")
        })
    {
        return false;
    }
    let current_radio = is_radio_control(dom, node);
    match name.local.as_ref() {
        "name" | "required" => current_radio,
        "type" => {
            current_radio || old_value.is_some_and(|value| value.eq_ignore_ascii_case("radio"))
        },
        _ => false,
    }
}

fn form_association_attribute_changed<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    name: &layout_dom_api::QualName,
) -> bool {
    if !dom.is_live(node) || name.ns != layout_dom_api::Namespace::from("") {
        return false;
    }
    let Some(element) = dom.element_name(node) else {
        return false;
    };
    match name.local.as_ref() {
        // Any earlier element with this ID can block a later form from owning
        // an explicitly-associated control, regardless of that element's tag.
        "id" => true,
        "form" => {
            element.ns.as_str() == "http://www.w3.org/1999/xhtml"
                && matches!(
                    element.local.as_str().to_ascii_lowercase().as_str(),
                    "button" | "fieldset" | "input" | "object" | "output" | "select" | "textarea"
                )
        },
        _ => false,
    }
}

/// Radio requiredness is shared by the whole same-name/form-owner group. The
/// selector owner has no reverse group index, so widen to this radio's tree.
fn radio_tree_root<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<D::NodeId> {
    if !dom.is_live(node) {
        return None;
    }
    let mut root = node;
    while let Some(parent) = dom.parent(root) {
        root = parent;
    }
    Some(root)
}

/// Aggregate validity depends on descendant membership and explicit form
/// owners, including controls outside a form subtree. Radio validity also
/// depends on same-name group membership. Widen only DOM trees containing one
/// of those dependencies, so ordinary documents retain the existing scope.
fn form_validity_tree_root<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<D::NodeId> {
    if !dom.is_live(node) {
        return None;
    }
    let mut root = node;
    while let Some(parent) = dom.parent(root) {
        root = parent;
    }
    let mut pending = vec![root];
    let mut contains_aggregate = false;
    while let Some(current) = pending.pop() {
        if dom.element_name(current).is_some_and(|name| {
            name.ns.as_str() == "http://www.w3.org/1999/xhtml"
                && (name.local.as_str().eq_ignore_ascii_case("form")
                    || name.local.as_str().eq_ignore_ascii_case("fieldset"))
        }) || is_radio_control(dom, current)
        {
            contains_aggregate = true;
            break;
        }
        pending.extend(dom.dom_children(current));
    }
    if contains_aggregate { Some(root) } else { None }
}

fn push_root<D>(dom: &D, roots: &mut Vec<D::NodeId>, root: D::NodeId)
where
    D: LayoutDom,
{
    if dom.is_live(root) {
        roots.push(root);
    }
}

// Coalesce the complete candidate set once. Checking each root against all
// earlier roots makes a batch of moving siblings quadratic. An ancestor walk
// gives the same disjoint coverage and preserves first-occurrence order.
fn coalesce_roots<D>(dom: &D, roots: &mut Vec<D::NodeId>)
where
    D: LayoutDom,
{
    let candidates: HashSet<_> = roots.iter().copied().collect();
    let mut seen = HashSet::new();
    roots.retain(|node| {
        if !seen.insert(*node) {
            return false;
        }
        let mut ancestor = dom.parent(*node);
        while let Some(parent) = ancestor {
            if candidates.contains(&parent) {
                return false;
            }
            ancestor = dom.parent(parent);
        }
        true
    });
}

fn inherited_parent<D>(
    dom: &D,
    plane: &StylePlane<D::NodeId>,
    root: D::NodeId,
) -> (Option<ComputedValues>, Option<CustomProperties>)
where
    D: LayoutDom,
{
    let mut parent = dom.parent(root);
    while let Some(id) = parent {
        if dom.kind(id) == NodeKind::Element {
            return (plane.get(id).cloned(), plane.custom_properties(id).cloned());
        }
        parent = dom.parent(id);
    }
    (None, None)
}

fn remove_subtree<D>(dom: &D, plane: &mut StylePlane<D::NodeId>, root: D::NodeId)
where
    D: LayoutDom,
{
    if dom.is_live(root) {
        for child in dom.flat_children(root) {
            remove_subtree(dom, plane, child);
        }
    }
    plane.remove(root);
}
