/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tables and grids: native cell roles by the HTML table model (HTML-AAM)
//! and the authored row and column properties (WAI-ARIA 1.2).

use std::collections::HashSet;

#[cfg(feature = "accesskit")]
use accesskit::{Node as AccessNode, SortDirection};
use document_session_api::{
    DocumentA11yCount, DocumentA11yRole, DocumentA11ySort, DocumentA11yTable,
};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

const HTML: &str = "http://www.w3.org/1999/xhtml";
/// HTML's clamps for `colspan` and `rowspan`.
const MAX_COLSPAN: u32 = 1000;
const MAX_ROWSPAN: u32 = 65534;

fn attr<'a, D: LayoutDom>(dom: &'a D, node: D::NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
}

/// The local name of `node` when it is an HTML element.
pub(super) fn html_tag<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<&str> {
    dom.element_name(node)
        .filter(|name| name.ns.as_str() == HTML)
        .map(|name| name.local.as_str())
}

/// The `table` that owns row `row` directly or through a row group.
fn row_table<D: LayoutDom>(dom: &D, row: D::NodeId) -> Option<D::NodeId> {
    let parent = dom.parent(row)?;
    match html_tag(dom, parent)? {
        "table" => Some(parent),
        "thead" | "tbody" | "tfoot" => dom
            .parent(parent)
            .filter(|table| html_tag(dom, *table) == Some("table")),
        _ => None,
    }
}

/// HTML's rules for parsing non-negative integers: leading whitespace, an
/// optional sign, digits, and anything after them ignored.
fn html_non_negative(value: &str) -> Option<u64> {
    let rest = value.trim_start_matches(['\t', '\n', '\x0C', '\r', ' ']);
    let (negative, rest) = match rest.as_bytes().first() {
        Some(b'-') => (true, &rest[1..]),
        Some(b'+') => (false, &rest[1..]),
        _ => (false, rest),
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit);
    let mut seen = false;
    let mut out = 0u64;
    for digit in digits {
        seen = true;
        out = out
            .saturating_mul(10)
            .saturating_add(u64::from(digit - b'0'));
    }
    (seen && (!negative || out == 0)).then_some(out)
}

/// A native span: absent or invalid is 1, 0 is 1 (the row-group rule for
/// `rowspan=0` is not modelled), then HTML's clamp.
fn native_span(value: Option<&str>, max: u32) -> u32 {
    value
        .and_then(html_non_negative)
        .map_or(1, |span| span.clamp(1, u64::from(max)) as u32)
}

fn aria_integer<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<i64> {
    attr(dom, node, name).and_then(|value| value.trim().parse().ok())
}

fn aria_count<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<DocumentA11yCount> {
    match aria_integer(dom, node, name)? {
        -1 => Some(DocumentA11yCount::Unknown),
        count if count >= 0 => Some(DocumentA11yCount::Known(
            u32::try_from(count).unwrap_or(u32::MAX),
        )),
        _ => None,
    }
}

fn aria_index<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<u32> {
    aria_integer(dom, node, name)
        .filter(|index| *index >= 1)
        .map(|index| u32::try_from(index).unwrap_or(u32::MAX))
}

/// An ARIA span: below 0 is invalid, 0 is taken as 1.
fn aria_span<D: LayoutDom>(dom: &D, node: D::NodeId, name: &str) -> Option<u32> {
    aria_integer(dom, node, name)
        .filter(|span| *span >= 0)
        .map(|span| u32::try_from(span.max(1)).unwrap_or(u32::MAX))
}

fn aria_sort<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<DocumentA11ySort> {
    let value = attr(dom, node, "aria-sort")?.trim();
    [
        ("none", DocumentA11ySort::None),
        ("ascending", DocumentA11ySort::Ascending),
        ("descending", DocumentA11ySort::Descending),
        ("other", DocumentA11ySort::Other),
    ]
    .into_iter()
    .find(|(token, _)| value.eq_ignore_ascii_case(token))
    .map(|(_, sort)| sort)
}

/// The authored table properties of `node`. On `td`/`th` the native spans
/// replace `aria-colspan`/`aria-rowspan`, as WAI-ARIA requires.
pub(super) fn table_properties<D: LayoutDom>(dom: &D, node: D::NodeId) -> DocumentA11yTable {
    let native = matches!(html_tag(dom, node), Some("td" | "th"));
    let span = |native_name, max, aria_name| {
        if native {
            attr(dom, node, native_name).map(|value| native_span(Some(value), max))
        } else {
            aria_span(dom, node, aria_name)
        }
    };
    DocumentA11yTable {
        row_count: aria_count(dom, node, "aria-rowcount"),
        column_count: aria_count(dom, node, "aria-colcount"),
        row_index: aria_index(dom, node, "aria-rowindex"),
        column_index: aria_index(dom, node, "aria-colindex"),
        row_span: span("rowspan", MAX_ROWSPAN, "aria-rowspan"),
        column_span: span("colspan", MAX_COLSPAN, "aria-colspan"),
        sort: aria_sort(dom, node),
    }
}

/// One cell placed in the HTML table model.
struct Slotted<Id> {
    node: Id,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    data: bool,
}

/// The HTML table-forming algorithm, reduced to what header cells need:
/// each cell's slots, with `rowspan=0` taken as 1.
fn form_table<D: LayoutDom>(dom: &D, table: D::NodeId) -> Vec<Slotted<D::NodeId>> {
    let mut cells = Vec::new();
    let mut taken = HashSet::new();
    let (mut y, mut height) = (0usize, 0usize);
    let mut row = |row: D::NodeId, y: &mut usize, height: &mut usize| {
        if *height == *y {
            *height += 1;
        }
        let mut x = 0;
        for cell in dom.dom_children(row) {
            let data = match html_tag(dom, cell) {
                Some("td") => true,
                Some("th") => false,
                _ => continue,
            };
            while taken.contains(&(x, *y)) {
                x += 1;
            }
            let width = native_span(attr(dom, cell, "colspan"), MAX_COLSPAN) as usize;
            let tall = native_span(attr(dom, cell, "rowspan"), MAX_ROWSPAN) as usize;
            *height = (*height).max(*y + tall);
            for slot_x in x..x + width {
                for slot_y in *y..*y + tall {
                    taken.insert((slot_x, slot_y));
                }
            }
            cells.push(Slotted {
                node: cell,
                x,
                y: *y,
                width,
                height: tall,
                data,
            });
            x += width;
        }
        *y += 1;
    };
    for child in dom.dom_children(table) {
        match html_tag(dom, child) {
            Some("tr") => row(child, &mut y, &mut height),
            Some("thead" | "tbody" | "tfoot") => {
                y = height;
                for member in dom.dom_children(child) {
                    if html_tag(dom, member) == Some("tr") {
                        row(member, &mut y, &mut height);
                    }
                }
                y = height;
            },
            _ => {},
        }
    }
    cells
}

/// Whether a `th` is a column header (`Some(true)`), a row header
/// (`Some(false)`) or neither, by its `scope` and HTML's auto-state rule.
fn header_axis<D: LayoutDom>(dom: &D, table: D::NodeId, th: D::NodeId) -> Option<bool> {
    let scope = attr(dom, th, "scope").unwrap_or("");
    for (keyword, column) in [
        ("col", true),
        ("colgroup", true),
        ("row", false),
        ("rowgroup", false),
    ] {
        if scope.eq_ignore_ascii_case(keyword) {
            return Some(column);
        }
    }
    let cells = form_table(dom, table);
    let cell = cells.iter().find(|cell| cell.node == th)?;
    let overlaps = |a: usize, a_len: usize, b: usize, b_len: usize| a < b + b_len && b < a + a_len;
    let data_in_rows = cells
        .iter()
        .any(|d| d.data && overlaps(d.y, d.height, cell.y, cell.height));
    if !data_in_rows {
        return Some(true);
    }
    let data_in_columns = cells
        .iter()
        .any(|d| d.data && overlaps(d.x, d.width, cell.x, cell.width));
    (!data_in_columns).then_some(false)
}

/// The role of a native `td`/`th` (HTML-AAM): a header by HTML's header-cell
/// definitions, else a cell, a grid cell when its table is a grid or treegrid.
pub(super) fn native_cell_role<D: LayoutDom>(dom: &D, node: D::NodeId) -> DocumentA11yRole {
    let Some(table) = dom
        .parent(node)
        .filter(|row| html_tag(dom, *row) == Some("tr"))
        .and_then(|row| row_table(dom, row))
    else {
        return DocumentA11yRole::Cell;
    };
    if html_tag(dom, node) == Some("th") {
        match header_axis(dom, table, node) {
            Some(true) => return DocumentA11yRole::ColumnHeader,
            Some(false) => return DocumentA11yRole::RowHeader,
            None => {},
        }
    }
    match super::document_role(dom, table) {
        DocumentA11yRole::Grid | DocumentA11yRole::TreeGrid => DocumentA11yRole::GridCell,
        _ => DocumentA11yRole::Cell,
    }
}

/// Lower table properties to AccessKit: an unknown count and `aria-sort=none`
/// set nothing, and indexes become zero-based.
#[cfg(feature = "accesskit")]
pub(super) fn lower_table(access: &mut AccessNode, table: &DocumentA11yTable) {
    let known = |count: Option<DocumentA11yCount>| match count? {
        DocumentA11yCount::Known(count) => Some(count as usize),
        DocumentA11yCount::Unknown => None,
    };
    if let Some(count) = known(table.row_count) {
        access.set_row_count(count);
    }
    if let Some(count) = known(table.column_count) {
        access.set_column_count(count);
    }
    if let Some(index) = table.row_index {
        access.set_row_index((index as usize).saturating_sub(1));
    }
    if let Some(index) = table.column_index {
        access.set_column_index((index as usize).saturating_sub(1));
    }
    if let Some(span) = table.row_span {
        access.set_row_span(span as usize);
    }
    if let Some(span) = table.column_span {
        access.set_column_span(span as usize);
    }
    let sort = match table.sort {
        Some(DocumentA11ySort::Ascending) => Some(SortDirection::Ascending),
        Some(DocumentA11ySort::Descending) => Some(SortDirection::Descending),
        Some(DocumentA11ySort::Other) => Some(SortDirection::Other),
        Some(DocumentA11ySort::None) | None => None,
    };
    if let Some(sort) = sort {
        access.set_sort_direction(sort);
    }
}
