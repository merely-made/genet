/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Grid roles and table properties: from ARIA tokens and native tables
//! (WAI-ARIA 1.2, HTML-AAM), their AccessKit lowering, and the session.

use accesskit::{Role, SortDirection};
use document_session_api::{
    DocumentA11yCount as Count, DocumentA11yNode, DocumentA11yProjection, DocumentA11yRole,
    DocumentA11ySort, DocumentA11yTable,
};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};

use super::session::Refusal;
use super::session_tests::{Host, append_html, host_with};
use super::table_fixture::{attr, by_id, table_dom};
use super::{accesskit_tree_from_projection, document_a11y_projection};
use crate::fragments_from_scripted_dom;

const SHEET: &[&str] = &["div, span { display: block; }"];

fn projection_for(html: &str) -> (ScriptedDom, DocumentA11yProjection) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root, html);
    let fragments = fragments_from_scripted_dom(&dom, SHEET, 600, 400).expect("layout");
    let projection = document_a11y_projection(&dom, &fragments, None, 1);
    (dom, projection)
}

/// The projected node for the element with `id`.
fn node<'a>(
    dom: &ScriptedDom,
    projection: &'a DocumentA11yProjection,
    id: &str,
) -> &'a DocumentA11yNode {
    let id = document_session_api::DocumentA11yNodeId::new(dom.opaque_id(by_id(dom, id)));
    projection
        .node(id)
        .unwrap_or_else(|| panic!("{id:?} projected"))
}

fn roles(html: &str, ids: &[&str]) -> Vec<DocumentA11yRole> {
    let (dom, projection) = projection_for(html);
    ids.iter()
        .map(|id| node(&dom, &projection, id).role)
        .collect()
}

fn table(html: &str, id: &str) -> DocumentA11yTable {
    let (dom, projection) = projection_for(html);
    node(&dom, &projection, id).table
}

#[test]
fn each_grid_role_comes_from_its_token() {
    use DocumentA11yRole::*;
    let html = "<div role=grid id=g><div role=row><div role=columnheader id=c>C</div>\
        <div role=rowheader id=r>R</div><div role=gridcell id=x>X</div></div></div>\
        <div role=TreeGrid id=t></div>";
    assert_eq!(
        roles(html, &["g", "c", "r", "x", "t"]),
        [Grid, ColumnHeader, RowHeader, GridCell, TreeGrid]
    );
}

#[test]
fn native_th_follows_its_scope() {
    use DocumentA11yRole::*;
    let html = "<table><tr><td>a<th scope=COL id=a>b<th scope=colgroup id=b>c</tr>\
        <tr><td>d<th scope=row id=c>e<th scope=rowgroup id=d>f</tr></table>";
    assert_eq!(
        roles(html, &["a", "b", "c", "d"]),
        [ColumnHeader, ColumnHeader, RowHeader, RowHeader]
    );
}

#[test]
fn native_th_without_scope_follows_html_header_cells() {
    use DocumentA11yRole::*;
    // The WPT html-aam/table-roles.html table.
    let html = "<table><thead><tr><th id=a>a<th>b<th>c</tr></thead>\
        <tbody><tr><th id=b>1<td id=c>2<td>3</tr><tr><th>4<td>5<td>6</tr></tbody>\
        <tfoot><tr><th id=d>x<th>y<th>z</tr></tfoot></table>";
    assert_eq!(
        roles(html, &["a", "b", "c", "d"]),
        [ColumnHeader, RowHeader, Cell, ColumnHeader]
    );
    // Data cells in both its row and its column: neither header.
    let html = "<table><tr><td>a<th id=n>b</tr><tr><td>c<td>d</tr></table>";
    assert_eq!(roles(html, &["n"]), [Cell]);
    // An invalid scope is the auto state.
    let html = "<table><tr><th scope=sideways id=s>h</tr><tr><td>1</tr></table>";
    assert_eq!(roles(html, &["s"]), [ColumnHeader]);
}

#[test]
fn spans_place_cells_in_the_table_model() {
    use DocumentA11yRole::*;
    // A rowspan keeps the second row's data cell out of the header's column.
    let html = "<table><tr><th rowspan=2 id=h>h<td>1</tr><tr><td>2</tr></table>";
    assert_eq!(roles(html, &["h"]), [RowHeader]);
    // A colspan header over a row of data is a column header.
    let html = "<table><tr><th colspan=2 id=h>h</tr><tr><td>1<td>2</tr></table>";
    assert_eq!(roles(html, &["h"]), [ColumnHeader]);
}

#[test]
fn cells_in_a_grid_table_are_grid_cells() {
    use DocumentA11yRole::*;
    let html = "<table role=grid id=t><tr><th id=h>h<th>i</tr>\
        <tr><td id=d>1<th id=n>2</tr><tr><td>3<td>4</tr></table>";
    assert_eq!(
        roles(html, &["t", "h", "d", "n"]),
        [Grid, ColumnHeader, GridCell, GridCell]
    );
    let html = "<table role=treegrid><tr><td id=d>1</tr></table>";
    assert_eq!(roles(html, &["d"]), [GridCell]);
}

#[test]
fn properties_come_from_their_aria_attributes() {
    let html = "<div role=grid id=g aria-rowcount=10001 aria-colcount=-1>\
        <div role=row id=r aria-rowindex=5><div role=columnheader id=c aria-colindex=2 \
        aria-colspan=2 aria-rowspan=0 aria-sort=Ascending>C</div>\
        <div role=columnheader id=n aria-sort=none>N</div></div></div>";
    let (dom, projection) = projection_for(html);
    let grid = node(&dom, &projection, "g").table;
    assert_eq!(grid.row_count, Some(Count::Known(10001)));
    assert_eq!(grid.column_count, Some(Count::Unknown));
    assert_eq!(node(&dom, &projection, "r").table.row_index, Some(5));
    let header = node(&dom, &projection, "c").table;
    assert_eq!(header.column_index, Some(2));
    assert_eq!((header.column_span, header.row_span), (Some(2), Some(1)));
    assert_eq!(header.sort, Some(DocumentA11ySort::Ascending));
    let unsorted = node(&dom, &projection, "n").table;
    assert_eq!(unsorted.sort, Some(DocumentA11ySort::None));
}

#[test]
fn invalid_values_are_absent() {
    let html = "<div role=grid id=a aria-rowcount=-2 aria-colcount=1.5>\
        <div role=row id=b aria-rowindex=0 aria-colindex=x>\
        <div role=gridcell id=c aria-colspan=-1 aria-rowspan=two aria-sort=up>c</div>\
        </div></div><div role=grid id=z aria-rowcount=0></div>";
    for id in ["a", "b", "c"] {
        assert_eq!(table(html, id), DocumentA11yTable::default(), "{id}");
    }
    // A count of 0 is an integer and is taken as authored.
    assert_eq!(table(html, "z").row_count, Some(Count::Known(0)));
}

#[test]
fn native_spans_are_clamped_and_replace_aria_spans() {
    let html = "<table><tr><td id=a colspan=3 rowspan=2>a<td id=b colspan=0 rowspan=0>b\
        <td id=c colspan=5000 rowspan=70000>c<td id=d colspan=\" 2abc\">d\
        <td id=e aria-colspan=4 aria-rowspan=4>e<td id=f colspan=2 aria-colspan=4>f</tr></table>";
    let spans = |id| {
        let table = table(html, id);
        (table.column_span, table.row_span)
    };
    assert_eq!(spans("a"), (Some(3), Some(2)));
    assert_eq!(spans("b"), (Some(1), Some(1)));
    assert_eq!(spans("c"), (Some(1000), Some(65534)));
    assert_eq!(spans("d"), (Some(2), None));
    assert_eq!(spans("e"), (None, None));
    assert_eq!(spans("f"), (Some(2), None));
}

#[test]
fn roles_and_properties_lower_to_accesskit() {
    let html = "<div role=grid id=g aria-rowcount=10001 aria-colcount=-1>\
        <div role=row id=r aria-rowindex=5><div role=columnheader id=c aria-colindex=2 \
        aria-colspan=2 aria-sort=descending>C</div><div role=rowheader id=h \
        aria-sort=none>H</div><div role=gridcell id=x aria-rowspan=3>X</div></div></div>\
        <div role=treegrid id=t></div>";
    let (dom, projection) = projection_for(html);
    let tree = accesskit_tree_from_projection(projection);
    let lowered = |id: &str| {
        let id = accesskit::NodeId(dom.opaque_id(by_id(&dom, id)));
        tree.nodes
            .iter()
            .find(|(node, _)| *node == id)
            .map(|(_, node)| node.clone())
            .expect("lowered")
    };
    let grid = lowered("g");
    assert_eq!(grid.role(), Role::Grid);
    assert_eq!((grid.row_count(), grid.column_count()), (Some(10001), None));
    assert_eq!(lowered("r").row_index(), Some(4), "zero-based");
    let header = lowered("c");
    assert_eq!(header.role(), Role::ColumnHeader);
    assert_eq!(header.column_index(), Some(1));
    assert_eq!(header.column_span(), Some(2));
    assert_eq!(header.sort_direction(), Some(SortDirection::Descending));
    let row_header = lowered("h");
    assert_eq!(row_header.role(), Role::RowHeader);
    assert_eq!(row_header.sort_direction(), None, "none lowers as absent");
    let cell = lowered("x");
    assert_eq!(cell.role(), Role::GridCell);
    assert_eq!(cell.row_span(), Some(3));
    assert_eq!(lowered("t").role(), Role::TreeGrid);
}

#[test]
fn grids_take_no_name_from_their_content() {
    let html = "<div role=grid id=g>Direct text</div>\
        <table role=grid id=t><caption>Nodes</caption><tr><td>1</tr></table>";
    let (dom, projection) = projection_for(html);
    assert_eq!(node(&dom, &projection, "g").name, None);
    assert_eq!(node(&dom, &projection, "t").name.as_deref(), Some("Nodes"));
}

#[test]
fn the_table_fixture_projects_a_grid_with_its_positions() {
    let host = Host::new(table_dom(0));
    let projection = host.session.projection();
    let dom = &host.dom;
    let role_of = |role| {
        projection
            .nodes()
            .iter()
            .filter(move |node| node.role == role)
    };
    let grid: Vec<_> = role_of(DocumentA11yRole::Grid).collect();
    assert_eq!(grid.len(), 1);
    assert_eq!(grid[0].table.row_count, Some(Count::Known(10001)));
    assert_eq!(grid[0].name.as_deref(), Some("Nodes"));
    let headers: Vec<_> = role_of(DocumentA11yRole::ColumnHeader).collect();
    assert_eq!(headers.len(), super::table_fixture::CELLS);
    assert_eq!(headers[0].table.column_index, Some(1));
    let row = node(dom, &projection, "r3");
    assert_eq!(row.table.row_index, Some(5));
    let cells: Vec<_> = row
        .children
        .iter()
        .map(|id| projection.node(*id).expect("cell"))
        .collect();
    assert!(cells.iter().all(|c| c.role == DocumentA11yRole::GridCell));
    assert_eq!(cells[7].table.column_index, Some(8));
}

fn native_host() -> Host {
    host_with(|dom| {
        append_html(
            dom,
            "<table id=nt><tr><th id=nh>h<td>1</tr><tr id=nr><th>x</tr></table>             <div id=out>o</div>",
        )
    })
}

/// The header in the first row is a row header until a data cell enters the
/// second row ahead of its `th`: a change outside the header's own chain.
#[test]
fn a_change_inside_a_native_table_rebuilds() {
    let mut host = native_host();
    let header = |host: &Host| {
        let projection = host.session.projection();
        node(&host.dom, &projection, "nh").role
    };
    assert_eq!(header(&host), DocumentA11yRole::RowHeader);
    host.refused(Refusal::NativeTable, |dom| {
        let row = by_id(dom, "nr");
        let cell = dom.create_element(layout_dom_api::QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from("td"),
        ));
        let first = dom.dom_children(row).next();
        dom.insert_before(row, cell, first);
    });
    assert_eq!(header(&host), DocumentA11yRole::Cell);
    host.refused(Refusal::NativeTable, |dom| {
        let th = by_id(dom, "nh");
        dom.set_attribute(th, attr("scope"), "row");
    });
}

#[test]
fn a_role_written_on_a_native_table_rebuilds() {
    let mut host = native_host();
    host.refused(Refusal::NativeTable, |dom| {
        let table = by_id(dom, "nt");
        dom.set_attribute(table, attr("role"), "grid");
    });
    assert!(
        host.session
            .projection()
            .nodes()
            .iter()
            .any(|node| node.role == DocumentA11yRole::Grid)
    );
}

#[test]
fn a_change_beside_a_native_table_still_patches() {
    let mut host = native_host();
    host.admitted(|dom| {
        let out = by_id(dom, "out");
        dom.set_attribute(out, attr("title"), "outside");
    });
}
