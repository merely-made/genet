// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An empty inline, or one whose white space collapses away, keeps a
//! zero-width box at its place in the line (CSS 2.1 10.8), and adding it moves
//! no line.

use std::collections::HashMap;

use genet_livery::{
    InteractionStates, LiveryLayout, StylePlane, StyleSet, TextSystem, ViewportSizes, inline_boxes,
    layout_with_text_system, resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::media::Device;

type Node = <StaticDocument as LayoutDom>::NodeId;

struct Fixture {
    dom: StaticDocument,
    styles: StylePlane<Node>,
    fragments: LiveryLayout<Node>,
}

impl Fixture {
    fn new(html: &str) -> Self {
        let dom = StaticDocument::parse(html);
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[
                "html, body { margin: 0; padding: 0; }                  div { width: 160px; font: 10px monospace; }",
            ]),
            &Device::screen(200.0, 100.0),
            &InteractionStates::default(),
        );
        let (styles, fragments) = layout_with_text_system(
            &dom,
            &styles,
            200.0,
            100.0,
            ViewportSizes::uniform(200.0, 100.0),
            &mut ahem_text_system(),
            &HashMap::new(),
        )
        .unwrap();
        Self {
            dom,
            styles,
            fragments,
        }
    }

    fn boxes(&self, class: &str) -> Option<Vec<[f32; 4]>> {
        let node = self
            .dom
            .first_with_class(self.dom.document(), class)
            .unwrap();
        inline_boxes(&self.dom, &self.styles, &self.fragments, node)
    }

    fn div_height(&self) -> f32 {
        let div = self.dom.first_with_class(self.dom.document(), "c").unwrap();
        self.fragments.get(div).unwrap().height
    }
}

/// Every generic family resolves to Ahem, one em per glyph.
fn ahem_text_system() -> TextSystem {
    let ahem = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf");
    let mut text = TextSystem::new();
    for generic in ["serif", "sans-serif", "monospace", "system-ui"] {
        text.register_generic_font_bytes(ahem.to_vec(), generic)
            .unwrap();
    }
    text
}

/// The target's one box: zero wide, at the reference's start and on the
/// reference's content area.
fn assert_at_start_of_ref(html: &str) {
    let fixture = Fixture::new(html);
    let reference = fixture.boxes("r").unwrap()[0];
    let target = fixture.boxes("t").expect("empty inline has a box");
    assert_eq!(target.len(), 1, "{target:?}");
    let [x, y, width, height] = target[0];
    assert_eq!(width, 0.0);
    assert_eq!((x, y, height), (reference[0], reference[1], reference[3]));
}

#[test]
fn empty_inline_after_a_forced_break() {
    assert_at_start_of_ref(
        r#"<div class="c"><br><span class="t"></span><span class="r">ref</span></div>"#,
    );
}

#[test]
fn empty_inline_after_text() {
    assert_at_start_of_ref(
        r#"<div class="c">offset<br>offset<span class="t"></span><span class="r">ref</span></div>"#,
    );
}

#[test]
fn empty_inline_in_rtl_between_latin_runs() {
    assert_at_start_of_ref(
        r#"<div class="c" style="direction: rtl">offset<br>offset<span class="t"></span><span class="r">ref</span></div>"#,
    );
}

#[test]
fn collapsed_whitespace_only_inline_is_empty() {
    assert_at_start_of_ref(
        r#"<div class="c"><br><span class="t"> </span><span class="r">ref</span></div>"#,
    );
}

#[test]
fn empty_inline_under_explicit_line_height() {
    assert_at_start_of_ref(
        r#"<div class="c" style="line-height: 30px">a<span class="t"></span><span class="r">ref</span></div>"#,
    );
}

#[test]
fn empty_inline_at_paragraph_end_follows_the_last_glyph() {
    let fixture =
        Fixture::new(r#"<div class="c"><span class="r">ref</span><span class="t"></span></div>"#);
    let reference = fixture.boxes("r").unwrap()[0];
    let [x, y, width, _] = fixture.boxes("t").unwrap()[0];
    assert_eq!(
        (x, y, width),
        (reference[0] + reference[2], reference[1], 0.0)
    );
}

/// Placed after line breaking, the box adds no line height of its own; CSS 2.1
/// 10.8 would give it one, which the post-break marker forgoes so that paint
/// cannot move.
#[test]
fn empty_inline_adds_no_line_height() {
    let plain = Fixture::new(r#"<div class="c">a</div>"#).div_height();
    let tall =
        Fixture::new(r#"<div class="c">a<span class="t" style="font-size: 40px"></span></div>"#);
    assert!(tall.boxes("t").is_some());
    assert_eq!(tall.div_height(), plain);
    let phantom = Fixture::new(r#"<div class="c"><span class="t"></span></div>"#);
    assert_eq!(phantom.div_height(), 0.0);
}

/// A line holding only an empty inline stays phantom, so a new formatting
/// context that cannot fit beside a float still clears it (WPT
/// css/CSS2/floats/new-fc-separates-from-float-4).
#[test]
fn a_line_of_one_empty_inline_stays_phantom() {
    let fixture = Fixture::new(
        r#"<div class="c" style="overflow: hidden; width: 200px"><div><div><div style="float: left; width: 200px; height: 200px"></div></div><span></span><div class="n" style="margin-top: 200px; overflow: hidden; width: 200px; height: 1px"></div></div></div>"#,
    );
    let next = fixture
        .dom
        .first_with_class(fixture.dom.document(), "n")
        .unwrap();
    assert_eq!(fixture.fragments.get(next).unwrap().y, 200.0);
}
