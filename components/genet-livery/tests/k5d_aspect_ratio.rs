// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

fn find(
    dom: &StaticDocument,
    node: <StaticDocument as LayoutDom>::NodeId,
    needle: &str,
) -> Option<<StaticDocument as LayoutDom>::NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(needle)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, needle))
}

fn document_rects(html: &str, ids: &[&str]) -> Vec<[f32; 4]> {
    let mut session = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[]),
        Device::screen(800.0, 600.0),
    );
    session.frame(800, 600).expect("frame");
    ids.iter()
        .map(|name| {
            let id = find(session.dom(), session.dom().document(), name).expect(name);
            session
                .fragment_rect(id)
                .unwrap_or_else(|| panic!("{name} has a fragment"))
        })
        .collect()
}

fn assert_rect(actual: [f32; 4], expected: [f32; 4], what: &str) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() <= 0.5),
        "{what}: expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn abspos_max_height_transfers_through_the_aspect_ratio_to_the_inline_size() {
    let item =
        "<div style=\"width:20px;height:10px;display:inline-block;vertical-align:bottom\"></div>";
    let html = format!(
        "<html><body style=\"margin:0\">\
         <div id=\"parent\" style=\"position:relative;height:100px\">\
           <div id=\"abs\" style=\"position:absolute;aspect-ratio:1/1;max-height:100%;background:green\">\
             {}\
           </div>\
         </div></body></html>",
        item.repeat(10)
    );
    let rects = document_rects(&html, &["parent", "abs"]);

    assert_rect(rects[0], [0.0, 0.0, 800.0, 100.0], "relative parent");
    assert_rect(rects[1], [0.0, 0.0, 100.0, 100.0], "aspect-ratio abspos");
}

#[test]
fn size_contained_abspos_uses_its_contain_intrinsic_size_before_ratio_clamping() {
    let html = "<html><body style=\"margin:0\">\
         <div id=\"parent\" style=\"position:relative;height:100px\">\
           <div id=\"abs\" style=\"position:absolute;aspect-ratio:1/1;max-height:100%;\
                min-height:0;contain:size;contain-intrinsic-size:500px 500px\"></div>\
         </div></body></html>";
    let rects = document_rects(html, &["parent", "abs"]);

    assert_rect(rects[0], [0.0, 0.0, 800.0, 100.0], "relative parent");
    assert_rect(
        rects[1],
        [0.0, 0.0, 100.0, 100.0],
        "size-contained aspect-ratio abspos",
    );
}

fn single_line_contained_field_width(
    value: &str,
    host_width: Option<&str>,
    width: Option<&str>,
) -> f32 {
    let host_size = host_width.map_or(String::new(), |size| {
        format!("--cambium-field-intrinsic-width:{size};")
    });
    let explicit_width = width.map_or(String::new(), |width| format!("width:{width};"));
    let html = format!(
        "<html><body style=\"margin:0\"><div id=\"field\" role=\"textbox\" \
         style=\"position:absolute;left:10px;top:10px;{explicit_width}padding:4px 8px;\
         border:1px solid black;font-size:16px;white-space:pre;overflow-x:auto;\
         overflow-y:hidden;contain:inline-size;{host_size}contain-intrinsic-size:\
         var(--cambium-field-intrinsic-width,10em) 1.2em;\">{value}<span> preedit</span>\
         <span>▍</span></div></body></html>"
    );
    document_rects(&html, &["field"])[0][2]
}

#[test]
fn abspos_single_line_field_uses_contained_intrinsic_width_with_descendants() {
    let short = "notes.djot";
    let long = "C:/Users/someone/AppData/Local/Temp/a/very/long/folder/structure/that/keeps/going/document.djot";

    for value in [short, long] {
        for (host_width, width, expected) in [
            (None, None, 178.0),
            (Some("12em"), None, 210.0),
            (Some("12em"), Some("200px"), 218.0),
        ] {
            let actual = single_line_contained_field_width(value, host_width, width);
            assert!(
                (actual - expected).abs() <= 0.5,
                "value {value:?}, host width {host_width:?}, explicit width {width:?}: expected {expected}px, got {actual}px"
            );
        }
    }
}
