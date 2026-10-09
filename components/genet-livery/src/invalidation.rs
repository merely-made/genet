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
            match mutation {
                DomMutation::AttributeChanged {
                    node,
                    name,
                    old_value,
                } => {
                    push_element_hint(dom, &mut roots, *node, sibling_dependencies);
                    push_validity_dependents(dom, &mut roots, *node);
                    // Ownership changes affect controls outside the target's
                    // subtree, and the former form owner is no longer
                    // derivable after the attribute mutation.
                    if form_association_attribute_changed(dom, *node, name) {
                        push_form_validity_tree(dom, &mut roots, *node);
                    }
                    if radio_group_attribute_changed(dom, *node, name, old_value.as_deref()) {
                        push_radio_tree(dom, &mut roots, *node);
                    }
                },
                DomMutation::FormControlStateChanged { node } => {
                    push_element_hint(dom, &mut roots, *node, sibling_dependencies);
                    push_validity_dependents(dom, &mut roots, *node);
                    if is_radio_control(dom, *node) {
                        push_radio_tree(dom, &mut roots, *node);
                    }
                },
                DomMutation::FormControlInteractionStateChanged { node }
                | DomMutation::FormControlCustomValidityChanged { node }
                | DomMutation::OptionStateChanged { node } => {
                    push_element_hint(dom, &mut roots, *node, sibling_dependencies);
                    push_validity_dependents(dom, &mut roots, *node);
                },
                DomMutation::Inserted { node, parent } => {
                    push_form_validity_tree(dom, &mut roots, *parent);
                    push_select_validity_dependents(dom, &mut roots, *parent);
                    if structural_dependencies {
                        push_element_hint(dom, &mut roots, *parent, sibling_dependencies);
                    } else {
                        push_root(dom, &mut roots, *node);
                    }
                },
                DomMutation::Removed {
                    node,
                    former_parent,
                } => {
                    remove_subtree(dom, &mut self.plane, *node);
                    push_form_validity_tree(dom, &mut roots, *former_parent);
                    push_select_validity_dependents(dom, &mut roots, *former_parent);
                    if structural_dependencies {
                        push_element_hint(dom, &mut roots, *former_parent, sibling_dependencies);
                    }
                },
                DomMutation::CharacterDataChanged { node } => {
                    push_form_validity_tree(dom, &mut roots, *node);
                    push_select_validity_dependents(dom, &mut roots, *node);
                    if structural_dependencies
                        && dom.is_live(*node)
                        && let Some(parent) = dom.parent(*node)
                    {
                        push_element_hint(dom, &mut roots, parent, sibling_dependencies);
                    }
                },
                DomMutation::SubtreeReplaced { node } => {
                    push_form_validity_tree(dom, &mut roots, *node);
                    push_select_validity_dependents(dom, &mut roots, *node);
                    if structural_dependencies {
                        push_element_hint(dom, &mut roots, *node, sibling_dependencies);
                    } else {
                        push_root(dom, &mut roots, *node);
                    }
                },
                DomMutation::Moved {
                    node,
                    from_parent,
                    to_parent,
                } => {
                    push_form_validity_tree(dom, &mut roots, *from_parent);
                    push_form_validity_tree(dom, &mut roots, *to_parent);
                    push_select_validity_dependents(dom, &mut roots, *from_parent);
                    push_select_validity_dependents(dom, &mut roots, *to_parent);
                    if structural_dependencies {
                        push_element_hint(dom, &mut roots, *from_parent, sibling_dependencies);
                        push_element_hint(dom, &mut roots, *to_parent, sibling_dependencies);
                    } else {
                        push_root(dom, &mut roots, *node);
                    }
                },
            }
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

/// A control's validity can affect a fieldset ancestor and a form owner that
/// is elsewhere in the tree through `form="id"`. Option state can affect its
/// owning select, so the same ancestor walk covers both paths.
fn push_validity_dependents<D: LayoutDom>(dom: &D, roots: &mut Vec<D::NodeId>, node: D::NodeId) {
    if !dom.is_live(node) {
        return;
    }
    let mut parent = dom.parent(node);
    while let Some(id) = parent {
        if dom.element_name(id).is_some_and(|name| {
            name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && (name.local.as_ref().eq_ignore_ascii_case("fieldset")
                    || name.local.as_ref().eq_ignore_ascii_case("select"))
        }) {
            push_root(dom, roots, id);
            if let Some(form) = dom.form_control_form_owner(id) {
                push_root(dom, roots, form);
            }
        }
        parent = dom.parent(id);
    }
    if let Some(form) = dom.form_control_form_owner(node) {
        push_root(dom, roots, form);
    }
}

/// Option membership and tree position can change a select's value-missing
/// state without changing any selectedness. Walk from the mutation boundary
/// itself so direct select-child changes invalidate the select as well as its
/// external form owner.
fn push_select_validity_dependents<D: LayoutDom>(
    dom: &D,
    roots: &mut Vec<D::NodeId>,
    node: D::NodeId,
) {
    if !dom.is_live(node) {
        return;
    }
    let mut current = Some(node);
    while let Some(id) = current {
        if dom.element_name(id).is_some_and(|name| {
            name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && name.local.as_ref().eq_ignore_ascii_case("select")
        }) {
            push_root(dom, roots, id);
            if let Some(form) = dom.form_control_form_owner(id) {
                push_root(dom, roots, form);
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
        name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
            && name.local.as_ref().eq_ignore_ascii_case("input")
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
            element.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && element.local.as_ref().eq_ignore_ascii_case("input")
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
            element.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && matches!(
                    element.local.as_ref().to_ascii_lowercase().as_str(),
                    "button" | "fieldset" | "input" | "object" | "output" | "select" | "textarea"
                )
        },
        _ => false,
    }
}

/// Radio requiredness is shared by the whole same-name/form-owner group, so
/// changing one radio can alter validity in a different branch. The selector
/// owner has no reverse group index; invalidate only this radio's DOM tree.
fn push_radio_tree<D: LayoutDom>(dom: &D, roots: &mut Vec<D::NodeId>, node: D::NodeId) {
    if !dom.is_live(node) {
        return;
    }
    let mut root = node;
    while let Some(parent) = dom.parent(root) {
        root = parent;
    }
    push_root(dom, roots, root);
}

/// Aggregate validity depends on descendant membership and explicit form
/// owners, including controls outside a form subtree. Radio validity also
/// depends on same-name group membership. Widen only DOM trees containing one
/// of those dependencies, so ordinary documents retain the existing scope.
fn push_form_validity_tree<D: LayoutDom>(dom: &D, roots: &mut Vec<D::NodeId>, node: D::NodeId) {
    if !dom.is_live(node) {
        return;
    }
    let mut root = node;
    while let Some(parent) = dom.parent(root) {
        root = parent;
    }
    let mut pending = vec![root];
    let mut contains_aggregate = false;
    while let Some(current) = pending.pop() {
        if dom.element_name(current).is_some_and(|name| {
            name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && (name.local.as_ref().eq_ignore_ascii_case("form")
                    || name.local.as_ref().eq_ignore_ascii_case("fieldset"))
        }) || is_radio_control(dom, current)
        {
            contains_aggregate = true;
            break;
        }
        pending.extend(dom.dom_children(current));
    }
    if contains_aggregate {
        push_root(dom, roots, root);
    }
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
