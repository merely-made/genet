// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{
    Device, InteractionStates, LiveryDocument, StyleSet, emit_paint_list, layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use paint_list_api::PaintCmd;

fn render(html: &str, css: &str, width: f32) -> genet_livery::LiveryPaintList {
    let document = StaticDocument::parse(html);
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[css]),
        &Device::screen(width, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, width, 240.0).unwrap();
    emit_paint_list(
        &document,
        &styles,
        &fragments,
        paint_list_api::DeviceIntSize::new(width as i32, 240),
        1,
    )
}

fn render_retained(
    html: &str,
    css: &str,
    width: u32,
) -> (genet_livery::LiveryPaintList, genet_livery::LiveryPaintList) {
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[css]),
        Device::screen(width as f32, 240.0),
    );
    let first = document.frame(width, 240).unwrap();
    let cached = document.frame(width, 240).unwrap();
    (first, cached)
}

fn glyph_signature(list: &genet_livery::LiveryPaintList) -> Vec<String> {
    list.commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawText(run) => Some(run),
            _ => None,
        })
        .flat_map(|run| {
            run.glyphs.iter().map(move |glyph| {
                format!(
                    "font:{:?};size:{:?};color:{:?};glyph:{};point:{:?}",
                    run.font_instance, run.font_size, run.color, glyph.index, glyph.point
                )
            })
        })
        .collect()
}

fn command_signature(list: &genet_livery::LiveryPaintList) -> Vec<String> {
    list.commands()
        .iter()
        .map(|command| format!("{command:?}"))
        .collect()
}

#[test]
fn rtl_marker_only_nested_decimal_lists_match_literal_in_stateless_and_retained_frames() {
    let candidate = r#"<html dir="rtl"><body><ol><li><ol><li><ol><li><span>List item text.</span></li></ol></li></ol></li></ol></body></html>"#;
    let reference = r#"<html><body><div>1.<div>1.<div>1. <span>List item text.</span></div></div></div></body></html>"#;
    let candidate_css = "ol, li { margin: 0; padding: 0; border: 0; } \
               li, div { color: blue; border: 4px solid silver; \
                         padding: 8px 48px 8px 8px; list-style-position: inside; } \
               span { color: white; }";
    let reference_css = format!("body {{ direction: rtl; }} {candidate_css}");

    for width in [320_u32, 247] {
        let actual = render(candidate, candidate_css, width as f32);
        let expected = render(reference, &reference_css, width as f32);
        assert_eq!(glyph_signature(&actual), glyph_signature(&expected));

        let (actual, cached) = render_retained(candidate, candidate_css, width);
        let (expected, _) = render_retained(reference, &reference_css, width);
        assert_eq!(glyph_signature(&actual), glyph_signature(&expected));
        assert_eq!(command_signature(&cached), command_signature(&actual));
    }
}
