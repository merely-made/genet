// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The batch's box tree is spliced where Buckram can prove it equal to a
//! regeneration, and regenerated with a reason everywhere else. Debug builds
//! also compare every splice with a regeneration inside the route.

use buckram::ChildSpliceRefusal;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::LayoutDomMut;

use super::super::scroll_batch_boxes::{BoxSpliceRefusal, same_box_tree, take_box_splices};
use super::*;

fn spliced(count: usize) -> Vec<Result<(), BoxSpliceRefusal>> {
    vec![Ok(()); count]
}

fn regenerated(reason: BoxSpliceRefusal) -> Vec<Result<(), BoxSpliceRefusal>> {
    vec![Err(reason)]
}

/// A table whose body holds `before` ahead of the strip and whose strip
/// holds `lead` ahead of the rows.
fn table_with(first: usize, before: &str, lead: &str) -> String {
    let rows = (first..first + WINDOW).map(row_html).collect::<String>();
    format!(
        "<html><body>{before}<div id=body><div id=rows style=\"top: -{}px\">{lead}{rows}</div>\
         </div></body></html>",
        first * ROW
    )
}

/// A bare row element, not yet in the document.
fn bare_row(dom: &mut ScriptedDom, index: usize, style: &str) -> NodeId {
    let row = div(dom);
    dom.set_attribute(row, attr("class"), "row");
    dom.set_attribute(row, attr("id"), &format!("r{index}"));
    dom.set_attribute(row, attr("style"), style);
    row
}

#[test]
fn rows_entering_at_the_bottom_splice_over_several_frames() {
    let mut session = Session::new(&table_html(FIRST));
    take_box_splices();
    for frame in 0..4 {
        session.admitted(|dom| scroll_by_one(dom, FIRST + frame));
    }
    assert_eq!(take_box_splices(), spliced(4));
}

#[test]
fn rows_entering_at_the_top_splice() {
    let mut session = Session::new(&table_html(FIRST));
    take_box_splices();
    session.admitted(|dom| {
        scroll_strip(dom, FIRST - 2);
        for index in [FIRST + WINDOW - 1, FIRST + WINDOW - 2] {
            let leaving = row_node(dom, index);
            dom.remove(leaving);
        }
        let first = row_node(dom, FIRST);
        insert_row(dom, FIRST - 2, Some(first));
        insert_row(dom, FIRST - 1, Some(first));
    });
    assert_eq!(take_box_splices(), spliced(1));
}

#[test]
fn rows_only_leaving_or_only_entering_splice() {
    let mut session = Session::new(&table_html(FIRST));
    take_box_splices();
    session.admitted(|dom| {
        for index in [FIRST, FIRST + 3] {
            let leaving = row_node(dom, index);
            dom.remove(leaving);
        }
    });
    session.admitted(|dom| {
        let before = row_node(dom, FIRST + 4);
        insert_row(dom, FIRST + 3, Some(before));
    });
    assert_eq!(take_box_splices(), spliced(2));
}

#[test]
fn rows_under_a_contents_wrapper_regenerate_as_foreign_children() {
    let wrapped = format!("<div style=\"display: contents\">{}</div>", row_html(10));
    let mut session = Session::new(&table_with(FIRST, "", &wrapped));
    take_box_splices();
    session.admitted(|dom| {
        insert_row(dom, FIRST + WINDOW, None);
    });
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::ForeignChild)
    );
}

#[test]
fn a_strip_beside_table_parts_regenerates() {
    let before = "<div style=\"display: table-row\">part</div>";
    let mut session = Session::new(&table_with(FIRST, before, ""));
    take_box_splices();
    session.admitted(|dom| scroll_by_one(dom, FIRST));
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::Buckram(
            ChildSpliceRefusal::AncestorContext
        ))
    );
}

#[test]
fn an_entering_text_root_regenerates_and_the_route_refuses() {
    let mut session = Session::new(&table_html(FIRST));
    take_box_splices();
    session.refused(|dom| {
        let text = dom.create_text("loose");
        let strip = by_id(dom, "rows");
        dom.append_child(strip, text);
    });
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::EnteringRoot)
    );
}

#[test]
fn an_entering_row_without_a_box_regenerates_and_the_route_refuses() {
    let mut session = Session::new(&table_html(FIRST));
    take_box_splices();
    session.refused(|dom| {
        let row = bare_row(dom, 99, "display: none");
        let strip = by_id(dom, "rows");
        dom.append_child(strip, row);
    });
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::Buckram(
            ChildSpliceRefusal::NotOneOutOfFlowBox
        ))
    );
}

#[test]
fn an_in_flow_child_of_the_strip_regenerates_and_the_route_refuses() {
    let lead = "<div style=\"height: 4px\"></div>";
    let mut session = Session::new(&table_with(FIRST, "", lead));
    take_box_splices();
    session.refused(|dom| {
        insert_row(dom, FIRST + WINDOW, None);
    });
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::Buckram(ChildSpliceRefusal::InFlowSibling))
    );
}

#[test]
fn a_leaving_root_without_a_box_regenerates_and_the_route_refuses() {
    // Absolute with a definite size, so only its missing box refuses it.
    let lead = "<div id=wrap style=\"display: contents; position: absolute; width: 10px;         height: 10px\"></div>";
    let mut session = Session::new(&table_with(FIRST, "", lead));
    take_box_splices();
    session.refused(|dom| {
        let wrap = by_id(dom, "wrap");
        dom.remove(wrap);
    });
    assert_eq!(
        take_box_splices(),
        regenerated(BoxSpliceRefusal::LeavingRoot)
    );
}

#[test]
fn rows_entering_a_parent_without_a_box_regenerate_and_the_route_refuses() {
    let lead = "<div id=wrap style=\"display: contents\"></div>";
    let mut session = Session::new(&table_with(FIRST, "", lead));
    take_box_splices();
    session.refused(|dom| {
        let row = bare_row(dom, 99, "top: 0px");
        let wrap = by_id(dom, "wrap");
        dom.append_child(wrap, row);
    });
    assert_eq!(take_box_splices(), regenerated(BoxSpliceRefusal::ParentBox));
}

/// The debug check's comparison tells a changed tree from an equal one.
#[test]
fn the_box_tree_comparison_sees_a_removed_row() {
    let session = Session::new(&table_html(FIRST));
    let boxes = session.layout.boxes();
    let mut changed = boxes.clone();
    assert!(same_box_tree(boxes, &changed));
    let row = boxes.principal_box(row_node(&session.dom, FIRST)).unwrap();
    assert!(changed.remove_subtree(row));
    assert!(!same_box_tree(boxes, &changed));
}
