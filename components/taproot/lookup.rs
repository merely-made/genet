// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Borrowed views of owner-computed semantics. No retained catalogue or geometry.
use super::*;
use document_session_api::DocumentA11yProjection;

/// A generated semantic identity is never a fabricated DOM node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticTarget {
    Dom(NodeId),
    Contributed(u64),
}

/// Borrowed selector facts from one current observation, in publication order.
pub struct SemanticNode<'a> {
    pub target: SemanticTarget,
    pub role: Option<&'static str>,
    pub name: Option<&'a str>,
    pub hidden: bool,
}

/// A read of the owner's complete observation, including composed descendants.
/// Implementations must preserve tree order and inherited hidden state.
pub trait SemanticLookup {
    fn nodes(&self) -> Vec<SemanticNode<'_>>;
}

/// Selector matching shared by DOM and contributed observations. Class filters
/// retain their shallow DOM meaning. Attribute filters require a real DOM node;
/// generated children do not inherit arbitrary attributes from a producer slot.
pub fn matching_with_lookup(
    dom: &ScriptedDom,
    sel: &Selector,
    lookup: &impl SemanticLookup,
) -> Vec<SemanticTarget> {
    matching_nodes(dom, sel, lookup, false)
}

/// Apply exact-name preference after every role/hidden/attribute filter.
pub fn matching_prefer_exact_with_lookup(
    dom: &ScriptedDom,
    sel: &Selector,
    lookup: &impl SemanticLookup,
) -> Vec<SemanticTarget> {
    matching_nodes(dom, sel, lookup, true)
}

fn matching_nodes(
    dom: &ScriptedDom,
    sel: &Selector,
    lookup: &impl SemanticLookup,
    prefer_exact: bool,
) -> Vec<SemanticTarget> {
    let Match::Role(role) = &sel.matcher else {
        return matching_classes(dom, sel)
            .into_iter()
            .map(SemanticTarget::Dom)
            .collect();
    };
    let nodes = lookup.nodes();
    let matched: Vec<_> = nodes
        .iter()
        .filter(|node| {
            !node.hidden
                && node.role.is_some_and(|token| {
                    token == role
                        || matches!(
                            (token, role.as_str()),
                            ("img", "image") | ("cell", "gridcell")
                        )
                })
                && sel
                    .text
                    .as_ref()
                    .is_none_or(|text| node.name.is_some_and(|name| name.contains(text)))
                && sel
                    .attr
                    .as_ref()
                    .is_none_or(|(name, value)| match node.target {
                        SemanticTarget::Dom(node) => {
                            attr(dom, node, name).is_some_and(|s| s.contains(value))
                        },
                        SemanticTarget::Contributed(_) => false,
                    })
        })
        .collect();
    let exact = prefer_exact
        && sel.text.as_ref().is_some_and(|text| {
            !text.is_empty() && matched.iter().any(|node| node.name == Some(text.as_str()))
        });
    matched
        .into_iter()
        .filter(|node| !exact || node.name == sel.text.as_deref())
        .map(|node| node.target)
        .collect()
}

pub struct DocumentLookup<'a> {
    dom: &'a ScriptedDom,
    projection: &'a DocumentA11yProjection,
}
impl<'a> DocumentLookup<'a> {
    pub fn new(dom: &'a ScriptedDom, projection: &'a DocumentA11yProjection) -> Self {
        Self { dom, projection }
    }
}
impl SemanticLookup for DocumentLookup<'_> {
    fn nodes(&self) -> Vec<SemanticNode<'_>> {
        let nodes: BTreeMap<_, _> = self
            .projection
            .nodes()
            .iter()
            .map(|n| (n.id.get(), n))
            .collect();
        let mut pending = vec![self.dom.document()];
        let mut result = Vec::new();
        while let Some(node) = pending.pop() {
            if let Some(semantic) = nodes.get(&self.dom.opaque_id(node)) {
                result.push(SemanticNode {
                    target: SemanticTarget::Dom(node),
                    role: super::projection::role_token(semantic.role),
                    name: semantic.name.as_deref(),
                    hidden: semantic.state.hidden,
                });
            }
            let children: Vec<_> = self.dom.flat_children(node).collect();
            pending.extend(children.into_iter().rev());
        }
        result
    }
}

/// View of a complete composed tree, not an AccessKit incremental update.
/// The owner supplies the DOM map explicitly; numeric ID shape is not evidence
/// that a semantic object belongs to the DOM. No action route is manufactured.
#[cfg(feature = "accesskit")]
pub struct AccessKitLookup<'a, F> {
    tree: &'a accesskit::TreeUpdate,
    dom_target: F,
}
#[cfg(feature = "accesskit")]
impl<'a, F: Fn(u64) -> Option<NodeId>> AccessKitLookup<'a, F> {
    pub fn new(tree: &'a accesskit::TreeUpdate, dom_target: F) -> Self {
        Self { tree, dom_target }
    }
}
#[cfg(feature = "accesskit")]
impl<F: Fn(u64) -> Option<NodeId>> SemanticLookup for AccessKitLookup<'_, F> {
    fn nodes(&self) -> Vec<SemanticNode<'_>> {
        let nodes: BTreeMap<_, _> = self.tree.nodes.iter().map(|(id, n)| (*id, n)).collect();
        let mut pending: Vec<_> = self.tree.tree.iter().map(|tree| tree.root).collect();
        let mut seen = std::collections::BTreeSet::new();
        let mut result = Vec::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(node) = nodes.get(&id) else {
                continue;
            };
            if node.is_hidden() {
                continue;
            }
            result.push(SemanticNode {
                target: (self.dom_target)(id.0)
                    .map_or(SemanticTarget::Contributed(id.0), SemanticTarget::Dom),
                role: document_session_api::a11y::accesskit_role_token(node.role()),
                name: node.label(),
                hidden: false,
            });
            pending.extend(node.children().iter().rev().copied());
        }
        result
    }
}
