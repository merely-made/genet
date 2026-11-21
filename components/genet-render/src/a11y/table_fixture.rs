/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A document shaped like Graphshell's windowed table: a labelled grid, a
//! header row, and a strip of absolutely placed rows of ARIA cells.

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};

pub(super) const WINDOW: usize = 30;
pub(super) const CELLS: usize = 8;
pub(super) const ROW: usize = 24;
pub(super) const WIDTH: u32 = 800;
pub(super) const HEIGHT: u32 = 760;

pub(super) const CSS: &[&str] = &["html, body, div { display: block; } \
    html, body { margin: 0; padding: 0; font: 10px monospace; } \
    .grid-body { position: relative; overflow: hidden; width: 760px; height: 720px; } \
    .grid-rows { position: absolute; left: 0; width: 760px; height: 0px; } \
    .grid-row { position: absolute; left: 0; width: 760px; height: 24px; } \
    .grid-cell { position: absolute; top: 0; width: 95px; height: 24px; }"];

fn cell_html(row: usize, cell: usize) -> String {
    let tab = if cell == 0 { " tabindex=\"-1\"" } else { "" };
    format!(
        "<div class=grid-cell role=gridcell aria-colindex={} style=\"left: {}px\"{tab}>\
         <span>r{row} c{cell}</span></div>",
        cell + 1,
        cell * 95
    )
}

pub(super) fn row_html(row: usize) -> String {
    let cells = (0..CELLS)
        .map(|cell| cell_html(row, cell))
        .collect::<String>();
    format!(
        "<div class=\"grid-row\" role=row aria-rowindex={} data-row={row} id=r{row} \
         style=\"top: {}px\">{cells}</div>",
        row + 2,
        row * ROW
    )
}

pub(super) fn table_html(first: usize) -> String {
    let header = (0..CELLS)
        .map(|c| {
            format!(
                "<div role=columnheader aria-colindex={} tabindex=0>Col {c}</div>",
                c + 1
            )
        })
        .collect::<String>();
    let rows = (first..first + WINDOW).map(row_html).collect::<String>();
    format!(
        "<html><body><div role=grid aria-label=Nodes aria-rowcount=10001>\
         <div role=row aria-rowindex=1>{header}</div><div class=grid-body>\
         <div class=grid-rows id=strip style=\"top: -{}px\">{rows}</div></div></div>\
         </body></html>",
        first * ROW
    )
}

pub(super) fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

pub(super) fn by_id(dom: &ScriptedDom, id: &str) -> NodeId {
    let mut pending = vec![dom.document()];
    while let Some(node) = pending.pop() {
        if dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id) {
            return node;
        }
        pending.extend(dom.dom_children(node));
    }
    panic!("fixture node {id}")
}

fn element(dom: &mut ScriptedDom, local: &str) -> NodeId {
    dom.create_element(QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from(local),
    ))
}

/// Insert row `row` under the strip, attached first and filled in place as
/// Cambium's keyed rows are, so each node and attribute is its own mutation.
pub(super) fn insert_row(dom: &mut ScriptedDom, row: usize, before: Option<NodeId>) -> NodeId {
    let strip = by_id(dom, "strip");
    let node = element(dom, "div");
    dom.insert_before(strip, node, before);
    for (name, value) in [
        ("class", "grid-row".to_owned()),
        ("role", "row".to_owned()),
        ("aria-rowindex", (row + 2).to_string()),
        ("data-row", row.to_string()),
        ("id", format!("r{row}")),
        ("style", format!("top: {}px", row * ROW)),
    ] {
        dom.set_attribute(node, attr(name), &value);
    }
    for cell in 0..CELLS {
        let div = element(dom, "div");
        dom.append_child(node, div);
        for (name, value) in [
            ("class", "grid-cell".to_owned()),
            ("role", "gridcell".to_owned()),
            ("aria-colindex", (cell + 1).to_string()),
            ("style", format!("left: {}px", cell * 95)),
        ] {
            dom.set_attribute(div, attr(name), &value);
        }
        if cell == 0 {
            dom.set_attribute(div, attr("tabindex"), "-1");
        }
        let span = element(dom, "span");
        dom.append_child(div, span);
        let text = dom.create_text(&format!("r{row} c{cell}"));
        dom.append_child(span, text);
    }
    node
}

/// One scroll frame down: the strip moves a row, the top row leaves and the
/// next enters at the bottom.
pub(super) fn scroll_down(dom: &mut ScriptedDom, first: usize) {
    let strip = by_id(dom, "strip");
    dom.set_attribute(
        strip,
        attr("style"),
        &format!("top: -{}px", (first + 1) * ROW),
    );
    let leaving = by_id(dom, &format!("r{first}"));
    dom.remove(leaving);
    insert_row(dom, first + WINDOW, None);
}

pub(super) fn table_dom(first: usize) -> ScriptedDom {
    let mut dom = ScriptedDom::from_serialized_document(&table_html(first));
    dom.drain_mutations(&mut Vec::new());
    dom
}
