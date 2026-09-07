// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use genet_livery::{
    Device, InteractionStates, LiveryDocument, StyleSet, emit_paint_list, layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use livery::values::ListStyleType;
use paint_list_api::PaintCmd;

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

fn render_retained(
    html: &str,
    css: &str,
) -> (genet_livery::LiveryPaintList, genet_livery::LiveryPaintList) {
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[css]),
        Device::screen(320.0, 240.0),
    );
    let first = document.frame(320, 240).unwrap();
    let cached = document.frame(320, 240).unwrap();
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
fn quoted_list_style_type_decodes_and_serializes_css_strings() {
    assert_eq!(
        ListStyleType::from_str(r#""\23  ""#).unwrap(),
        ListStyleType::String("# ".to_owned())
    );
    assert_eq!(
        ListStyleType::from_str(r#""""#).unwrap(),
        ListStyleType::String(String::new())
    );
    assert_eq!(
        ListStyleType::String("#  ".to_owned()).to_string(),
        "\"#  \""
    );
    assert_eq!(
        livery::canonicalize_specified_longhand("list-style-type", r#""\23  ""#),
        Some("\"# \"".to_owned())
    );
}

#[test]
fn inside_string_markers_match_literal_glyphs_with_preserved_spaces() {
    let candidate = render(
        "<html><body><ol><li>item</li></ol><ul><li>item</li></ul></body></html>",
        r##"* { margin: 0; padding: 0; }
             ol, ul { list-style-position: inside; list-style-type: "#  "; }"##,
    );
    let reference = render(
        "<html><body><div><span>#  </span>item</div><div><span>#  </span>item</div></body></html>",
        "* { margin: 0; padding: 0; } span { white-space-collapse: preserve; }",
    );

    assert_eq!(glyph_signature(&candidate), glyph_signature(&reference));

    let (retained, cached) = render_retained(
        "<html><body><ol><li>item</li></ol><ul><li>item</li></ul></body></html>",
        r##"* { margin: 0; padding: 0; }
             ol, ul { list-style-position: inside; list-style-type: "#  "; }"##,
    );
    assert_eq!(glyph_signature(&retained), glyph_signature(&reference));
    assert_eq!(command_signature(&cached), command_signature(&retained));
}

#[test]
fn escaped_and_empty_string_markers_are_stateless_and_retained() {
    let escaped_html = "<html><body><ol><li>ITEM</li></ol></body></html>";
    let escaped_css = r#"* { margin: 0; padding: 0; text-transform: lowercase; }
                         ol { list-style-position: inside; list-style-type: "\4d ARK "; }"#;
    let escaped_reference = render(
        "<html><body><div><span>MARK </span>ITEM</div></body></html>",
        "* { margin: 0; padding: 0; text-transform: lowercase; } span { text-transform: none; }",
    );
    assert_eq!(
        glyph_signature(&render(escaped_html, escaped_css)),
        glyph_signature(&escaped_reference)
    );

    let empty_html = "<html><body><ol><li>item</li></ol></body></html>";
    let empty_css = r#"* { margin: 0; padding: 0; }
                       ol { list-style-position: inside; list-style-type: ""; }"#;
    let empty_reference = render(
        "<html><body><div>item</div></body></html>",
        "* { margin: 0; padding: 0; }",
    );
    assert_eq!(
        glyph_signature(&render(empty_html, empty_css)),
        glyph_signature(&empty_reference)
    );

    let (retained, cached) = render_retained(empty_html, empty_css);
    assert_eq!(
        glyph_signature(&retained),
        glyph_signature(&empty_reference)
    );
    assert_eq!(command_signature(&cached), command_signature(&retained));
}
