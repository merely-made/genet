// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{
    Device, InteractionStates, LiveryDocument, StyleSet, emit_paint_list, layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use paint_list_api::{PaintCmd, PaintList};

fn render(html: &str, css: &str) -> genet_livery::LiveryPaintList {
    let document = StaticDocument::parse(html);
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[css]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, 320.0, 240.0).unwrap();
    emit_paint_list(
        &document,
        &styles,
        &fragments,
        paint_list_api::DeviceIntSize::new(320, 240),
        1,
    )
}

fn render_retained(html: &str, css: &str) -> genet_livery::LiveryPaintList {
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[css]),
        Device::screen(320.0, 240.0),
    );
    document.frame(320, 240).unwrap()
}

fn glyph_signature(list: &genet_livery::LiveryPaintList) -> Vec<(u32, f32, f32)> {
    list.commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawText(run) => Some(run),
            _ => None,
        })
        .flat_map(|run| {
            run.glyphs
                .iter()
                .map(|glyph| (glyph.index, glyph.point.x, glyph.point.y))
        })
        .collect()
}

fn rightmost_glyph_x(list: &genet_livery::LiveryPaintList) -> f32 {
    glyph_signature(list)
        .into_iter()
        .map(|(_, x, _)| x)
        .reduce(f32::max)
        .expect("fixture paints text")
}

fn horizontal_glyph_signature(list: &genet_livery::LiveryPaintList) -> Vec<(u32, f32)> {
    glyph_signature(list)
        .into_iter()
        .map(|(glyph, x, _)| (glyph, x))
        .collect()
}

#[test]
fn padded_horizontal_rtl_text_uses_the_content_right_edge() {
    let padded = render(
        "<html><body><div>AB</div></body></html>",
        "html, body { margin: 0; } div { width: 200px; direction: rtl; \
         border: 4px solid black; padding: 8px 48px 8px 8px; }",
    );
    let positioned_content_box = render(
        "<html><body><div>AB</div></body></html>",
        "html, body { margin: 0; } div { width: 200px; margin-left: 12px; direction: rtl; }",
    );

    assert_eq!(
        horizontal_glyph_signature(&padded),
        horizontal_glyph_signature(&positioned_content_box)
    );
    let content_right = 4.0 + 8.0 + 200.0;
    let rightmost = rightmost_glyph_x(&padded);
    assert!(
        rightmost <= content_right,
        "{rightmost} exceeds {content_right}"
    );
    assert!(
        rightmost > content_right - 24.0,
        "{rightmost} is not right aligned"
    );
}

#[test]
fn nested_padded_horizontal_rtl_text_matches_retained_content_geometry() {
    let html = "<html><body><div class='box'><div class='box'><div class='box'>AB</div></div></div></body></html>";
    let css = "html, body { margin: 0; } .box { direction: rtl; \
               border: 4px solid black; padding: 8px 48px 8px 8px; }";
    let stateless = render(html, css);
    let retained = render_retained(html, css);

    assert_eq!(glyph_signature(&stateless), glyph_signature(&retained));
    let deepest_content_right = 320.0 - 3.0 * (4.0 + 48.0);
    let rightmost = rightmost_glyph_x(&stateless);
    assert!(
        rightmost <= deepest_content_right,
        "{rightmost} exceeds nested content edge {deepest_content_right}"
    );
    assert!(
        rightmost > deepest_content_right - 24.0,
        "{rightmost} is not aligned to nested content edge {deepest_content_right}"
    );
}
