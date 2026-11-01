/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{
    AttributeView, LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName,
};

use crate::{ScrollOffsets, document_a11y_projection_with_scroll, fragments_from_scripted_dom};

struct CountingDom<'a> {
    inner: &'a ScriptedDom,
    parents: Cell<usize>,
}

impl LayoutDom for CountingDom<'_> {
    type NodeId = NodeId;

    fn document(&self) -> NodeId {
        self.inner.document()
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
        self.inner.dom_children(id)
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

#[test]
fn nested_scroll_projection_does_not_rewalk_ancestor_chains() {
    let mut measurements = Vec::new();
    for depth in [16, 32] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let html = format!(
            "{}<input value=\"edit\">{}",
            "<div aria-label=\"container\">".repeat(depth),
            "</div>".repeat(depth)
        );
        dom.set_inner_html(root, &html);
        let fragments = fragments_from_scripted_dom(&dom, &["div { display:block }"], 400, 300)
            .expect("layout");
        let counted = CountingDom {
            inner: &dom,
            parents: Cell::new(0),
        };
        let projection = document_a11y_projection_with_scroll(
            &counted,
            &fragments,
            None,
            1,
            &ScrollOffsets::new(),
        );
        let visits = counted.parents.get();
        eprintln!(
            "depth={depth} nodes={} parent_reads={visits}",
            projection.nodes().len()
        );
        assert!(projection.nodes().len() > depth);
        measurements.push((depth, visits, projection.nodes().len()));
    }
    assert!(
        measurements
            .iter()
            .all(|(_, reads, nodes)| reads <= &(nodes * 2)),
        "scroll geometry must carry inherited state: {measurements:?} (depth, parent reads, nodes)"
    );
}

#[test]
fn cancelling_ancestor_scroll_keeps_action_barrier_focus_and_style_semantics() {
    use crate::{A11yStyleQueries, document_a11y_projection_with_style};
    use document_session_api::DocumentA11yAction;

    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root,
        "<div class=outer aria-label=outer><div class=inner aria-label=inner><input value=edit></div></div>");
    let outer = dom.first_with_class(root, "outer").unwrap();
    let inner = dom.first_with_class(root, "inner").unwrap();
    let input = dom.dom_children(inner).next().unwrap();
    let fragments =
        fragments_from_scripted_dom(&dom, &["div { display:block }"], 400, 300).expect("layout");
    let generated = |_| (String::new(), String::new());
    let rendered = |node| node != outer;
    let style = A11yStyleQueries {
        generated: &generated,
        rendered: &rendered,
    };
    let baseline =
        document_a11y_projection_with_style(&dom, &fragments, Some(input), 7, None, &style);
    let offsets = ScrollOffsets::from([(outer, (3.0, 5.0)), (inner, (-3.0, -5.0))]);
    let scrolled = document_a11y_projection_with_style(
        &dom,
        &fragments,
        Some(input),
        7,
        Some(&offsets),
        &style,
    );
    assert!(
        !scrolled
            .nodes()
            .iter()
            .any(|node| node.id.get() == dom.opaque_id(outer))
    );
    let find = |projection: &document_session_api::DocumentA11yProjection| {
        projection
            .nodes()
            .iter()
            .find(|node| node.id.get() == dom.opaque_id(input))
            .unwrap()
            .clone()
    };
    let before = find(&baseline);
    let after = find(&scrolled);
    assert_eq!(
        after.bounds, before.bounds,
        "opposite ancestor offsets cancel geometrically"
    );
    assert!(after.state.focused);
    assert!(after.state.editable);
    assert!(after.actions.contains(&DocumentA11yAction::Focus));
    assert!(after.actions.contains(&DocumentA11yAction::ScrollIntoView));
    assert!(!after.actions.contains(&DocumentA11yAction::Click));
    assert!(!after.actions.contains(&DocumentA11yAction::SetValue));
}

#[test]
fn scroll_projection_uses_flat_slot_ancestors() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root,
        "<div class=host><template shadowrootmode=open><slot></slot></template><button class=action>Action</button></div>");
    assert_eq!(dom.realize_declarative_shadow_roots(root).len(), 1);
    let host = dom.first_with_class(root, "host").unwrap();
    let action = dom.first_with_class(root, "action").unwrap();
    let shadow = dom.shadow_root(host).unwrap();
    let slot = dom.assigned_slot(action).unwrap();
    let fragments = fragments_from_scripted_dom(&dom, &["div, slot { display:block }"], 400, 300)
        .expect("layout");
    let before = document_a11y_projection_with_scroll(
        &dom,
        &fragments,
        Some(action),
        1,
        &ScrollOffsets::new(),
    );
    let offsets = ScrollOffsets::from([
        (host, (3.0, 5.0)),
        (slot, (7.0, 11.0)),
        (shadow, (100.0, 200.0)),
    ]);
    let after = document_a11y_projection_with_scroll(&dom, &fragments, Some(action), 1, &offsets);
    let find = |projection: &document_session_api::DocumentA11yProjection| {
        projection
            .nodes()
            .iter()
            .find(|node| node.id.get() == dom.opaque_id(action))
            .unwrap()
            .clone()
    };
    let first = find(&before);
    let second = find(&after);
    let first_bounds = first.bounds.unwrap();
    let second_bounds = second.bounds.unwrap();
    assert_eq!(second_bounds.x, first_bounds.x - 10.0);
    assert_eq!(second_bounds.y, first_bounds.y - 16.0);
    assert!(second.state.focused);
}
