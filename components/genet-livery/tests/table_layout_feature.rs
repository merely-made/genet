// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `without-table-layout`: a consumer that never lays out a table compiles the
//! table pipeline out. Cells then lay out as the blocks an unknown display
//! would give, and a default build still puts them side by side. The same
//! document is checked both ways, so one `cargo test` and one
//! `cargo test --features without-table-layout` cover the pair.

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

type NodeId = <StaticDocument as LayoutDom>::NodeId;

fn find(dom: &StaticDocument, node: NodeId, id: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, id))
}

fn cells() -> ([f32; 4], [f32; 4]) {
    let document = StaticDocument::parse(
        "<html><body><table id=t><tr><td id=a>one</td><td id=b>two</td></tr></table></body></html>",
    );
    let a = find(&document, document.document(), "a").unwrap();
    let b = find(&document, document.document(), "b").unwrap();
    let styles = StyleSet::cambium(&[
        "table { display: table; width: 200px; } tr { display: table-row; } \
         td { display: table-cell; padding: 0; height: 20px; }",
    ]);
    let mut retained = LiveryDocument::new(document, styles, Device::screen(400.0, 300.0));
    retained.frame(400, 300).expect("a document with a table");
    (
        retained.fragment_rect(a).expect("first cell"),
        retained.fragment_rect(b).expect("second cell"),
    )
}

#[cfg(not(feature = "without-table-layout"))]
#[test]
fn cells_sit_side_by_side_in_a_default_build() {
    let (a, b) = cells();
    assert_eq!(a[1], b[1], "one row: {a:?} {b:?}");
    assert!(b[0] > a[0], "{a:?} {b:?}");
}

#[cfg(feature = "without-table-layout")]
#[test]
fn cells_stack_as_blocks_without_table_layout() {
    let (a, b) = cells();
    assert_eq!(a[0], b[0], "{a:?} {b:?}");
    assert!(
        b[1] >= a[1] + a[3],
        "second cell below the first: {a:?} {b:?}"
    );
}
