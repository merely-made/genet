// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A `style` attribute change stops at its element unless a descendant could
//! read something it changed. Every case is checked against a full restyle.

use genet_livery::{
    Device, IncrementalStyle, InteractionStates, RestyleStats, StyleSet, resolve_styles,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{
    DomMutation, LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName,
};

const TREE: &str = "<html><body>\
     <div id='root' style='position: relative; top: 0px'>\
       <section class='a'><div class='b'><p id='leaf' class='leaf'><span></span></p></div></section>\
       <section class='a'></section><section class='b'><span></span><span></span></section>\
     </div>\
     <aside><span></span></aside>\
     </body></html>";

struct Fixture {
    dom: ScriptedDom,
    styles: StyleSet,
    states: InteractionStates<NodeId>,
    device: Device,
    session: IncrementalStyle<NodeId>,
}

impl Fixture {
    fn new(markup: &str, sheet: &str) -> Self {
        let mut dom = ScriptedDom::from_serialized_document(markup);
        let mut discarded = Vec::new();
        dom.drain_mutations(&mut discarded);
        let mut fixture = Self {
            dom,
            styles: StyleSet::cambium(&[sheet]),
            states: InteractionStates::default(),
            device: Device::screen(800.0, 600.0),
            session: IncrementalStyle::new(),
        };
        assert!(fixture.update().full_document);
        fixture
    }

    fn id(&self, expected: &str) -> NodeId {
        fn find(dom: &ScriptedDom, node: NodeId, expected: &str) -> Option<NodeId> {
            if dom.kind(node) == NodeKind::Element
                && dom.attribute(node, &Namespace::from(""), &LocalName::from("id"))
                    == Some(expected)
            {
                return Some(node);
            }
            dom.dom_children(node)
                .find_map(|child| find(dom, child, expected))
        }
        find(&self.dom, self.dom.document(), expected).expect("fixture id")
    }

    fn set(&mut self, id: &str, name: &str, value: &str) {
        let node = self.id(id);
        self.dom.set_attribute(
            node,
            QualName::new(None, Namespace::from(""), LocalName::from(name)),
            value,
        );
    }

    fn update(&mut self) -> RestyleStats {
        let mut mutations = Vec::<DomMutation<NodeId>>::new();
        self.dom.drain_mutations(&mut mutations);
        self.session.update(
            &self.dom,
            &self.styles,
            &self.device,
            &self.states,
            &mutations,
        )
    }

    /// The incremental plane equals a fresh full restyle of the final DOM.
    fn assert_matches_full(&self) {
        let full = resolve_styles(&self.dom, &self.styles, &self.device, &self.states);
        assert!(*self.session.styles() == full, "incremental plane differs");
    }

    fn subtree_elements(&self, id: &str) -> usize {
        fn count(dom: &ScriptedDom, node: NodeId) -> usize {
            usize::from(dom.kind(node) == NodeKind::Element)
                + dom
                    .dom_children(node)
                    .map(|child| count(dom, child))
                    .sum::<usize>()
        }
        count(&self.dom, self.id(id))
    }

    fn computed(&self, id: &str, property: &str) -> Option<String> {
        self.session.styles().computed_style(self.id(id), property)
    }
}

const MOVE_TOP: &str = "position: relative; top: 10px";

#[test]
fn top_only_stops_at_the_root() {
    let mut fixture = Fixture::new(TREE, "");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.hints, 1);
    assert_eq!(stats.restyled_elements, 1);
    assert_eq!(fixture.computed("root", "top"), Some("10px".to_string()));
}

#[test]
fn inherited_property_change_descends() {
    let mut fixture = Fixture::new(TREE, "");
    fixture.set(
        "root",
        "style",
        "position: relative; top: 0px; color: #0000ff",
    );
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
    assert_eq!(
        fixture.computed("leaf", "color"),
        Some("rgb(0, 0, 255)".to_string())
    );
}

#[test]
fn custom_property_change_descends() {
    let mut fixture = Fixture::new(TREE, ".leaf { width: var(--w); }");
    fixture.set("root", "style", "position: relative; top: 0px; --w: 7px");
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
    assert_eq!(fixture.computed("leaf", "width"), Some("7px".to_string()));
}

#[test]
fn sheet_inherit_of_a_reset_property_descends() {
    let mut fixture = Fixture::new(TREE, "section.a { position: relative; top: inherit; }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
}

#[test]
fn inline_inherit_of_a_reset_property_descends() {
    let markup = TREE.replace(
        "<section class='a'><div",
        "<section id='reader' class='a' style='top: inherit'><div",
    );
    let mut fixture = Fixture::new(&markup, "");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
    assert_eq!(fixture.computed("reader", "top"), Some("10px".to_string()));
}

/// `--t: inherit` inherits the custom value, not the keyword: `top` reads the
/// root's `--t`, which did not change, so the restyle may stop.
#[test]
fn custom_property_keyword_inherit_reads_only_custom_values() {
    let mut fixture = Fixture::new(TREE, "section.a { --t: inherit; top: var(--t); }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, 1);
}

#[test]
fn var_resolving_to_inherit_descends() {
    let mut fixture = Fixture::new(TREE, "section.a { top: var(--missing, inherit); }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
}

#[test]
fn style_attribute_selector_descends() {
    let mut fixture = Fixture::new(TREE, "[style] .leaf { color: #008000; }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert!(stats.restyled_elements >= fixture.subtree_elements("root"));
}

#[test]
fn sibling_combinator_does_not_promote_a_style_change() {
    let mut fixture = Fixture::new(TREE, ".a + .b { color: #008000; } .a ~ .b { width: 3px; }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.hints, 1);
    assert_eq!(stats.restyled_elements, 1);
}

#[test]
fn class_change_in_the_same_batch_descends() {
    let mut fixture = Fixture::new(TREE, ".on .leaf { color: #0000ff; }");
    fixture.set("root", "style", MOVE_TOP);
    fixture.set("root", "class", "on");
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
    assert_eq!(
        fixture.computed("leaf", "color"),
        Some("rgb(0, 0, 255)".to_string())
    );
}

/// The leaf's class change is its own root below the stopped root.
#[test]
fn descendant_hint_restyles_below_a_stopped_root() {
    let mut fixture = Fixture::new(TREE, ".on { color: #0000ff; }");
    fixture.set("root", "style", MOVE_TOP);
    fixture.set("leaf", "class", "leaf on");
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(
        stats.restyled_elements,
        1 + fixture.subtree_elements("leaf")
    );
    assert_eq!(
        fixture.computed("leaf", "color"),
        Some("rgb(0, 0, 255)".to_string())
    );
}

#[test]
fn removing_the_inheriting_child_restores_the_stop() {
    let reader = TREE.replace(
        "<section class='a'><div",
        "<section id='reader' class='a'><div",
    );
    for (markup, sheet) in [
        (
            reader.replace("id='reader'", "id='reader' style='top: inherit'"),
            "",
        ),
        (reader.clone(), "#reader { top: inherit; }"),
    ] {
        let mut fixture = Fixture::new(&markup, sheet);
        fixture.set("root", "style", MOVE_TOP);
        let stats = fixture.update();
        fixture.assert_matches_full();
        assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));

        let reader = fixture.id("reader");
        fixture.dom.remove(reader);
        fixture.update();
        fixture.assert_matches_full();

        fixture.set("root", "style", "position: relative; top: 20px");
        let stats = fixture.update();
        fixture.assert_matches_full();
        assert_eq!(stats.restyled_elements, 1);
        assert_eq!(fixture.computed("root", "top"), Some("20px".to_string()));
    }
}

/// A grandchild's `inherit` reads the child, which the stop leaves unchanged.
#[test]
fn grandchild_inherit_stops() {
    for (markup, sheet) in [
        (
            TREE.replace("<p id='leaf'", "<p id='leaf' style='top: inherit'"),
            "",
        ),
        (TREE.to_string(), "div.b { top: inherit; }"),
    ] {
        let mut fixture = Fixture::new(&markup, sheet);
        fixture.set("root", "style", MOVE_TOP);
        let stats = fixture.update();
        fixture.assert_matches_full();
        assert_eq!(stats.restyled_elements, 1);
    }
}

#[test]
fn table_elsewhere_in_the_document_stops() {
    let markup = TREE.replace(
        "<aside>",
        "<table><tbody><tr><td>x</td></tr></tbody></table><aside>",
    );
    let mut fixture = Fixture::new(&markup, "");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, 1);
}

#[test]
fn table_levels_below_the_root_stops() {
    let rows = "<tr><td>a</td><td>b</td></tr>".repeat(8);
    let markup = format!(
        "<html><body><div id='root' style='position: relative; top: 0px'>         <div><table><tbody>{rows}</tbody></table></div></div></body></html>"
    );
    let mut fixture = Fixture::new(&markup, "");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, 1);
}

/// UA `td { vertical-align: inherit }` and `tr { border-*-color: inherit }`.
#[test]
fn table_part_children_descend() {
    let rows = "<tr><td>a</td><td>b</td></tr>".repeat(3);
    for (root, style) in [
        ("tr", "vertical-align: top; color: red; top: 1px"),
        ("tbody", "border-top-color: blue; top: 1px"),
    ] {
        let markup = format!(
            "<html><body><table><tbody id='{}'>{rows}</tbody></table></body></html>",
            if root == "tbody" { "root" } else { "body-rows" }
        )
        .replacen(
            "<tr>",
            if root == "tr" {
                "<tr id='root'>"
            } else {
                "<tr>"
            },
            1,
        );
        let mut fixture = Fixture::new(&markup, "");
        fixture.set("root", "style", style);
        let stats = fixture.update();
        fixture.assert_matches_full();
        assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
        fixture.set("root", "style", &style.replace("1px", "9px"));
        let stats = fixture.update();
        fixture.assert_matches_full();
        assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
    }
}

#[test]
fn child_pseudo_inherit_descends() {
    let mut fixture = Fixture::new(TREE, "section.a::before { content: 'x'; top: inherit; }");
    fixture.set("root", "style", MOVE_TOP);
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
}

/// A strip of rows scrolled by whole rows: `top` moves, one row enters, one
/// row leaves, all in one batch. 12 rows of 4 cells plus a 6-deep chain.
fn strip_markup() -> String {
    let rows =
        "<div class='row'><span></span><span></span><span></span><span></span></div>".repeat(12);
    format!(
        "<html><body><div id='root' style='position: relative; top: 0px'>         <div id='first' class='row'><span></span></div>{rows}         <div><div><div><div id='deep'><div><p id='grand'><span></span></p></div></div></div></div></div>         </div><aside><span></span></aside></body></html>"
    )
}

/// Append a row of `cells` spans under `root`; returns the row.
fn append_row(fixture: &mut Fixture, class: &str, style: Option<&str>, cells: usize) -> NodeId {
    let html = |local: &str| {
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from(local),
        )
    };
    let root = fixture.id("root");
    let row = fixture.dom.create_element(html("div"));
    fixture.dom.append_child(root, row);
    fixture.dom.set_attribute(
        row,
        QualName::new(None, Namespace::from(""), LocalName::from("class")),
        class,
    );
    if let Some(style) = style {
        fixture.dom.set_attribute(
            row,
            QualName::new(None, Namespace::from(""), LocalName::from("style")),
            style,
        );
    }
    for _ in 0..cells {
        let cell = fixture.dom.create_element(html("span"));
        fixture.dom.append_child(row, cell);
    }
    row
}

fn count_subtree(dom: &ScriptedDom, node: NodeId) -> usize {
    usize::from(dom.kind(node) == NodeKind::Element)
        + dom
            .dom_children(node)
            .map(|child| count_subtree(dom, child))
            .sum::<usize>()
}

fn scroll_step(
    fixture: &mut Fixture,
    root_style: &str,
    row_style: Option<&str>,
) -> (RestyleStats, usize) {
    fixture.set("root", "style", root_style);
    let row = append_row(fixture, "row", row_style, 4);
    let first = fixture.id("first");
    fixture.dom.remove(first);
    let stats = fixture.update();
    fixture.assert_matches_full();
    (stats, count_subtree(&fixture.dom, row))
}

#[test]
fn scroll_step_stops_at_the_strip() {
    let mut fixture = Fixture::new(&strip_markup(), "");
    assert!(fixture.subtree_elements("root") > 50);
    let (stats, row) = scroll_step(&mut fixture, MOVE_TOP, None);
    assert_eq!(row, 5);
    assert_eq!(stats.restyled_elements, 1 + row);
}

/// The inserted row is its own root and reads the strip's new values, so
/// the strip may still stop: the row has no stale entry to protect.
#[test]
fn scroll_step_with_an_inheriting_inserted_row() {
    let mut fixture = Fixture::new(&strip_markup(), "");
    let (stats, row) = scroll_step(
        &mut fixture,
        MOVE_TOP,
        Some("position: relative; top: inherit"),
    );
    assert_eq!(stats.restyled_elements, 1 + row);
}

/// A retained child that inherits `top` forces the strip to descend, and the
/// inserted row below it is not cascaded a second time.
#[test]
fn scroll_step_with_an_inheriting_retained_row_descends_once() {
    let markup = strip_markup().replacen(
        "<div class='row'>",
        "<div class='row' style='top: inherit'>",
        1,
    );
    let mut fixture = Fixture::new(&markup, "");
    let (stats, _) = scroll_step(&mut fixture, MOVE_TOP, None);
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
}

#[test]
fn scroll_step_with_a_color_change_descends_once() {
    let mut fixture = Fixture::new(&strip_markup(), "");
    let (stats, _) = scroll_step(
        &mut fixture,
        "position: relative; top: 10px; color: #0000ff",
        None,
    );
    assert_eq!(stats.restyled_elements, fixture.subtree_elements("root"));
}

#[test]
fn grandchild_class_change_restyles_only_its_subtree() {
    let mut fixture = Fixture::new(&strip_markup(), ".on { color: #0000ff; }");
    fixture.set("root", "style", MOVE_TOP);
    fixture.set("grand", "class", "on");
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(
        stats.restyled_elements,
        1 + fixture.subtree_elements("grand")
    );
}

#[test]
fn nested_style_roots_both_stop() {
    let mut fixture = Fixture::new(&strip_markup(), "");
    fixture.set("root", "style", MOVE_TOP);
    fixture.set("deep", "style", "position: relative; top: 4px");
    let stats = fixture.update();
    fixture.assert_matches_full();
    assert_eq!(stats.restyled_elements, 2);
}

#[test]
fn scroll_step_under_a_sibling_combinator_sheet() {
    let mut fixture = Fixture::new(
        &strip_markup(),
        ".row + .row { color: #008000; } .row + .row > span { width: 3px; }",
    );
    let (stats, _) = scroll_step(&mut fixture, MOVE_TOP, None);
    // Removing the first row changes `+` matching among the strip's
    // children, so the structural hint restyles the strip's whole subtree.
    assert!(stats.restyled_elements >= fixture.subtree_elements("root"));
}
