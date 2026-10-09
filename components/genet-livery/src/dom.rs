// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use layout_dom_api::{LayoutDom, NodeKind};
use livery::selector::{
    AttrOperation, CaseSensitivity, Element, NamespaceConstraint, StatePseudoClass,
};

/// Host-supplied dynamic pseudo-class state for one document.
pub struct InteractionStates<Id> {
    states: HashMap<Id, HashSet<StatePseudoClass>>,
    generation: u64,
    changed: HashMap<Id, u64>,
}

impl<Id> Default for InteractionStates<Id> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            generation: 0,
            changed: HashMap::new(),
        }
    }
}

impl<Id> InteractionStates<Id>
where
    Id: Copy + Eq + std::hash::Hash,
{
    pub fn set(&mut self, id: Id, state: StatePseudoClass, enabled: bool) -> bool {
        let changed = if enabled {
            self.states.entry(id).or_default().insert(state)
        } else if let Some(states) = self.states.get_mut(&id) {
            let changed = states.remove(&state);
            if states.is_empty() {
                self.states.remove(&id);
            }
            changed
        } else {
            false
        };
        if changed {
            self.generation = self.generation.saturating_add(1);
            self.changed.insert(id, self.generation);
        }
        changed
    }

    pub fn matches(&self, id: Id, state: StatePseudoClass) -> bool {
        self.states
            .get(&id)
            .is_some_and(|states| states.contains(&state))
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn changed_since(&self, generation: u64) -> impl Iterator<Item = Id> + '_ {
        self.changed
            .iter()
            .filter_map(move |(id, changed)| (*changed > generation).then_some(*id))
    }
}

/// Stable identity and state storage used while matching one DOM.
pub struct SelectorTree<'a, D: LayoutDom> {
    dom: &'a D,
    identities: HashMap<D::NodeId, Box<u64>>,
    states: &'a InteractionStates<D::NodeId>,
}

impl<'a, D: LayoutDom> SelectorTree<'a, D> {
    pub fn new(dom: &'a D, states: &'a InteractionStates<D::NodeId>) -> Self {
        Self::for_roots(dom, states, &[dom.document()], true)
    }

    /// Build selector identity storage only for restyle roots, their
    /// descendants, ancestors, and sibling neighborhoods. Normal selector
    /// matching cannot escape that closure without `:has`, which this lane does
    /// not parse.
    pub(crate) fn for_roots(
        dom: &'a D,
        states: &'a InteractionStates<D::NodeId>,
        roots: &[D::NodeId],
        include_siblings: bool,
    ) -> Self {
        let mut included = HashSet::new();
        let mut pending = roots.to_vec();
        while let Some(id) = pending.pop() {
            if !dom.is_live(id) || !included.insert(id) {
                continue;
            }
            pending.extend(dom.dom_children(id));
            // A shadow tree is not among its host's children, so a descent that
            // only followed `dom_children` would mint no identity for anything
            // inside it and every selector would silently skip that subtree.
            if let Some(shadow) = dom.shadow_root(id) {
                pending.push(shadow);
            }
        }
        for root in roots {
            if !dom.is_live(*root) {
                continue;
            }
            let mut ancestor = ascend(dom, *root);
            while let Some(id) = ancestor {
                included.insert(id);
                ancestor = ascend(dom, id);
            }
        }
        if include_siblings {
            let neighborhood: Vec<_> = included.iter().copied().collect();
            let mut parents = HashSet::new();
            for id in neighborhood {
                if let Some(parent) = dom.parent(id)
                    && parents.insert(parent)
                {
                    included.extend(dom.dom_children(parent));
                }
            }
        }
        let identities = included
            .into_iter()
            .filter(|id| dom.kind(*id) == NodeKind::Element)
            .map(|id| (id, Box::new(dom.opaque_id(id))))
            .collect();
        Self {
            dom,
            identities,
            states,
        }
    }

    pub fn dom(&self) -> &'a D {
        self.dom
    }

    pub(crate) fn len(&self) -> usize {
        self.identities.len()
    }

    pub fn element(&self, id: D::NodeId) -> Option<ElementRef<'_, 'a, D>> {
        (self.dom.kind(id) == NodeKind::Element).then_some(ElementRef { tree: self, id })
    }
}

/// One step up the shadow-including tree: the parent, or — at the top of a
/// shadow tree — the host, so the identity closure a restyle root needs covers
/// the outer document too.
fn ascend<D: LayoutDom>(dom: &D, id: D::NodeId) -> Option<D::NodeId> {
    dom.parent(id).or_else(|| dom.shadow_host(id))
}

/// A selector-facing element reference over a neutral Genet DOM.
pub struct ElementRef<'tree, 'dom, D: LayoutDom> {
    tree: &'tree SelectorTree<'dom, D>,
    id: D::NodeId,
}

impl<D: LayoutDom> Copy for ElementRef<'_, '_, D> {}

impl<D: LayoutDom> Clone for ElementRef<'_, '_, D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: LayoutDom> fmt::Debug for ElementRef<'_, '_, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ElementRef").field(&self.id).finish()
    }
}

impl<'tree, 'dom, D: LayoutDom> ElementRef<'tree, 'dom, D> {
    pub fn id(self) -> D::NodeId {
        self.id
    }

    fn dom(self) -> &'dom D {
        self.tree.dom
    }

    fn attribute(self, namespace: &str, local: &str) -> Option<&'dom str> {
        self.dom().attributes(self.id).find_map(|attribute| {
            (attribute.name.ns.as_ref() == namespace && attribute.name.local.as_ref() == local)
                .then_some(attribute.value)
        })
    }

    fn sibling_element(self, previous: bool) -> Option<Self> {
        let mut sibling = if previous {
            self.dom().prev_sibling(self.id)
        } else {
            self.dom().next_sibling(self.id)
        };
        while let Some(id) = sibling {
            if let Some(element) = self.tree.element(id) {
                return Some(element);
            }
            sibling = if previous {
                self.dom().prev_sibling(id)
            } else {
                self.dom().next_sibling(id)
            };
        }
        None
    }
}

impl<D: LayoutDom> Element for ElementRef<'_, '_, D> {
    type PseudoClass = StatePseudoClass;

    fn is_same(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn parent_element(&self) -> Option<Self> {
        let mut parent = self.dom().parent(self.id);
        while let Some(id) = parent {
            if let Some(element) = self.tree.element(id) {
                return Some(element);
            }
            parent = self.dom().parent(id);
        }
        None
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        self.dom()
            .parent(self.id)
            .is_some_and(|parent| self.dom().kind(parent) == NodeKind::ShadowRoot)
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        let root = self.dom().containing_shadow_root(self.id)?;
        let host = self.dom().shadow_host(root)?;
        self.tree.element(host)
    }

    /// The slot this element is assigned to (`::slotted()`'s jump), walked to
    /// decide whether a `::slotted` rule from a shadow tree reaches a
    /// light-DOM element.
    fn assigned_slot(&self) -> Option<Self> {
        let slot = self.dom().assigned_slot(self.id)?;
        self.tree.element(slot)
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.sibling_element(true)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.sibling_element(false)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.dom()
            .element_name(self.id)
            .is_some_and(|name| name.ns.as_ref() == "http://www.w3.org/1999/xhtml")
    }

    fn has_local_name(&self, local_name: &str) -> bool {
        self.dom()
            .element_name(self.id)
            .is_some_and(|name| name.local.as_ref().eq_ignore_ascii_case(local_name))
    }

    fn has_namespace(&self, namespace: &str) -> bool {
        self.dom()
            .element_name(self.id)
            .is_some_and(|name| name.ns.as_ref() == namespace)
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.dom().element_name(self.id) == other.dom().element_name(other.id)
    }

    fn attr_matches(
        &self,
        namespace: &NamespaceConstraint<'_>,
        local_name: &str,
        operation: &AttrOperation<'_>,
    ) -> bool {
        self.dom().attributes(self.id).any(|attribute| {
            let namespace_matches = match namespace {
                NamespaceConstraint::Any => true,
                NamespaceConstraint::Specific(namespace) => {
                    attribute.name.ns.as_ref() == *namespace
                },
            };
            namespace_matches
                && attribute.name.local.as_ref() == local_name
                && operation.eval_str(attribute.value)
        })
    }

    fn matches_pseudo_class(&self, pseudo: &StatePseudoClass) -> bool {
        if *pseudo == StatePseudoClass::Checked
            && self.dom().element_name(self.id).is_some_and(|name| {
                name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                    && name.local.as_ref().eq_ignore_ascii_case("input")
            })
            && self.attribute("", "type").is_some_and(|kind| {
                matches!(kind.to_ascii_lowercase().as_str(), "checkbox" | "radio")
            })
        {
            return self.dom().form_control_state(self.id).map_or_else(
                || self.attribute("", "checked").is_some(),
                |state| state.checked,
            );
        }
        if matches!(
            pseudo,
            StatePseudoClass::Valid
                | StatePseudoClass::Invalid
                | StatePseudoClass::UserValid
                | StatePseudoClass::UserInvalid
        ) {
            let Some(qname) = self.dom().element_name(self.id) else {
                return false;
            };
            if qname.ns.as_ref() != "http://www.w3.org/1999/xhtml" {
                return false;
            }
            let name = qname.local.as_ref().to_ascii_lowercase();
            let user = matches!(
                pseudo,
                StatePseudoClass::UserValid | StatePseudoClass::UserInvalid
            );
            let (candidate, valid, user_validity) = if name == "form" || name == "fieldset" {
                if user {
                    return false;
                }
                let mut nodes = Vec::new();
                collect_descendants(
                    self.dom(),
                    if name == "form" {
                        tree_root(self.dom(), self.id)
                    } else {
                        self.id
                    },
                    &mut nodes,
                );
                let mut invalid = false;
                let mut complete = true;
                for node in nodes {
                    let Some(control_name) = self
                        .dom()
                        .element_name(node)
                        .map(|name| name.local.as_ref().to_ascii_lowercase())
                    else {
                        continue;
                    };
                    let status = self.dom().form_control_validity(node);
                    let belongs = if name == "form" {
                        associated_with_form(self.dom(), node, self.id, status.is_some())
                    } else {
                        true
                    };
                    let Some(status) = status else {
                        if belongs
                            && matches!(
                                control_name.as_str(),
                                "input" | "textarea" | "select" | "button" | "fieldset" | "object"
                            )
                        {
                            complete = false;
                        }
                        continue;
                    };
                    let relevant = if name == "form" {
                        belongs
                    } else {
                        status.will_validate
                    };
                    if relevant && status.will_validate && !status.flags.valid() {
                        invalid = true;
                        break;
                    }
                }
                (complete || invalid, !invalid, false)
            } else {
                let Some(status) = self.dom().form_control_validity(self.id) else {
                    return false;
                };
                if user && !matches!(name.as_str(), "input" | "textarea" | "select") {
                    return false;
                }
                (
                    status.will_validate,
                    status.flags.valid(),
                    status.user_validity,
                )
            };
            return match pseudo {
                StatePseudoClass::Valid => candidate && valid,
                StatePseudoClass::Invalid => candidate && !valid,
                StatePseudoClass::UserValid => candidate && user_validity && valid,
                StatePseudoClass::UserInvalid => candidate && user_validity && !valid,
                _ => false,
            };
        }
        self.tree.states.matches(self.id, *pseudo)
    }

    fn is_slot(&self) -> bool {
        self.has_local_name("slot")
    }

    fn has_id(&self, id: &str, case_sensitivity: CaseSensitivity) -> bool {
        self.attribute("", "id")
            .is_some_and(|value| case_sensitivity.eq(value, id))
    }

    fn has_class(&self, class: &str, case_sensitivity: CaseSensitivity) -> bool {
        self.attribute("", "class").is_some_and(|classes| {
            classes
                .split_ascii_whitespace()
                .any(|value| case_sensitivity.eq(value, class))
        })
    }

    /// `exportparts` on this host, read outer to inner: the name a part has
    /// inside this host's shadow tree, given the `outer_name` it is exposed
    /// under. Parsed per read; the attribute is short and only consulted while
    /// a `::part()` selector is climbing shadow boundaries.
    fn imported_part(&self, outer_name: &str) -> Option<String> {
        self.attribute("", "exportparts")?
            .split(',')
            .find_map(|entry| {
                let entry = entry.trim();
                let (inner, outer) = match entry.split_once(':') {
                    Some((inner, outer)) => (inner.trim(), outer.trim()),
                    None => (entry, entry),
                };
                (outer == outer_name).then(|| inner.to_string())
            })
    }

    /// Whether this element carries part `name` in its `part` attribute — the
    /// whitespace-separated token list `::part()` matches against.
    fn is_part(&self, name: &str) -> bool {
        self.attribute("", "part")
            .is_some_and(|parts| parts.split_ascii_whitespace().any(|part| part == name))
    }

    fn is_empty(&self) -> bool {
        self.dom()
            .dom_children(self.id)
            .all(|id| match self.dom().kind(id) {
                NodeKind::Element => false,
                NodeKind::Text => self.dom().text(id).is_none_or(str::is_empty),
                _ => true,
            })
    }

    fn is_root(&self) -> bool {
        self.parent_element().is_none()
    }
}

fn collect_descendants<D: LayoutDom>(dom: &D, node: D::NodeId, out: &mut Vec<D::NodeId>) {
    for child in dom.dom_children(node) {
        out.push(child);
        collect_descendants(dom, child, out);
    }
}

fn tree_root<D: LayoutDom>(dom: &D, mut node: D::NodeId) -> D::NodeId {
    while let Some(parent) = dom.parent(node) {
        node = parent;
    }
    node
}

fn associated_with_form<D: LayoutDom>(
    dom: &D,
    control: D::NodeId,
    form: D::NodeId,
    native_validity: bool,
) -> bool {
    let owner = dom.form_control_form_owner(control);
    if owner.is_some() || native_validity {
        // A supported native view also authoritatively reports no owner.
        // Reconstructing from attributes would bypass first-ID lookup rules.
        return owner == Some(form);
    }
    let namespace = layout_dom_api::Namespace::from("");
    let local = layout_dom_api::LocalName::from("form");
    if let Some(owner_id) = dom.attribute(control, &namespace, &local) {
        return dom.attribute(form, &namespace, &layout_dom_api::LocalName::from("id"))
            == Some(owner_id);
    }
    let mut ancestor = dom.parent(control);
    while let Some(node) = ancestor {
        if dom.element_name(node).is_some_and(|name| {
            name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && name.local.as_ref().eq_ignore_ascii_case("form")
        }) {
            return node == form;
        }
        ancestor = dom.parent(node);
    }
    false
}
