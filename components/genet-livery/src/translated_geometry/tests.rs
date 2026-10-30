// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{AttributeView, LayoutDomMut, LocalName, Namespace, QualName};

use super::*;
use crate::{Device, InteractionStates, StyleSet, layout, resolve_styles};

struct ScopedCount<'a> {
    dom: &'a ScriptedDom,
    root: NodeId,
    parents: Cell<usize>,
    children: Cell<usize>,
}

impl LayoutDom for ScopedCount<'_> {
    type NodeId = NodeId;
    fn document(&self) -> NodeId {
        self.root
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.parents.set(self.parents.get() + 1);
        (id != self.root).then(|| self.dom.parent(id)).flatten()
    }
    fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.dom.prev_sibling(id)
    }
    fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.dom.next_sibling(id)
    }
    fn dom_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.children.set(self.children.get() + 1);
        self.dom.dom_children(id)
    }
    fn flat_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.children.set(self.children.get() + 1);
        self.dom.flat_children(id)
    }
    fn kind(&self, id: NodeId) -> NodeKind {
        self.dom.kind(id)
    }
    fn element_name(&self, id: NodeId) -> Option<&QualName> {
        self.dom.element_name(id)
    }
    fn attribute(&self, id: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
        self.dom.attribute(id, ns, local)
    }
    fn attributes(&self, id: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
        self.dom.attributes(id)
    }
    fn text(&self, id: NodeId) -> Option<&str> {
        self.dom.text(id)
    }
}

fn scoped(dom: &ScriptedDom, root: NodeId) -> ScopedCount<'_> {
    ScopedCount {
        dom,
        root,
        parents: Cell::new(0),
        children: Cell::new(0),
    }
}

fn fixture(html: &str, css: &str) -> (ScriptedDom, StylePlane<NodeId>, LiveryLayout<NodeId>) {
    let mut dom = ScriptedDom::new();
    dom.set_inner_html(dom.document(), html);
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[css]),
        &Device::screen(400.0, 300.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 400.0, 300.0).unwrap();
    (dom, styles, fragments)
}

fn node(dom: &ScriptedDom, class: &str) -> NodeId {
    dom.first_with_class(dom.document(), class).unwrap()
}

#[test]
fn translated_geometry_nested_offsets_and_scope_boundary() {
    let (dom, styles, fragments) = fixture(
        "<div class=outer><div class=inner><div class=leaf></div></div></div>",
        ".outer { width:100px; height:80px; transform:translate(20%,10px) }
         .inner { width:50px; height:40px; transform:translate(3px,4px) }
         .leaf { width:10px; height:8px }",
    );
    let outer = node(&dom, "outer");
    let inner = node(&dom, "inner");
    let leaf = node(&dom, "leaf");
    let scroll = HashMap::from([
        (outer, (4.0, 6.0)),
        (inner, (2.0, 5.0)),
        (leaf, (99.0, 99.0)),
    ]);
    let mut rows = HashMap::new();
    walk_translated_geometry(&dom, &styles, &fragments, &scroll, |id, geometry| {
        rows.insert(id, geometry);
    });
    assert_eq!(rows[&outer].translation, (20.0, 10.0));
    assert_eq!(rows[&outer].ancestor_scroll, (0.0, 0.0));
    assert_eq!(rows[&inner].translation, (23.0, 14.0));
    assert_eq!(rows[&inner].ancestor_scroll, (4.0, 6.0));
    assert_eq!(rows[&leaf].ancestor_scroll, (6.0, 11.0));
    let fragment = fragments.get(leaf).unwrap();
    assert_eq!(
        rows[&leaf].border_rect,
        Some((fragment.x + 17.0, fragment.y + 3.0, 10.0, 8.0))
    );

    let scoped = scoped(&dom, inner);
    let mut subtree = HashMap::new();
    walk_translated_geometry(&scoped, &styles, &fragments, &scroll, |id, geometry| {
        subtree.insert(id, geometry);
    });
    assert!(!subtree.contains_key(&outer));
    assert_eq!(subtree[&inner].translation, (3.0, 4.0));
    assert_eq!(subtree[&inner].ancestor_scroll, (0.0, 0.0));
    assert_eq!(subtree[&leaf].ancestor_scroll, (2.0, 5.0));
    assert_eq!(scoped.parents.get(), 0);
}

#[test]
fn translated_geometry_style_facts_match_owner_queries_after_restyle() {
    let (dom, mut styles, fragments) = fixture(
        "<div class=hidden><div class=visible>text</div></div>
         <div class=suppressed><span class=unpainted>ignored</span></div>
         <img class=replaced>",
        ".hidden { visibility:hidden } .visible { visibility:visible }
         .visible::before { content:'before'; } .visible::after { content:'after'; }
         .suppressed { display:none } .unpainted::before { content:'absent'; }
         img::before { content:'not image text'; }",
    );
    let mut rows = HashMap::new();
    walk_translated_geometry(
        &dom,
        &styles,
        &fragments,
        &HashMap::new(),
        |id, geometry| {
            assert_eq!(
                geometry.rendered,
                crate::rendered_visible(&dom, &styles, id)
            );
            let generated = crate::rendered_generated_text(&dom, &styles, id);
            assert_eq!(
                geometry.generated_text,
                (generated.0.as_str(), generated.1.as_str())
            );
            rows.insert(id, geometry);
        },
    );
    assert!(!rows[&node(&dom, "hidden")].rendered);
    assert!(rows[&node(&dom, "visible")].rendered);
    assert!(
        !rows[&node(&dom, "visible")].placement_visible,
        "placement preserves hidden-ancestor admission even with visible override"
    );
    assert_eq!(
        rows[&node(&dom, "visible")].generated_text,
        ("before", "after")
    );
    assert_eq!(rows[&node(&dom, "replaced")].generated_text, ("", ""));
    assert!(!rows[&node(&dom, "unpainted")].rendered);
    assert!(!rows[&node(&dom, "unpainted")].placement_visible);
    drop(rows);

    // A paint-only style change must be read even while the retained geometry
    // has not changed. Its old fragment must not imply visible rendering.
    styles.get_mut(node(&dom, "visible")).unwrap().visibility = Visibility::Hidden;
    let mut after = None;
    walk_translated_geometry(
        &dom,
        &styles,
        &fragments,
        &HashMap::new(),
        |id, geometry| {
            if id == node(&dom, "visible") {
                after = Some(geometry);
            }
        },
    );
    assert!(!after.unwrap().rendered);
    assert!(after.unwrap().border_rect.is_some());
}

#[test]
fn translated_geometry_content_edges_and_placement_admission() {
    let (dom, styles, fragments) = fixture(
        "<div class=outer><div class=leaf></div></div>",
        ".outer { transform:translate(10px,20px) }
         .leaf { box-sizing:border-box; width:80px; height:60px;
            border:2px solid; padding:3px 5px; transform:translate(4px,6px) }",
    );
    let outer = node(&dom, "outer");
    let leaf = node(&dom, "leaf");
    let scroll = HashMap::from([(outer, (1.0, 2.0))]);
    let counted = scoped(&dom, dom.document());
    let mut geometry = None;
    walk_translated_geometry(&counted, &styles, &fragments, &scroll, |id, row| {
        if id == leaf {
            geometry = Some(row);
        }
    });
    let geometry = geometry.unwrap();
    let fragment = fragments.get(leaf).unwrap();
    assert_eq!(
        geometry.content_rect,
        Some((fragment.x + 20.0, fragment.y + 29.0, 66.0, 50.0))
    );
    assert!(geometry.placement_visible);
    assert_eq!(counted.parents.get(), 0);
    let original = crate::element_geometry(&dom, &styles, &fragments, &scroll, leaf).unwrap();
    assert_eq!(
        original.content_rect,
        (fragment.x + 7.0, fragment.y + 5.0, 66.0, 50.0)
    );
}

#[test]
fn translated_geometry_visits_each_flat_node_once() {
    for depth in [16, 32] {
        let (dom, styles, fragments) = fixture(
            &format!("{}<input>{}", "<div>".repeat(depth), "</div>".repeat(depth)),
            "",
        );
        let scoped = scoped(&dom, dom.document());
        let mut visits = 0;
        walk_translated_geometry(&scoped, &styles, &fragments, &HashMap::new(), |_, _| {
            visits += 1
        });
        assert!(visits > depth);
        assert_eq!(scoped.children.get(), visits, "one child iterator per node");
        assert_eq!(scoped.parents.get(), 0, "no repeated ancestor work");
    }
}
