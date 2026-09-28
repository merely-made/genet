// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Count DOM work rather than timing a noisy test process. Every changed
//! element still cascades and is checked against a fresh full style plane.

use std::cell::Cell;

use genet_livery::{Device, IncrementalStyle, InteractionStates, StyleSet, resolve_styles};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{
    AttributeView, LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName,
};

struct CountedDom {
    inner: ScriptedDom,
    children: Cell<usize>,
    parents: Cell<usize>,
}

impl LayoutDom for CountedDom {
    type NodeId = NodeId;
    fn document(&self) -> NodeId {
        self.inner.document()
    }
    fn is_live(&self, id: NodeId) -> bool {
        self.inner.is_live(id)
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.parents.set(self.parents.get() + 1);
        self.inner.parent(id)
    }
    fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.inner.prev_sibling(id)
    }
    fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.inner.next_sibling(id)
    }
    fn dom_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.inner
            .dom_children(id)
            .inspect(|_| self.children.set(self.children.get() + 1))
    }
    fn kind(&self, id: NodeId) -> NodeKind {
        self.inner.kind(id)
    }
    fn opaque_id(&self, id: NodeId) -> u64 {
        self.inner.opaque_id(id)
    }
    fn element_name(&self, id: NodeId) -> Option<&QualName> {
        self.inner.element_name(id)
    }
    fn attribute(&self, id: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
        self.inner.attribute(id, ns, local)
    }
    fn attributes(&self, id: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
        self.inner.attributes(id)
    }
    fn text(&self, id: NodeId) -> Option<&str> {
        self.inner.text(id)
    }
}

fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str, class: &str) -> NodeId {
    let node = dom.create_element(QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from(tag),
    ));
    dom.set_attribute(node, attr("class"), class);
    dom.append_child(parent, node);
    node
}

fn compare_full(
    dom: &impl LayoutDom<NodeId = NodeId>,
    session: &IncrementalStyle<NodeId>,
    styles: &StyleSet,
    states: &InteractionStates<NodeId>,
) {
    let full = resolve_styles(dom, styles, &Device::screen(800.0, 600.0), states);
    let mut pending = vec![dom.document()];
    while let Some(node) = pending.pop() {
        assert_eq!(session.styles().get(node), full.get(node));
        assert_eq!(
            session.styles().custom_properties(node),
            full.custom_properties(node)
        );
        pending.extend(dom.dom_children(node));
    }
}

fn moving_sibling_work(count: usize) -> (usize, usize) {
    let mut inner = ScriptedDom::new();
    let root = inner.document();
    let stage = element(&mut inner, root, "div", "stage");
    let nodes: Vec<_> = (0..count)
        .map(|_| {
            let node = element(&mut inner, stage, "div", "node");
            element(&mut inner, node, "span", "caption");
            node
        })
        .collect();
    inner.drain_mutations(&mut Vec::new());
    let mut dom = CountedDom {
        inner,
        children: Cell::new(0),
        parents: Cell::new(0),
    };
    let styles = StyleSet::cambium(&[
        ".node { position:absolute; transform:translate(0px,0px); z-index:calc(sibling-index() * 100 + sibling-count()); } .caption { color: green; }",
    ]);
    let states = InteractionStates::default();
    let device = Device::screen(800.0, 600.0);
    let mut session = IncrementalStyle::new();
    session.update(&dom, &styles, &device, &states, &[]);
    for (index, node) in nodes.iter().enumerate() {
        dom.inner.set_attribute(
            *node,
            attr("style"),
            &format!("transform:translate({index}px,20px)"),
        );
    }
    let mut mutations = Vec::new();
    dom.inner.drain_mutations(&mut mutations);
    dom.children.set(0);
    dom.parents.set(0);
    let stats = session.update(&dom, &styles, &device, &states, &mutations);
    let work = (dom.children.get(), dom.parents.get());
    assert_eq!(stats.hints, count);
    assert_eq!(stats.restyled_elements, count * 2);
    assert!(!stats.full_document);
    compare_full(&dom, &session, &styles, &states);
    for (index, node) in nodes.iter().enumerate() {
        assert_eq!(
            session.styles().computed_style(*node, "z-index"),
            Some(((index + 1) * 100 + count).to_string())
        );
    }
    work
}

#[test]
fn moving_siblings_do_not_repeat_document_and_sibling_walks() {
    let small = moving_sibling_work(64);
    let large = moving_sibling_work(256);
    eprintln!(
        "DOM work: 64 siblings {small:?}; 256 siblings {large:?} (child visits, parent lookups)"
    );
    assert!(
        large.0 <= small.0 * 5,
        "child visits must scale near linearly: {small:?} -> {large:?}"
    );
    assert!(
        large.1 <= small.1 * 5,
        "hint coalescing must not compare all sibling pairs: {small:?} -> {large:?}"
    );
}

#[test]
fn batched_ancestor_descendant_and_duplicate_hints_keep_full_cascade_results() {
    for sibling_rules in [false, true] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let left = element(&mut dom, root, "main", "left");
        let left_child = element(&mut dom, left, "span", "leaf");
        let right = element(&mut dom, root, "aside", "right");
        let right_child = element(&mut dom, right, "span", "leaf");
        dom.drain_mutations(&mut Vec::new());
        let rules = if sibling_rules {
            ".on + .right .leaf { background: blue; } .leaf:last-child { width:30px; }"
        } else {
            ""
        };
        let styles = StyleSet::cambium(&[".leaf { color:var(--ink, green); }", rules]);
        let states = InteractionStates::default();
        let device = Device::screen(800.0, 600.0);
        let mut session = IncrementalStyle::new();
        session.update(&dom, &styles, &device, &states, &[]);
        // The later ancestor covers the earlier leaf, while the separate
        // branch and repeated leaf mutations remain represented exactly once.
        dom.set_attribute(left_child, attr("style"), "height: 10px");
        dom.set_attribute(right_child, attr("style"), "height: 20px");
        dom.set_attribute(left, attr("style"), "--ink: red");
        dom.set_attribute(left, attr("class"), "left on");
        dom.set_attribute(left_child, attr("style"), "height: 12px");
        let added = element(&mut dom, right, "span", "leaf");
        dom.remove(right_child);
        let mut mutations = Vec::new();
        dom.drain_mutations(&mut mutations);
        session.update(&dom, &styles, &device, &states, &mutations);
        compare_full(&dom, &session, &styles, &states);
        assert!(session.styles().get(right_child).is_none());
        assert!(session.styles().get(added).is_some());
        assert_eq!(
            session.styles().computed_style(left_child, "color"),
            Some("rgb(255, 0, 0)".into())
        );
    }
}

#[test]
fn disjoint_shadow_roots_share_facts_without_sharing_scopes() {
    use genet_document_resources::ResolvedDocumentResources;
    use genet_static_dom::StaticDocument;
    use livery::selector::StatePseudoClass;

    let dom = StaticDocument::parse(
        "<!doctype html><html><body>\
         <div><template shadowrootmode='open'>\
           <style>font:hover { font-size:21px; color:red; }</style>\
           <font color='red'>first</font>\
         </template></div>\
         <div><template shadowrootmode='open'>\
           <style>font:hover { font-size:31px; color:blue; }</style>\
           <font color='blue'>second</font>\
         </template></div></body></html>",
    );
    let roots = dom.shadow_roots();
    assert_eq!(roots.len(), 2);
    let fonts: Vec<_> = roots
        .iter()
        .map(|root| {
            dom.dom_children(*root)
                .find(|node| {
                    dom.element_name(*node)
                        .is_some_and(|name| name.local.as_ref() == "font")
                })
                .unwrap()
        })
        .collect();
    let styles =
        StyleSet::cambium_resources(&ResolvedDocumentResources::discover(&dom, None).stylesheets);
    let device = Device::screen(800.0, 600.0);
    let mut states = InteractionStates::default();
    let mut session = IncrementalStyle::new();
    session.update(&dom, &styles, &device, &states, &[]);
    for font in &fonts {
        states.set(*font, StatePseudoClass::Hover, true);
    }
    let stats = session.update(&dom, &styles, &device, &states, &[]);
    assert_eq!(stats.hints, 2);
    assert_eq!(stats.restyled_elements, 2);
    let full = resolve_styles(&dom, &styles, &device, &states);
    for font in fonts {
        assert_eq!(session.styles().get(font), full.get(font));
        let red =
            dom.attribute(font, &Namespace::from(""), &LocalName::from("color")) == Some("red");
        assert_eq!(
            session.styles().computed_style(font, "font-size"),
            Some(if red { "21px" } else { "31px" }.into())
        );
        assert_eq!(
            session.styles().computed_style(font, "color"),
            Some(
                if red {
                    "rgb(255, 0, 0)"
                } else {
                    "rgb(0, 0, 255)"
                }
                .into()
            )
        );
    }
}

#[test]
fn disjoint_html_hint_mutations_use_current_document_facts() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let first = element(&mut dom, root, "font", "first");
    let second = element(&mut dom, root, "font", "second");
    dom.set_attribute(first, attr("color"), "green");
    dom.set_attribute(second, attr("color"), "green");
    dom.drain_mutations(&mut Vec::new());
    let styles = StyleSet::cambium(&[]);
    let states = InteractionStates::default();
    let device = Device::screen(800.0, 600.0);
    let mut session = IncrementalStyle::new();
    session.update(&dom, &styles, &device, &states, &[]);
    dom.set_attribute(first, attr("color"), "red");
    dom.set_attribute(second, attr("color"), "blue");
    let mut mutations = Vec::new();
    dom.drain_mutations(&mut mutations);
    let stats = session.update(&dom, &styles, &device, &states, &mutations);
    assert_eq!(stats.hints, 2);
    compare_full(&dom, &session, &styles, &states);
    assert_eq!(
        session.styles().computed_style(first, "color"),
        Some("rgb(255, 0, 0)".into())
    );
    assert_eq!(
        session.styles().computed_style(second, "color"),
        Some("rgb(0, 0, 255)".into())
    );
}
