// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `Document.evaluate()` native half: adapt the live `ScriptedDom` to the
//! generic XPath 1.0 engine, then serialize the result for the JS `XPathResult`
//! wrapper in `bootstrap.js`.

use crate::OwnerResolvedCx as _;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom, NodeKind};
use markup5ever::{LocalName, Namespace, Prefix};
use script_engine_api::{NativeFn, ScriptEngine};
use xpath::Node as _;

use super::*;

#[derive(Clone, Debug)]
struct XPathTree {
    nodes: Vec<XPathNodeData>,
    attrs: Vec<XPathAttrData>,
    root: XPathKind,
    html_document: bool,
}

#[derive(Clone, Debug)]
struct XPathNodeData {
    kind: NodeKind,
    name: Option<QualName>,
    attrs: Vec<usize>,
    text: Option<String>,
    parent: Option<usize>,
    children: Vec<usize>,
    raw: u64,
    order: usize,
}

#[derive(Clone, Debug)]
struct XPathAttrData {
    owner: Option<usize>,
    name: QualName,
    value: String,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum XPathKind {
    Node(usize),
    Attr(usize),
}

#[derive(Clone, Debug)]
struct XPathNode {
    tree: Rc<XPathTree>,
    kind: XPathKind,
}

#[derive(Clone, Debug)]
struct XPathElement {
    node: XPathNode,
}

#[derive(Clone, Debug)]
struct XPathAttr {
    node: XPathNode,
}

#[derive(Clone, Debug)]
struct XPathDocument {
    tree: Rc<XPathTree>,
}

#[derive(Clone)]
struct CollectNamespaces(Rc<RefCell<Vec<String>>>);

#[derive(Clone)]
struct ResolvedNamespaces(HashMap<String, String>);

struct XPathDom;

impl xpath::Dom for XPathDom {
    type Node = XPathNode;
    type NamespaceResolver = ResolvedNamespaces;
}

impl xpath::NamespaceResolver for CollectNamespaces {
    fn resolve_namespace_prefix(&self, prefix: &str) -> Option<String> {
        self.0.borrow_mut().push(prefix.to_owned());
        Some("urn:genet:xpath-prefix-probe".to_owned())
    }
}

impl xpath::NamespaceResolver for ResolvedNamespaces {
    fn resolve_namespace_prefix(&self, prefix: &str) -> Option<String> {
        self.0.get(prefix).cloned()
    }
}

impl PartialEq for XPathNode {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for XPathNode {}

impl XPathTree {
    fn from_dom(dom: &ScriptedDom, root: NodeId, html_document: bool) -> Rc<Self> {
        fn copy_node(
            dom: &ScriptedDom,
            id: NodeId,
            parent: Option<usize>,
            nodes: &mut Vec<XPathNodeData>,
            attrs: &mut Vec<XPathAttrData>,
            order: &mut usize,
        ) -> usize {
            let idx = nodes.len();
            let node_order = *order;
            *order += 1;
            let attr_ids = dom
                .attributes(id)
                .enumerate()
                .map(|(index, attr)| {
                    let attr_idx = attrs.len();
                    attrs.push(XPathAttrData {
                        owner: Some(idx),
                        name: attr.name.clone(),
                        value: attr.value.to_string(),
                        index,
                    });
                    attr_idx
                })
                .collect::<Vec<_>>();

            nodes.push(XPathNodeData {
                kind: dom.kind(id),
                name: dom.element_name(id).cloned(),
                attrs: attr_ids,
                text: dom.text(id).map(str::to_string),
                parent,
                children: Vec::new(),
                raw: id.raw(),
                order: node_order,
            });

            let children = dom
                .dom_children(id)
                .map(|child| copy_node(dom, child, Some(idx), nodes, attrs, order))
                .collect::<Vec<_>>();
            nodes[idx].children = children;
            idx
        }

        let mut nodes = Vec::new();
        let mut attrs = Vec::new();
        let mut order = 0;
        let root = copy_node(dom, root, None, &mut nodes, &mut attrs, &mut order);
        Rc::new(Self { nodes, attrs, root: XPathKind::Node(root), html_document })
    }

    fn detached_attr(name: QualName, value: String, html_document: bool) -> Rc<Self> {
        Rc::new(Self {
            nodes: Vec::new(),
            attrs: vec![XPathAttrData { owner: None, name, value, index: 0 }],
            root: XPathKind::Attr(0),
            html_document,
        })
    }

    fn node(self: &Rc<Self>, idx: usize) -> XPathNode {
        XPathNode {
            tree: self.clone(),
            kind: XPathKind::Node(idx),
        }
    }

    fn attr(self: &Rc<Self>, idx: usize) -> XPathNode {
        XPathNode {
            tree: self.clone(),
            kind: XPathKind::Attr(idx),
        }
    }
}

impl XPathNode {
    fn node_data(&self) -> Option<&XPathNodeData> {
        match self.kind {
            XPathKind::Node(idx) => self.tree.nodes.get(idx),
            XPathKind::Attr(_) => None,
        }
    }

    fn attr_data(&self) -> Option<&XPathAttrData> {
        match self.kind {
            XPathKind::Attr(idx) => self.tree.attrs.get(idx),
            XPathKind::Node(_) => None,
        }
    }

    fn order_key(&self) -> (usize, usize) {
        match self.kind {
            XPathKind::Node(idx) => (self.tree.nodes[idx].order * 2, 0),
            XPathKind::Attr(idx) => {
                let attr = &self.tree.attrs[idx];
                (attr.owner.map_or(0, |owner| self.tree.nodes[owner].order * 2 + 1), attr.index)
            },
        }
    }

    fn collect_text(&self, out: &mut String) {
        match self.kind {
            XPathKind::Attr(idx) => out.push_str(&self.tree.attrs[idx].value),
            XPathKind::Node(idx) => {
                if let Some(text) = &self.tree.nodes[idx].text {
                    out.push_str(text);
                }
                for &child in &self.tree.nodes[idx].children {
                    self.tree.node(child).collect_text(out);
                }
            },
        }
    }

    fn collect_preorder(&self, out: &mut Vec<XPathNode>) {
        out.push(self.clone());
        if let XPathKind::Node(idx) = self.kind {
            for &child in &self.tree.nodes[idx].children {
                self.tree.node(child).collect_preorder(out);
            }
        }
    }

    fn root(&self) -> XPathNode {
        match self.tree.root {
            XPathKind::Node(idx) => self.tree.node(idx),
            XPathKind::Attr(idx) => self.tree.attr(idx),
        }
    }

    fn is_ancestor_of(&self, other: &XPathNode) -> bool {
        let mut current = other.parent();
        while let Some(node) = current {
            if node == *self {
                return true;
            }
            current = node.parent();
        }
        false
    }
}

impl xpath::Node for XPathNode {
    type ProcessingInstruction = XPathNode;
    type Document = XPathDocument;
    type Attribute = XPathAttr;
    type Element = XPathElement;
    type Opaque = XPathKind;

    fn is_comment(&self) -> bool {
        self.node_data()
            .is_some_and(|d| d.kind == NodeKind::Comment)
    }

    fn is_text(&self) -> bool {
        self.node_data().is_some_and(|d| d.kind == NodeKind::Text)
    }

    fn text_content(&self) -> String {
        let mut out = String::new();
        self.collect_text(&mut out);
        out
    }

    fn language(&self) -> Option<String> {
        let mut current = Some(self.clone());
        while let Some(node) = current {
            if let XPathKind::Node(idx) = node.kind {
                for &attr in &node.tree.nodes[idx].attrs {
                    let attr = &node.tree.attrs[attr];
                    if attr.name.local.as_ref().eq_ignore_ascii_case("lang") {
                        return Some(attr.value.clone());
                    }
                }
            }
            current = node.parent();
        }
        None
    }

    fn parent(&self) -> Option<Self> {
        match self.kind {
            XPathKind::Node(idx) => self.tree.nodes[idx].parent.map(|p| self.tree.node(p)),
            XPathKind::Attr(idx) => self.tree.attrs[idx].owner.map(|owner| self.tree.node(owner)),
        }
    }

    fn children(&self) -> impl Iterator<Item = Self> {
        match self.kind {
            XPathKind::Node(idx) => self.tree.nodes[idx]
                .children
                .iter()
                .map(|&child| self.tree.node(child))
                .collect::<Vec<_>>()
                .into_iter(),
            XPathKind::Attr(_) => Vec::new().into_iter(),
        }
    }

    fn compare_tree_order(&self, other: &Self) -> Ordering {
        self.order_key().cmp(&other.order_key())
    }

    fn traverse_preorder(&self) -> impl Iterator<Item = Self> {
        let mut out = Vec::new();
        self.collect_preorder(&mut out);
        out.into_iter()
    }

    fn inclusive_ancestors(&self) -> impl Iterator<Item = Self> {
        let mut out = Vec::new();
        let mut current = Some(self.clone());
        while let Some(node) = current {
            current = node.parent();
            out.push(node);
        }
        out.into_iter()
    }

    fn preceding_nodes(&self) -> impl Iterator<Item = Self> {
        let mut nodes = self.root().traverse_preorder().collect::<Vec<_>>();
        nodes.retain(|node| {
            node == self || (node.compare_tree_order(self).is_lt() && !node.is_ancestor_of(self))
        });
        nodes.reverse();
        nodes.into_iter()
    }

    fn following_nodes(&self) -> impl Iterator<Item = Self> {
        let mut nodes = self.root().traverse_preorder().collect::<Vec<_>>();
        nodes.retain(|node| {
            node == self || (node.compare_tree_order(self).is_gt() && !self.is_ancestor_of(node))
        });
        nodes.into_iter()
    }

    fn preceding_siblings(&self) -> impl Iterator<Item = Self> {
        let Some(parent) = self.parent() else {
            return Vec::new().into_iter();
        };
        let mut out = Vec::new();
        for child in parent.children() {
            if child == *self {
                break;
            }
            out.push(child);
        }
        out.into_iter()
    }

    fn following_siblings(&self) -> impl Iterator<Item = Self> {
        let Some(parent) = self.parent() else {
            return Vec::new().into_iter();
        };
        let mut seen_self = false;
        let mut out = Vec::new();
        for child in parent.children() {
            if seen_self {
                out.push(child);
            } else if child == *self {
                seen_self = true;
            }
        }
        out.into_iter()
    }

    fn owner_document(&self) -> Self::Document {
        XPathDocument {
            tree: self.tree.clone(),
        }
    }

    fn to_opaque(&self) -> Self::Opaque {
        self.kind
    }

    fn as_processing_instruction(&self) -> Option<Self::ProcessingInstruction> {
        self.node_data()
            .is_some_and(|d| d.kind == NodeKind::ProcessingInstruction)
            .then(|| self.clone())
    }

    fn as_attribute(&self) -> Option<Self::Attribute> {
        matches!(self.kind, XPathKind::Attr(_)).then(|| XPathAttr { node: self.clone() })
    }

    fn as_element(&self) -> Option<Self::Element> {
        self.node_data()
            .is_some_and(|d| d.kind == NodeKind::Element)
            .then(|| XPathElement { node: self.clone() })
    }

    fn get_root_node(&self) -> Self {
        self.root()
    }
}

impl xpath::ProcessingInstruction for XPathNode {
    fn target(&self) -> String {
        String::new()
    }
}

impl xpath::Document for XPathDocument {
    type Node = XPathNode;

    fn get_elements_with_id(
        &self,
        id: &str,
    ) -> impl Iterator<Item = <Self::Node as xpath::Node>::Element> {
        let root = match self.tree.root {
            XPathKind::Node(idx) => self.tree.node(idx),
            XPathKind::Attr(idx) => self.tree.attr(idx),
        };
        root
            .traverse_preorder()
            .filter_map(|node| {
                let XPathKind::Node(idx) = node.kind else {
                    return None;
                };
                let has_id = self.tree.nodes[idx].attrs.iter().any(|&attr| {
                    let attr = &self.tree.attrs[attr];
                    attr.name.local.as_ref() == "id" && attr.value == id
                });
                (has_id && self.tree.nodes[idx].kind == NodeKind::Element)
                    .then(|| XPathElement { node })
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl xpath::Element for XPathElement {
    type Node = XPathNode;
    type Attribute = XPathAttr;

    fn as_node(&self) -> Self::Node {
        self.node.clone()
    }

    fn prefix(&self) -> Option<Prefix> {
        self.node
            .node_data()
            .and_then(|d| d.name.as_ref()?.prefix.clone())
    }

    fn namespace(&self) -> Namespace {
        self.node
            .node_data()
            .and_then(|d| d.name.as_ref().map(|q| q.ns.clone()))
            .unwrap_or_default()
    }

    fn local_name(&self) -> LocalName {
        self.node
            .node_data()
            .and_then(|d| d.name.as_ref().map(|q| q.local.clone()))
            .unwrap_or_default()
    }

    fn attributes(&self) -> impl Iterator<Item = Self::Attribute> {
        match self.node.kind {
            XPathKind::Node(idx) => self.node.tree.nodes[idx]
                .attrs
                .iter()
                .map(|&attr| XPathAttr {
                    node: self.node.tree.attr(attr),
                })
                .collect::<Vec<_>>()
                .into_iter(),
            XPathKind::Attr(_) => Vec::new().into_iter(),
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.node.tree.html_document && self.namespace().as_ref() == XHTML_NS
    }
}

impl xpath::Attribute for XPathAttr {
    type Node = XPathNode;

    fn as_node(&self) -> Self::Node {
        self.node.clone()
    }

    fn prefix(&self) -> Option<Prefix> {
        self.node.attr_data().and_then(|d| d.name.prefix.clone())
    }

    fn namespace(&self) -> Namespace {
        self.node
            .attr_data()
            .map(|d| d.name.ns.clone())
            .unwrap_or_default()
    }

    fn local_name(&self) -> LocalName {
        self.node
            .attr_data()
            .map(|d| d.name.local.clone())
            .unwrap_or_default()
    }
}

/// `__xpathEvaluate(expression, contextState, resolver, append)` returns a scalar record. Node-set
/// members are delivered through `append` as genuine node or Attr identities.
pub(crate) struct EvaluateXPath;

/// An arena-owned mutation cursor. The absolute sequence remains monotonic
/// when layout drains the pending journal, unlike a JS-side mutation hook.
pub(crate) struct XPathMutationCursor;
pub(crate) struct XPathIteratorInvalidated;

impl<E: ScriptEngine> NativeFn<E> for XPathMutationCursor {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let document = cx.arg(0);
        let Some(document) = cx.owned_node(&document)? else {
            return cx.make_string("");
        };
        let cursor = document.with_dom(|dom| {
            let (base, pending) = dom.pending_mutations();
            format!("{}:{}", dom.arena_id(), base + pending.len() as u64)
        });
        cx.make_string(&cursor)
    }
}

impl<E: ScriptEngine> NativeFn<E> for XPathIteratorInvalidated {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let document = cx.arg(0);
        let cursor = cx.arg(1);
        let cursor = cx.value_to_string(&cursor)?;
        let Some(document) = cx.owned_node(&document)? else {
            return cx.make_string("true");
        };
        let document_id = document.id();
        let invalidated = document.with_dom(|dom| {
            let (base, pending) = dom.pending_mutations();
            let Some((arena, sequence)) = cursor.split_once(':') else { return true };
            let Some(arena) = arena.parse::<u32>().ok() else { return true };
            let Some(sequence) = sequence.parse::<u64>().ok() else { return true };
            let end = base + pending.len() as u64;
            if arena != dom.arena_id() || sequence < base || sequence > end {
                // A drained journal cannot prove which document changed.
                return true;
            }
            let touches_document = |node: NodeId| {
                // Detached or retired nodes have no native document owner to
                // classify. Conservatively invalidate rather than miss one.
                dom.tree_root(node).is_none_or(|root| root == document_id ||
                    dom.kind(root) != NodeKind::Document)
            };
            pending.iter().skip((sequence - base) as usize).any(|change| match change {
                DomMutation::Inserted { parent, .. } => touches_document(*parent),
                DomMutation::Removed { former_parent, .. } => touches_document(*former_parent),
                DomMutation::Moved { from_parent, to_parent, .. } =>
                    touches_document(*from_parent) || touches_document(*to_parent),
                DomMutation::SubtreeReplaced { node } |
                DomMutation::AttributeChanged { node, .. } |
                DomMutation::CharacterDataChanged { node } => touches_document(*node),
                _ => false,
            })
        });
        cx.make_string(if invalidated { "true" } else { "false" })
    }
}

impl<E: ScriptEngine> NativeFn<E> for EvaluateXPath {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let expr_v = cx.arg(0);
        let expression = cx.value_to_string(&expr_v)?;
        let context_state = cx.arg(1);
        let resolver = cx.arg(2);
        let append = cx.arg(3);
        let read_context = |cx: &mut E::CallCx<'_>, field: &str| {
            let field = cx.make_string(field)?;
            let this = cx.undefined();
            E::call_from_call(cx, &context_state, &this, &[field])
        };
        let html_v = read_context(cx, "html")?;
        let mut html_document = cx.value_to_string(&html_v)? == "true";

        // Ask the generic parser which prefixes its name tests use, invoke the
        // supplied JS resolver without a DOM borrow, then parse with those URIs.
        let prefixes = Rc::new(RefCell::new(Vec::new()));
        if let Err(err) = xpath::parse(&expression, Some(CollectNamespaces(prefixes.clone())), html_document) {
            return cx.make_string(&format!("error\n{err:?}"));
        }
        let mut bindings = HashMap::new();
        for prefix in prefixes.borrow().iter() {
            if bindings.contains_key(prefix) {
                continue;
            }
            let arg = cx.make_string(prefix)?;
            let this = cx.undefined();
            let resolved = E::call_from_call(cx, &resolver, &this, &[arg])?;
            let uri = cx.value_to_string(&resolved)?;
            if !uri.is_empty() {
                bindings.insert(prefix.clone(), uri);
            }
        }
        // A resolver can remove an Attr, adopt the context, or move it to a
        // different document. Read trusted JS wrapper metadata after it runs.
        let context_v = read_context(cx, "ref")?;
        let kind_v = read_context(cx, "kind")?;
        let kind = cx.value_to_string(&kind_v)?;
        let ns_v = read_context(cx, "ns")?;
        let ns = cx.value_to_string(&ns_v)?;
        let prefix_v = read_context(cx, "prefix")?;
        let prefix = cx.value_to_string(&prefix_v)?;
        let local_v = read_context(cx, "local")?;
        let local = cx.value_to_string(&local_v)?;
        let value_v = read_context(cx, "value")?;
        let value = cx.value_to_string(&value_v)?;
        let html_v = read_context(cx, "html")?;
        html_document = cx.value_to_string(&html_v)? == "true";
        let parsed = match xpath::parse(&expression, Some(ResolvedNamespaces(bindings)), html_document) {
            Ok(parsed) => parsed,
            Err(err) => return cx.make_string(&format!("error\n{err:?}")),
        };
        let context = if kind == "detached-attr" {
            let name = QualName::new(
                (!prefix.is_empty()).then(|| Prefix::from(prefix.as_str())),
                Namespace::from(ns.as_str()),
                LocalName::from(local.as_str()),
            );
            let tree = XPathTree::detached_attr(name, value, html_document);
            tree.attr(0)
        } else {
            let Some(context_raw) = cx.owned_node(&context_v)? else {
                return cx.make_string("error\nDocument.evaluate requires a context node");
            };
            let context_id = context_raw.id();
            let (tree, context_idx) = context_raw.with_dom(|dom| {
                // Detached subtrees and secondary documents have their own
                // XPath roots; the primary document is not universal.
                let mut root = context_id;
                while let Some(parent) = dom.parent(root) {
                    root = parent;
                }
                let tree = XPathTree::from_dom(dom, root, html_document);
                let idx = tree.nodes.iter().position(|node| node.raw == context_id.raw());
                (tree, idx)
            });
            let Some(context_idx) = context_idx else {
                return cx.make_string("error\nDocument.evaluate context node is outside its tree");
            };
            if kind == "attr" {
                let found = tree.nodes[context_idx].attrs.iter().copied().find(|&idx| {
                    let attr = &tree.attrs[idx];
                    attr.name.ns.as_ref() == ns.as_str() && attr.name.local.as_ref() == local.as_str()
                });
                let Some(idx) = found else {
                    return cx.make_string("error\nDocument.evaluate context attribute is detached");
                };
                tree.attr(idx)
            } else {
                tree.node(context_idx)
            }
        };

        let mut result_nodes = Vec::new();
        let record = match xpath::evaluate_parsed_xpath::<XPathDom>(&parsed, context) {
            Ok(xpath::Value::Boolean(value)) => format!("boolean\n{value}"),
            Ok(xpath::Value::Number(value)) => format!("number\n{value}"),
            Ok(xpath::Value::String(value)) => format!("string\n{value}"),
            Ok(xpath::Value::NodeSet(nodes)) => {
                result_nodes.extend(nodes);
                "nodes\n".to_owned()
            },
            Err(err) => format!("error\n{err:?}"),
        };
        for node in result_nodes {
            let (result_kind, raw, ns, local) = match node.kind {
                XPathKind::Node(idx) => ("node", node.tree.nodes[idx].raw, "", ""),
                XPathKind::Attr(idx) => {
                    let attr = &node.tree.attrs[idx];
                    let Some(owner) = attr.owner else {
                        let result_kind = cx.make_string("detached-attr")?;
                        let this = cx.undefined();
                        E::call_from_call(cx, &append, &this, &[result_kind])?;
                        continue;
                    };
                    ("attr", node.tree.nodes[owner].raw, attr.name.ns.as_ref(), attr.name.local.as_ref())
                },
            };
            let result_kind = cx.make_string(result_kind)?;
            let reference = reflect_pinned::<E>(cx, raw)?;
            let ns = cx.make_string(ns)?;
            let local = cx.make_string(local)?;
            let this = cx.undefined();
            E::call_from_call(cx, &append, &this, &[result_kind, reference, ns, local])?;
        }
        cx.make_string(&record)
    }
}
