// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host-only stylesheet puts a vector face before a link with
//! `content: url(...)`, without touching the guest DOM.

use std::sync::{Arc, Mutex};

use genet_livery::{Device, LiveryDocument, StyleSet, VectorImage};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PaintList, PathCommand, PathData,
    PathItem,
};

const AHEM: &[u8] = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf");
/// Ahem's cap height and ascent are both 0.8 em.
const AHEM_CAP: f32 = 0.8;
const FACE: &str = "face:6162";
const FACE_RULE: &str = "height: round(nearest, max(8px, 1cap), 1px); width: auto; \
                         vertical-align: baseline";

#[derive(Debug, Default)]
struct Square {
    painted: Mutex<Vec<LayoutRect>>,
}

impl VectorImage for Square {
    fn intrinsic_size(&self) -> (f32, f32) {
        (64.0, 64.0)
    }

    fn paint(&self, rect: LayoutRect) -> Vec<PaintCmd> {
        self.painted.lock().unwrap().push(rect);
        let corner = |x, y| PathCommand::LineTo(LayoutPoint::new(x, y));
        vec![PaintCmd::DrawPath(PathItem {
            placement: CommonPlacement::new(rect),
            path: PathData {
                commands: vec![
                    PathCommand::MoveTo(rect.min),
                    corner(rect.max.x, rect.min.y),
                    corner(rect.max.x, rect.max.y),
                    corner(rect.min.x, rect.max.y),
                    PathCommand::Close,
                ],
            },
            fill: Some(ColorF::new(0.2, 0.4, 0.8, 1.0)),
            stroke: None,
        })]
    }
}

fn document(font_size: f32, rule: &str) -> LiveryDocument<StaticDocument> {
    let html = "<html><body><p>XX <a class=link href=\"x\">AB</a><span class=probe></span> XX</p>\
                </body></html>";
    let css = format!(
        "@font-face {{ font-family: ahem; src: url(https://site.test/ahem.ttf); }}\
         html, body, p {{ margin: 0; }}\
         p {{ font-family: ahem; font-size: {font_size}px; line-height: 2; }}\
         .probe {{ display: inline-block; width: 1px; height: 0; }}\
         a[href=\"x\"]::before {{ {rule} }}"
    );
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[&css]),
        Device::screen(400.0, 200.0),
    );
    document.set_font_resource("https://site.test/ahem.ttf", AHEM.to_vec());
    document
}

fn paths(document: &mut LiveryDocument<StaticDocument>) -> Vec<PathItem> {
    let frame = document.frame(400, 200).expect("paint frame");
    frame
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawPath(item) => Some(item.clone()),
            _ => None,
        })
        .collect()
}

fn rect_of(document: &LiveryDocument<StaticDocument>, class: &str) -> [f32; 4] {
    let dom = document.dom();
    let node = dom.first_with_class(dom.document(), class).unwrap();
    document.fragment_rect(node).expect("placed")
}

fn link_rect(document: &LiveryDocument<StaticDocument>) -> [f32; 4] {
    rect_of(document, "link")
}

#[test]
fn a_content_image_is_a_cap_height_square_on_the_baseline_before_the_link_text() {
    for font_size in [12.0, 14.0, 16.0] {
        let mut document = document(font_size, &format!("content: url({FACE}); {FACE_RULE}"));
        let square = Arc::new(Square::default());
        document.set_vector_image(FACE, Some(square.clone() as Arc<dyn VectorImage>));
        let drawn = paths(&mut document);
        let [_, probe_y, _, _] = rect_of(&document, "probe");
        let want = (font_size * AHEM_CAP).round().max(8.0);
        let at = format!("{font_size}px");
        assert_eq!(drawn.len(), 1, "{at}: painted once");
        let painted = square.painted.lock().unwrap().clone();
        assert_eq!(painted.len(), 1, "{at}: asked once");
        let rect = painted[0];
        assert!((rect.width() - want).abs() < 0.01, "{at}: {rect:?} want {want}");
        assert!((rect.height() - want).abs() < 0.01, "{at}: square {rect:?}");
        assert!(
            (rect.max.y - probe_y).abs() < 0.01,
            "{at}: bottom {} on baseline {probe_y}",
            rect.max.y
        );
        // The owner's text starts after the image: it is left of the link's
        // text, and the link (which holds both) is wider by the image.
        let [link_x, _, link_width, _] = link_rect(&document);
        assert!(rect.min.x >= link_x - 0.01, "{at}: face inside the link");
        assert!(
            (link_width - (2.0 * font_size + want)).abs() < 0.01,
            "{at}: link {link_width} = text + face"
        );
        let text_start = rect.max.x;
        assert!(
            (link_x + link_width - text_start - 2.0 * font_size).abs() < 0.01,
            "{at}: text follows the face"
        );
    }
}

#[test]
fn an_unregistered_url_draws_nothing_and_takes_no_room() {
    let mut document = document(16.0, &format!("content: url({FACE}); {FACE_RULE}"));
    assert!(paths(&mut document).is_empty());
    let [_, _, width, _] = link_rect(&document);
    assert!((width - 32.0).abs() < 0.01, "link is just its text: {width}");
}

#[test]
fn removing_the_vector_image_leaves_no_stale_paint_or_room() {
    let mut document = document(16.0, &format!("content: url({FACE}); {FACE_RULE}"));
    let square: Arc<dyn VectorImage> = Arc::new(Square::default());
    document.set_vector_image(FACE, Some(square));
    assert_eq!(paths(&mut document).len(), 1);
    document.set_vector_image(FACE, None);
    assert!(paths(&mut document).is_empty());
    assert!((link_rect(&document)[2] - 32.0).abs() < 0.01);
}

#[test]
fn mixed_content_keeps_its_string_and_ignores_the_image() {
    let mut document = document(16.0, &format!("content: \"Q\" url({FACE}); {FACE_RULE}"));
    let square = Arc::new(Square::default());
    document.set_vector_image(FACE, Some(square.clone() as Arc<dyn VectorImage>));
    assert!(paths(&mut document).is_empty(), "the image is ignored");
    assert!(square.painted.lock().unwrap().is_empty());
    let [_, _, width, _] = link_rect(&document);
    assert!((width - 48.0).abs() < 0.01, "\"Q\" then AB: {width}");
}

#[test]
fn a_generated_image_adds_nothing_to_the_accessible_name() {
    let mut document = document(16.0, &format!("content: url({FACE}); {FACE_RULE}"));
    document.set_vector_image(FACE, Some(Arc::new(Square::default())));
    let _ = paths(&mut document);
    let dom = document.dom();
    let link = dom.first_with_class(dom.document(), "link").unwrap();
    assert_eq!(document.generated_text(link), (String::new(), String::new()));
}
