// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CSS 2.1 section 10.8's line box model against Chromium 152, in standards
//! and quirks mode. Each case is one 160px block in Ahem 16px/20px, laid out
//! alone; the expectations are Chromium's `getBoundingClientRect` heights and
//! atom offsets for the same markup (design_docs/2026-09-25_line_box_model_plan.md).

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

const CSS: &str = "body { margin: 0; } .case { width: 160px; font: 16px/20px Ahem; } \
    .ib { display: inline-block; width: 15px; height: 10px; border: 1px solid; } \
    .tall { display: inline-block; width: 20px; height: 40px; } \
    .wide { display: inline-block; width: 80px; height: 40px; }";

/// A case's markup, then Chromium's block height and atom offset.
type Case = (&'static str, &'static str, f32, Option<f32>);

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

/// The block's height, and its `#atom`'s offset from the block's top.
fn measure(doctype: &str, inner: &str) -> (f32, Option<f32>) {
    let html = format!("{doctype}<html><body><div class=case id=case>{inner}</div></body></html>");
    let mut session = LiveryDocument::new(
        StaticDocument::parse(&html),
        StyleSet::cambium(&[CSS]),
        Device::screen(320.0, 240.0),
    );
    session.set_font_resource(
        "/fonts/Ahem.ttf",
        include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf").to_vec(),
    );
    session.frame(320, 240).expect("frame");
    let rect = |id| {
        find(session.dom(), session.dom().document(), id)
            .and_then(|node| session.fragment_rect(node))
    };
    let [_, top, _, height] = rect("case").expect("case fragment");
    (height, rect("atom").map(|[_, atom, _, _]| atom - top))
}

fn assert_matches_chromium(doctype: &str, cases: &[Case]) {
    let failures = cases
        .iter()
        .filter_map(|(id, inner, height, atom)| {
            let (got, got_atom) = measure(doctype, inner);
            let atom_matches = match (atom, got_atom) {
                (Some(atom), Some(got_atom)) => (atom - got_atom).abs() <= 0.5,
                (atom, got_atom) => atom.is_none() && got_atom.is_none(),
            };
            ((got - height).abs() > 0.5 || !atom_matches).then(|| {
                format!("{id}: height {got}, atom {got_atom:?}; Chromium {height}, {atom:?}")
            })
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn standards_mode_line_boxes_match_chromium() {
    assert_matches_chromium(
        "<!DOCTYPE html>",
        &[
            // Knot's gap: an atom on a wrapped line sits below the line above.
            (
                "wrapped-atom",
                "aaaa bbbb cccc <span class=wide id=atom></span> dddd eeee ffff",
                105.0,
                Some(20.0),
            ),
            (
                "atom-one-line",
                "x <span class=wide id=atom></span> x",
                45.0,
                Some(0.0),
            ),
            (
                "super",
                "x <span style='vertical-align: super'>x</span> x",
                26.33,
                None,
            ),
            (
                "sub",
                "x <span style='vertical-align: sub'>x</span> x",
                24.19,
                None,
            ),
            (
                "length",
                "x <span style='vertical-align: 10px'>x</span> x",
                30.0,
                None,
            ),
            (
                "big-font",
                "x <span style='font-size: 32px'>x</span> x",
                25.0,
                None,
            ),
            (
                "empty-tall",
                "x<span style='line-height: 40px'></span>",
                40.0,
                None,
            ),
            (
                "nested-tall",
                "x <span style='line-height: 40px'><span style='line-height: 10px'>x</span></span>",
                40.0,
                None,
            ),
            (
                "atom-top",
                "x <span class=wide id=atom style='vertical-align: top'></span>",
                40.0,
                Some(0.0),
            ),
            (
                "atom-bottom",
                "x <span class=wide id=atom style='vertical-align: bottom'></span>",
                40.0,
                Some(0.0),
            ),
            (
                "atom-middle",
                "x <span class=wide id=atom style='vertical-align: middle'></span>",
                40.0,
                Some(0.0),
            ),
            (
                "atom-text-top",
                "x <span class=wide id=atom style='vertical-align: text-top'></span>",
                42.0,
                Some(2.0),
            ),
            (
                "atom-text-bottom",
                "x <span class=wide id=atom style='vertical-align: text-bottom'></span>",
                42.0,
                Some(0.0),
            ),
            (
                "atoms-only",
                "<span class=ib></span><span class=ib></span>",
                20.0,
                None,
            ),
            (
                "tall-atom-only",
                "<span class=tall id=atom></span>",
                45.0,
                Some(0.0),
            ),
            (
                "raised-atom-only",
                "<span class=tall id=atom style='vertical-align: 20px'></span>",
                65.0,
                Some(0.0),
            ),
            (
                "br-in-tall-span",
                "<span style='line-height: 40px'><br></span>",
                40.0,
                None,
            ),
            // Nested boxes align within their parent: a `top` or `bottom` box
            // places its whole aligned subtree, and raises chain.
            (
                "top-subtree",
                "x <span style='vertical-align: top'><span style='font-size: 40px'>x</span>\
                 <span class=ib id=atom></span></span>",
                27.0,
                Some(10.0),
            ),
            (
                "bottom-subtree",
                "x <span style='vertical-align: bottom'><span style='font-size: 40px'>x</span>\
                 <span class=ib id=atom></span></span>",
                27.0,
                Some(10.0),
            ),
            (
                "sub-in-super",
                "x <span style='vertical-align: super'>x<span style='vertical-align: sub'>\
                 <span class=ib id=atom></span></span></span>",
                26.33,
                Some(7.19),
            ),
            (
                "text-top-in-big-span",
                "x <span style='font-size: 40px'>x\
                 <span class=ib id=atom style='vertical-align: text-top'></span></span>",
                37.0,
                Some(0.0),
            ),
            (
                "middle-in-big-span",
                "x <span style='font-size: 40px'>x\
                 <span class=ib id=atom style='vertical-align: middle'></span></span>",
                27.0,
                Some(0.0),
            ),
        ],
    );
}

/// The line height calculation quirk: a box counts only where it holds text
/// directly or has inline-axis margins, borders or padding.
#[test]
fn quirks_mode_line_boxes_match_chromium() {
    assert_matches_chromium(
        "",
        &[
            (
                "small-span",
                "<span style='font-size: 10px'>x</span>",
                20.0,
                None,
            ),
            (
                "small-span-root-text",
                "x<span style='font-size: 10px'>x</span>",
                22.0,
                None,
            ),
            (
                "nested-tall-alone",
                "<span style='line-height: 40px'><span style='line-height: 10px'>x</span></span>",
                10.0,
                None,
            ),
            (
                "nested-tall",
                "x <span style='line-height: 40px'><span style='line-height: 10px'>x</span></span>",
                20.0,
                None,
            ),
            (
                "empty-tall",
                "x<span style='line-height: 40px'></span>",
                20.0,
                None,
            ),
            (
                "atom-in-tall-span",
                "<span style='line-height: 40px'><span class=ib></span></span>",
                12.0,
                None,
            ),
            (
                "atom-middle-alone",
                "<span class=tall id=atom style='vertical-align: middle'></span>",
                40.0,
                Some(0.0),
            ),
            (
                "edge-only",
                "<span style='padding-left: 5px; line-height: 40px'></span>",
                40.0,
                None,
            ),
            ("br-only", "<br>", 20.0, None),
            (
                "br-in-tall-span",
                "<span style='line-height: 40px'><br></span>",
                40.0,
                None,
            ),
            (
                "big-font-alone",
                "<span style='font-size: 32px'>x</span>",
                20.0,
                None,
            ),
            (
                "atoms-only",
                "<span class=ib></span><span class=ib></span>",
                12.0,
                None,
            ),
            (
                "tall-atom-only",
                "<span class=tall id=atom></span>",
                40.0,
                Some(0.0),
            ),
            (
                "raised-atom-only",
                "<span class=tall id=atom style='vertical-align: 20px'></span>",
                40.0,
                Some(0.0),
            ),
            ("text-and-atom", "x<span class=ib></span>", 20.0, None),
            // A forced break brings the strut only to an otherwise empty line,
            // but always brings its own inline box.
            ("atom-br", "<span class=ib></span><br>", 12.0, None),
            ("atom-space-br", "<span class=ib></span> <br>", 12.0, None),
            (
                "atom-tall-br",
                "<span class=ib></span><span style='line-height: 40px'><br></span>",
                40.0,
                None,
            ),
            ("br-br", "<br><br>", 40.0, None),
            // Inline-axis borders and padding make a box count; margins do not.
            (
                "margin-span",
                "<span style='margin: 0 5px; line-height: 40px'></span>",
                0.0,
                None,
            ),
            (
                "padding-span",
                "<span style='padding: 0 5px; line-height: 40px'></span>",
                40.0,
                None,
            ),
        ],
    );
}
