// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Live-DOM adapter for Cadency's shared selector grammar. Queries walk DOM
//! children, not flat-tree or shadow-root children.

use cadency::{AttrOperation, CaseSensitivity, Element, NamespaceConstraint, PseudoClass, SelectorList};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidityPseudo {
    Valid,
    Invalid,
    UserValid,
    UserInvalid,
}

impl PseudoClass for ValidityPseudo {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "valid" => Self::Valid,
            "invalid" => Self::Invalid,
            "user-valid" => Self::UserValid,
            "user-invalid" => Self::UserInvalid,
            _ => return None,
        })
    }
}

pub type Selectors = SelectorList<ValidityPseudo>;

pub fn parse(input: &str) -> Result<Selectors, cadency::ParseError> {
    Selectors::parse(input)
}

/// One query uses the receiver as its explicit `:scope` element. A Document
/// supplies its document element; a fragment has no element scoping root.
fn scope_element<D: LayoutDom>(dom: &D, scope: D::NodeId) -> Option<D::NodeId> {
    match dom.kind(scope) {
        NodeKind::Element => Some(scope),
        NodeKind::Document => dom.dom_children(scope).find(|&id| dom.element_name(id).is_some()),
        _ => None,
    }
}

pub fn matches<D: LayoutDom>(selectors: &Selectors, dom: &D, node: D::NodeId, html_document: bool) -> bool {
    let element = DomElement { dom, id: node, html_document };
    selectors.matches_scoped(&element, None, Some(&element))
}

/// One preorder traversal and one test per element. The returned IDs describe
/// static membership; the host materializes canonical reflectors before JS runs.
pub fn query_all<D: LayoutDom>(
    selectors: &Selectors,
    dom: &D,
    scope: D::NodeId,
    html_document: bool,
) -> Vec<D::NodeId> {
    let scope_element = scope_element(dom, scope).map(|id| DomElement { dom, id, html_document });
    let mut out = Vec::new();
    let mut stack: Vec<_> = dom.dom_children(scope).collect();
    stack.reverse();
    while let Some(id) = stack.pop() {
        if dom.element_name(id).is_some() {
            let element = DomElement { dom, id, html_document };
            if selectors.matches_scoped(&element, None, scope_element.as_ref()) {
                out.push(id);
            }
        }
        let children: Vec<_> = dom.dom_children(id).collect();
        stack.extend(children.into_iter().rev());
    }
    out
}

pub fn query_first<D: LayoutDom>(
    selectors: &Selectors,
    dom: &D,
    scope: D::NodeId,
    html_document: bool,
) -> Option<D::NodeId> {
    let scope_element = scope_element(dom, scope).map(|id| DomElement { dom, id, html_document });
    let mut stack: Vec<_> = dom.dom_children(scope).collect();
    stack.reverse();
    while let Some(id) = stack.pop() {
        if dom.element_name(id).is_some() {
            let element = DomElement { dom, id, html_document };
            if selectors.matches_scoped(&element, None, scope_element.as_ref()) {
                return Some(id);
            }
        }
        let children: Vec<_> = dom.dom_children(id).collect();
        stack.extend(children.into_iter().rev());
    }
    None
}

struct DomElement<'a, D: LayoutDom> {
    dom: &'a D,
    id: D::NodeId,
    html_document: bool,
}

impl<D: LayoutDom> Clone for DomElement<'_, D> {
    fn clone(&self) -> Self {
        Self { dom: self.dom, id: self.id, html_document: self.html_document }
    }
}

impl<D: LayoutDom> Element for DomElement<'_, D> {
    type PseudoClass = ValidityPseudo;

    fn is_same(&self, other: &Self) -> bool { self.id == other.id }
    fn parent_element(&self) -> Option<Self> {
        let id = self.dom.parent(self.id)?;
        self.dom.element_name(id).map(|_| Self { id, ..self.clone() })
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        let mut id = self.dom.prev_sibling(self.id);
        while let Some(sibling) = id {
            if self.dom.element_name(sibling).is_some() {
                return Some(Self { id: sibling, ..self.clone() });
            }
            id = self.dom.prev_sibling(sibling);
        }
        None
    }
    fn next_sibling_element(&self) -> Option<Self> {
        let mut id = self.dom.next_sibling(self.id);
        while let Some(sibling) = id {
            if self.dom.element_name(sibling).is_some() {
                return Some(Self { id: sibling, ..self.clone() });
            }
            id = self.dom.next_sibling(sibling);
        }
        None
    }
    fn is_root(&self) -> bool {
        self.dom.parent(self.id).is_none_or(|parent| self.dom.kind(parent) == NodeKind::Document)
    }
    fn is_empty(&self) -> bool {
        !self.dom.dom_children(self.id).any(|id| {
            self.dom.element_name(id).is_some()
                || matches!(self.dom.kind(id), NodeKind::Text | NodeKind::CdataSection)
                    && self.dom.text(id).is_some_and(|text| !text.is_empty())
        })
    }
    fn is_html_element_in_html_document(&self) -> bool {
        self.html_document && self.dom.element_name(self.id)
            .is_some_and(|name| name.ns.as_ref() == "http://www.w3.org/1999/xhtml")
    }
    fn has_local_name(&self, name: &str) -> bool {
        self.dom.element_name(self.id).is_some_and(|q| {
            if self.is_html_element_in_html_document() {
                q.local.as_ref().eq_ignore_ascii_case(name)
            } else {
                q.local.as_ref() == name
            }
        })
    }
    fn has_namespace(&self, url: &str) -> bool {
        self.dom.element_name(self.id).is_some_and(|q| q.ns.as_ref() == url)
    }
    fn is_same_type(&self, other: &Self) -> bool {
        match (self.dom.element_name(self.id), self.dom.element_name(other.id)) {
            (Some(a), Some(b)) => a.ns.as_ref() == b.ns.as_ref() && if self.is_html_element_in_html_document() {
                a.local.as_ref().eq_ignore_ascii_case(b.local.as_ref())
            } else {
                a.local.as_ref() == b.local.as_ref()
            },
            _ => false,
        }
    }
    fn has_id(&self, id: &str, case: CaseSensitivity) -> bool {
        self.dom.attribute(self.id, &Namespace::from(""), &LocalName::from("id"))
            .is_some_and(|value| case.eq(value, id))
    }
    fn has_class(&self, class: &str, case: CaseSensitivity) -> bool {
        self.dom.attribute(self.id, &Namespace::from(""), &LocalName::from("class"))
            .is_some_and(|value| value.split([' ', '\t', '\n', '\r', '\x0c']).any(|token| case.eq(token, class)))
    }
    fn attr_matches(
        &self,
        namespace: &NamespaceConstraint<'_>,
        local_name: &str,
        operation: &AttrOperation<'_>,
    ) -> bool {
        self.dom.attributes(self.id).any(|attr| {
            let name = attr.name;
            let namespace_matches = match namespace {
                NamespaceConstraint::Any => true,
                NamespaceConstraint::Specific(ns) => name.ns.as_ref() == *ns,
            };
            let local_matches = if self.is_html_element_in_html_document() && name.ns.as_ref().is_empty() {
                name.local.as_ref().eq_ignore_ascii_case(local_name)
            } else {
                name.local.as_ref() == local_name
            };
            namespace_matches && local_matches && operation.eval_str(attr.value)
        })
    }
    fn matches_pseudo_class(&self, pseudo: &Self::PseudoClass) -> bool {
        matches_validity_pseudo(self.dom, self.id, *pseudo)
    }
    fn parent_node_is_shadow_root(&self) -> bool {
        self.dom.parent(self.id).is_some_and(|parent| self.dom.kind(parent) == NodeKind::ShadowRoot)
    }
    fn containing_shadow_host(&self) -> Option<Self> {
        let root = self.dom.containing_shadow_root(self.id)?;
        let id = self.dom.shadow_host(root)?;
        Some(Self { id, ..self.clone() })
    }
}

fn collect_descendants<D: LayoutDom>(dom: &D, node: D::NodeId, out: &mut Vec<D::NodeId>) {
    for child in dom.dom_children(node) {
        out.push(child);
        collect_descendants(dom, child, out);
    }
}

fn matches_validity_pseudo<D: LayoutDom>(dom: &D, node: D::NodeId, pseudo: ValidityPseudo) -> bool {
    let Some(qname) = dom.element_name(node) else {
        return false;
    };
    if qname.ns.as_ref() != "http://www.w3.org/1999/xhtml" {
        return false;
    }
    let name = qname.local.as_ref().to_ascii_lowercase();
    let is_form = name == "form";
    let is_fieldset = name == "fieldset";
    let user = matches!(
        pseudo,
        ValidityPseudo::UserValid | ValidityPseudo::UserInvalid
    );
    let (candidate, valid, user_validity) = if is_form || is_fieldset {
        if user {
            return false;
        }
        let mut descendants = Vec::new();
        collect_descendants(
            dom,
            if is_form { tree_root(dom, node) } else { node },
            &mut descendants,
        );
        let mut invalid = false;
        let mut complete = true;
        for descendant in descendants {
            let Some(control_name) = dom
                .element_name(descendant)
                .map(|name| name.local.as_ref().to_ascii_lowercase())
            else {
                continue;
            };
            let validity = dom.form_control_validity(descendant);
            let belongs = if is_form {
                associated_with_form(dom, descendant, node, validity.is_some())
            } else {
                true
            };
            let Some(validity) = validity else {
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
            let relevant = if is_form {
                belongs
            } else {
                validity.will_validate
            };
            if relevant && validity.will_validate && !validity.flags.valid() {
                invalid = true;
                break;
            }
        }
        (complete || invalid, !invalid, false)
    } else {
        let Some(validity) = dom.form_control_validity(node) else {
            return false;
        };
        if user {
            if !matches!(name.as_str(), "input" | "textarea" | "select") {
                return false;
            }
        }
        (
            validity.will_validate,
            validity.flags.valid(),
            validity.user_validity,
        )
    };
    match pseudo {
        ValidityPseudo::Valid => candidate && valid,
        ValidityPseudo::Invalid => candidate && !valid,
        ValidityPseudo::UserValid => candidate && user_validity && valid,
        ValidityPseudo::UserInvalid => candidate && user_validity && !valid,
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
        // None is authoritative when native validation is implemented.
        return owner == Some(form);
    }
    let namespace = Namespace::from("");
    if let Some(owner_id) = dom.attribute(control, &namespace, &LocalName::from("form")) {
        return dom.attribute(form, &namespace, &LocalName::from("id")) == Some(owner_id);
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{AttributeView, LayoutDomMut, QualName};

    use super::*;

    struct CountingDom {
        dom: ScriptedDom,
        child_walks: Cell<usize>,
    }

    impl LayoutDom for CountingDom {
        type NodeId = NodeId;

        fn document(&self) -> NodeId { self.dom.document() }
        fn parent(&self, id: NodeId) -> Option<NodeId> { self.dom.parent(id) }
        fn prev_sibling(&self, id: NodeId) -> Option<NodeId> { self.dom.prev_sibling(id) }
        fn next_sibling(&self, id: NodeId) -> Option<NodeId> { self.dom.next_sibling(id) }
        fn dom_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
            self.child_walks.set(self.child_walks.get() + 1);
            self.dom.dom_children(id)
        }
        fn kind(&self, id: NodeId) -> NodeKind { self.dom.kind(id) }
        fn element_name(&self, id: NodeId) -> Option<&QualName> { self.dom.element_name(id) }
        fn attribute(&self, id: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
            self.dom.attribute(id, ns, local)
        }
        fn attributes(&self, id: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
            self.dom.attributes(id)
        }
        fn text(&self, id: NodeId) -> Option<&str> { self.dom.text(id) }
    }

    #[test]
    fn all_matching_query_walks_each_node_once() {
        let mut dom = ScriptedDom::new();
        let body = dom.create_element(QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from("body"),
        ));
        let document = dom.document();
        dom.append_child(document, body);
        dom.set_inner_html(body, &"<i></i>".repeat(500));
        let counted = CountingDom { dom, child_walks: Cell::new(0) };
        let selectors = parse("i").expect("selector");
        let matched = query_all(&selectors, &counted, body, true);
        assert_eq!(matched.len(), 500);
        // One body enumeration, then one child enumeration per matched node.
        assert_eq!(counted.child_walks.get(), 501);
    }
}
