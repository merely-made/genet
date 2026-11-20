// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Equivalence receipts for the scroll-batch route: after each batch the
//! retained layout must equal a fresh layout of the same final DOM. The DOM is
//! a borrowed `ScriptedDom` and the cascade is run by hand on each side of the
//! batch, as a host that owns its DOM does.

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDomMut, QualName, ShadowRootInit, ShadowRootMode, SlotAssignmentMode};

use super::*;

mod harness;
mod timing;
use harness::*;

const FIRST: usize = 50;

fn row_node(dom: &ScriptedDom, index: usize) -> NodeId {
    by_id(dom, &format!("r{index}"))
}

/// One scroll frame: the strip moves a row up, the top row leaves and the
/// next row enters at the bottom, and the window's rows are re-labelled.
fn scroll_by_one(dom: &mut ScriptedDom, first: usize) {
    scroll_strip(dom, first + 1);
    let leaving = row_node(dom, first);
    dom.remove(leaving);
    insert_row(dom, first + WINDOW, None);
    for index in first + 1..first + 1 + WINDOW {
        let row = row_node(dom, index);
        dom.set_attribute(row, attr("aria-rowindex"), &index.to_string());
        dom.set_attribute(row, attr("tabindex"), "-1");
    }
}

fn div(dom: &mut ScriptedDom) -> NodeId {
    dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("div"),
    ))
}

#[test]
fn scroll_batch_inset_only_matches_a_fresh_layout() {
    let mut session = Session::new(&table_html(FIRST));
    session.admitted(|dom| scroll_strip(dom, FIRST + 1));
    session.admitted(|dom| scroll_strip(dom, FIRST - 3));
}

#[test]
fn scroll_batch_one_row_in_one_out_matches_a_fresh_layout() {
    let mut session = Session::new(&table_html(FIRST));
    session.admitted(|dom| {
        let leaving = row_node(dom, FIRST);
        dom.remove(leaving);
        insert_row(dom, FIRST + WINDOW, None);
    });
}

#[test]
fn scroll_batch_two_rows_in_two_out_matches_a_fresh_layout() {
    let mut session = Session::new(&table_html(FIRST));
    session.admitted(|dom| {
        for index in [FIRST, FIRST + 1] {
            let leaving = row_node(dom, index);
            dom.remove(leaving);
        }
        insert_row(dom, FIRST + WINDOW, None);
        insert_row(dom, FIRST + WINDOW + 1, None);
    });
}

#[test]
fn scroll_batch_rows_in_with_the_strip_moved_match_over_several_frames() {
    let mut session = Session::new(&table_html(FIRST));
    for frame in 0..4 {
        session.admitted(|dom| scroll_by_one(dom, FIRST + frame));
    }
    // Scrolling back inserts before the first kept row.
    session.admitted(|dom| {
        scroll_strip(dom, FIRST + 3);
        let leaving = row_node(dom, FIRST + 4 + WINDOW - 1);
        dom.remove(leaving);
        let first = row_node(dom, FIRST + 4);
        insert_row(dom, FIRST + 3, Some(first));
    });
}

#[test]
fn scroll_batch_row_holding_an_inline_child_keeps_its_inline_box() {
    let mut session = Session::new(&table_html(FIRST));
    session.admitted(|dom| scroll_by_one(dom, FIRST));
    let row = row_node(&session.dom, FIRST + WINDOW);
    let mut pending = vec![row];
    let mut bold = None;
    while let Some(node) = pending.pop() {
        if session
            .dom
            .element_name(node)
            .is_some_and(|name| &*name.local == "b")
        {
            bold = Some(node);
        }
        pending.extend(session.dom.dom_children(node));
    }
    let bold = bold.expect("the entering row holds a <b>");
    let retained = session.layout.inline_box_fragments(bold).map(<[_]>::to_vec);
    let (_, fresh) = fresh_layout(&session.dom, &session.styles_set, &mut session.text);
    assert!(retained.is_some());
    assert_eq!(
        retained,
        fresh.inline_box_fragments(bold).map(<[_]>::to_vec)
    );
}

#[test]
fn scroll_batch_with_a_paint_only_class_change_matches_a_fresh_layout() {
    let mut session = Session::new(&table_html(FIRST));
    session.admitted(|dom| {
        scroll_by_one(dom, FIRST);
        let focused = row_node(dom, FIRST + 2);
        dom.set_attribute(focused, attr("class"), "row focus");
    });
}

#[test]
fn scroll_batch_refuses_an_entering_root_that_is_not_absolute() {
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        dom.set_attribute(row, attr("style"), "position: static");
    });
}

#[test]
fn scroll_batch_refuses_an_entering_root_sized_by_its_content() {
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        dom.set_attribute(row, attr("style"), "top: 0px; height: auto");
    });
}

#[test]
fn scroll_batch_refuses_roots_under_two_parents() {
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        insert_row(dom, FIRST + WINDOW, None);
        let body = by_id(dom, "body");
        let stray = div(dom);
        dom.set_attribute(
            stray,
            attr("style"),
            "position: absolute; width: 1px; height: 1px",
        );
        dom.append_child(body, stray);
    });
}

#[test]
fn scroll_batch_refuses_a_layout_affecting_style_change() {
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        scroll_by_one(dom, FIRST);
        let row = row_node(dom, FIRST + 2);
        let cell = dom.dom_children(row).next().unwrap();
        dom.set_attribute(cell, attr("class"), "cell c0 wide");
    });
}

#[test]
fn scroll_batch_refuses_two_moved_elements() {
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        scroll_strip(dom, FIRST + 1);
        let row = row_node(dom, FIRST + 2);
        dom.set_attribute(row, attr("style"), "top: 7px");
    });
}

/// Condition 4: a box the local formatter does not reproduce, inside an
/// entering row, sends the whole batch to complete layout.
#[test]
fn scroll_batch_refuses_fixed_float_table_and_shadow_content() {
    for style in ["position: fixed", "float: left", "display: table-cell"] {
        let mut session = Session::new(&table_html(FIRST));
        session.refused(|dom| {
            let row = insert_row(dom, FIRST + WINDOW, None);
            let cell = dom.dom_children(row).nth(1).unwrap();
            let inner = dom.dom_children(cell).nth(1).unwrap();
            dom.set_attribute(inner, attr("style"), style);
        });
    }
    let mut session = Session::new(&table_html(FIRST));
    session.refused(|dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        let cell = dom.dom_children(row).next().unwrap();
        dom.attach_shadow(
            cell,
            ShadowRootInit {
                mode: ShadowRootMode::Open,
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: SlotAssignmentMode::Named,
            },
        )
        .expect("a div hosts a shadow root");
    });
}

#[test]
fn scroll_batch_refuses_a_parent_that_was_not_laid_out() {
    let html = table_html(FIRST).replace(
        "</div></div></body>",
        "</div><div id=hidden style=\"display: none\"></div></div></body>",
    );
    let mut session = Session::new(&html);
    session.refused(|dom| {
        let hidden = by_id(dom, "hidden");
        let row = div(dom);
        dom.set_attribute(row, attr("class"), "row");
        dom.append_child(hidden, row);
    });
}

#[test]
fn scroll_batch_refuses_a_changed_viewport() {
    let mut session = Session::new(&table_html(FIRST));
    scroll_strip(&mut session.dom, FIRST + 1);
    let mutations = session.restyle();
    let admitted = session
        .layout
        .relayout_borrowed_dom(
            &session.dom,
            &session.styles_set,
            session.session.styles(),
            &mut session.styles,
            session.session.restyled_elements(),
            ViewportSizes::uniform(WIDTH + 1.0, HEIGHT),
            &mutations,
            &mut session.text,
            &HashMap::<String, Vec<u8>>::new(),
        )
        .unwrap();
    assert!(!admitted);
}

/// The route rests on the restyled set: a pass that restyled the whole
/// document reports none, and the batch falls back.
#[test]
fn scroll_batch_refuses_without_a_restyled_set() {
    let mut session = Session::new(&table_html(FIRST));
    session.session.invalidate();
    session.refused(|dom| scroll_by_one(dom, FIRST));
}
