// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An embedder's vector picture inline in text, sized by the `cap` unit: the
//! host route for faces beside a mention (iconography plan, T3).

use std::sync::{Arc, Mutex};

use genet_livery::{Device, LiveryDocument, StyleSet, VectorImage};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use paint_list_api::{
    ColorF, CommonPlacement, LayoutRect, PaintCmd, PaintList, PathCommand, PathData, PathItem,
};

const AHEM: &[u8] = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf");
/// Ahem's cap height and ascent are both 0.8 em.
const AHEM_CAP: f32 = 0.8;

/// A square that records every rect it was asked to fill.
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
        let corner = |x, y| PathCommand::LineTo(paint_list_api::LayoutPoint::new(x, y));
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

/// The face sizing rule as a host writes it for a device scale (pictograph's
/// `inline_face_style`, copied; livery takes no pictograph dependency): the
/// cap height, at least 8 device pixels, rounded to a device pixel.
fn face_style(scale: f32) -> String {
    let pixel = 1.0 / scale;
    let floor = 8.0 / scale;
    format!(
        "height: round(nearest, max({floor}px, 1cap), {pixel}px); width: auto; \
         vertical-align: baseline"
    )
}

struct Placed {
    face: [f32; 4],
    baseline: f32,
    painted: Vec<LayoutRect>,
    paths: usize,
}

fn place(font_size: f32, scale: f32) -> Placed {
    let html = "<html><body><p>XX <img class=face src=\"https://site.test/face/a\">\
                <span class=probe></span> XX</p></body></html>";
    let css = format!(
        "@font-face {{ font-family: ahem; src: url(https://site.test/ahem.ttf); }}\
         html, body, p {{ margin: 0; }}\
         p {{ font-family: ahem; font-size: {font_size}px; line-height: 2; }}\
         .face {{ {} }}\
         .probe {{ display: inline-block; width: 1px; height: 0; }}",
        face_style(scale)
    );
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[&css]),
        Device::screen(400.0, 200.0),
    );
    document.set_font_resource("https://site.test/ahem.ttf", AHEM.to_vec());
    let square = Arc::new(Square::default());
    document.set_vector_image(
        "https://site.test/face/a",
        Some(square.clone() as Arc<dyn VectorImage>),
    );
    let frame = document.frame(400, 200).expect("paint frame");
    let dom = document.dom();
    let face = dom.first_with_class(dom.document(), "face").unwrap();
    let probe = dom.first_with_class(dom.document(), "probe").unwrap();
    let paths = frame
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCmd::DrawPath(_)))
        .count();
    let [_, probe_y, _, _] = document.fragment_rect(probe).expect("probe placed");
    Placed {
        face: document.fragment_rect(face).expect("face placed"),
        baseline: probe_y,
        painted: square.painted.lock().unwrap().clone(),
        paths,
    }
}

fn expected_height(font_size: f32, scale: f32) -> f32 {
    let device = (font_size * AHEM_CAP * scale).round().max(8.0);
    device / scale
}

#[test]
fn a_vector_image_sits_on_the_baseline_at_cap_height() {
    for scale in [1.0, 2.0] {
        for font_size in [12.0, 14.0, 16.0] {
            let placed = place(font_size, scale);
            let [_, y, width, height] = placed.face;
            let want = expected_height(font_size, scale);
            let at = format!("{font_size}px at {scale}x");
            assert!(
                (height - want).abs() < 0.01,
                "{at}: height {height}, want {want}"
            );
            assert!(
                (width - height).abs() < 0.01,
                "{at}: square, {width} by {height}"
            );
            assert!(
                (y + height - placed.baseline).abs() < 0.01,
                "{at}: bottom {} on baseline {}",
                y + height,
                placed.baseline
            );
            assert!(
                ((height * scale) - (height * scale).round()).abs() < 0.01,
                "{at}: whole device pixels"
            );
            assert_eq!(placed.painted.len(), 1, "{at}: painted once");
            let rect = placed.painted[0];
            assert!(
                (rect.min.y - y).abs() < 0.01 && (rect.height() - height).abs() < 0.01,
                "{at}: painted into its own box"
            );
            assert_eq!(placed.paths, 1, "{at}: the image's own paths, no raster");
        }
    }
}

#[test]
fn cap_resolves_from_the_font_not_a_fixed_ratio() {
    // Ahem's cap height is 0.8em; the stateless fallback is 0.7em.
    let placed = place(20.0, 1.0);
    assert!((placed.face[3] - 16.0).abs() < 0.01, "{:?}", placed.face);
}

#[test]
fn removing_the_vector_image_leaves_a_broken_image_not_stale_paths() {
    let html = "<html><body><img src=\"https://site.test/face/a\" style=\"width: 10px; height: 10px\"></body></html>";
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[]),
        Device::screen(100.0, 100.0),
    );
    let square: Arc<dyn VectorImage> = Arc::new(Square::default());
    document.set_vector_image("https://site.test/face/a", Some(square));
    let paths = |frame: &genet_livery::LiveryPaintList| {
        frame
            .commands()
            .iter()
            .filter(|command| matches!(command, PaintCmd::DrawPath(_)))
            .count()
    };
    assert_eq!(paths(&document.frame(100, 100).unwrap()), 1);
    document.set_vector_image("https://site.test/face/a", None);
    assert_eq!(paths(&document.frame(100, 100).unwrap()), 0);
}
