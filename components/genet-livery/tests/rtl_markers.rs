// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{
    Device, InteractionStates, LiveryDocument, StyleSet, TextRange, emit_paint_list, layout,
    resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use paint_list_api::{ColorF, PaintCmd, PaintList};

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
) -> (
    LiveryDocument<StaticDocument>,
    genet_livery::LiveryPaintList,
    genet_livery::LiveryPaintList,
) {
    let mut document = LiveryDocument::new(
        StaticDocument::parse(html),
        StyleSet::cambium(&[css]),
        Device::screen(width as f32, 240.0),
    );
    let first = document.frame(width, 240).unwrap();
    let cached = document.frame(width, 240).unwrap();
    (document, first, cached)
}

fn find(
    dom: &StaticDocument,
    node: <StaticDocument as LayoutDom>::NodeId,
    id: &str,
) -> Option<<StaticDocument as LayoutDom>::NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, id))
}

fn text_runs(list: &genet_livery::LiveryPaintList, color: ColorF) -> Vec<Vec<(f32, f32)>> {
    list.commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawText(run) if run.color == color => Some(
                run.glyphs
                    .iter()
                    .map(|glyph| (glyph.point.x, glyph.point.y))
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

fn blue_marker_rows(list: &genet_livery::LiveryPaintList) -> Vec<Vec<f32>> {
    let mut glyphs = text_runs(list, ColorF::new(0.0, 0.0, 1.0, 1.0))
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    glyphs.sort_by(|left, right| left.1.total_cmp(&right.1));

    let mut rows: Vec<(f32, Vec<f32>)> = Vec::new();
    for (x, y) in glyphs {
        if let Some((baseline, row)) = rows.last_mut()
            && (y - *baseline).abs() < 0.01
        {
            row.push(x);
        } else {
            rows.push((y, vec![x]));
        }
    }
    rows.into_iter().map(|(_, row)| row).collect()
}

fn command_signature(list: &genet_livery::LiveryPaintList) -> Vec<String> {
    list.commands()
        .iter()
        .map(|command| format!("{command:?}"))
        .collect()
}

#[test]
fn rtl_marker_only_nested_decimal_lists_keep_markers_isolated_at_content_edges() {
    let candidate = r#"<html dir="rtl"><body><ol><li id="outer"><ol><li id="middle"><ol><li id="inner"><span id="text">List item text.</span></li></ol></li></ol></li></ol></body></html>"#;
    let marker_shape =
        r#"<html><body><div><span class="marker">1. </span><i>X</i></div></body></html>"#;
    let css = "html, body, ol, li { margin: 0; padding: 0; border: 0; } \
               li, div { color: blue; border: 4px solid silver; \
                         padding: 8px 48px 8px 8px; list-style-position: inside; } \
               span { color: white; } .marker { color: blue; } \
               i { color: white; font-style: normal; }";

    for width in [320_u32, 247] {
        let shape = text_runs(
            &render(marker_shape, css, width as f32),
            ColorF::new(0.0, 0.0, 1.0, 1.0),
        );
        let marker = shape
            .iter()
            .find(|run| run.len() >= 3)
            .expect("marker shape");
        let white_shape = text_runs(
            &render(marker_shape, css, width as f32),
            ColorF::new(1.0, 1.0, 1.0, 1.0),
        );
        let after = white_shape
            .iter()
            .find(|run| run.len() == 1)
            .expect("marker boundary");
        let marker_advance = after[0].0 - marker.iter().map(|(x, _)| *x).reduce(f32::min).unwrap();

        let stateless = render(candidate, css, width as f32);
        let (document, retained, cached) = render_retained(candidate, css, width);
        let stateless_rows = blue_marker_rows(&stateless);
        let retained_rows = blue_marker_rows(&retained);
        assert_eq!(stateless_rows, retained_rows);
        assert_eq!(
            retained_rows.len(),
            3,
            "all generated decimal markers paint"
        );
        assert!(
            retained_rows.iter().all(|row| row.len() >= 2),
            "every marker row paints the decimal glyphs"
        );
        assert_eq!(command_signature(&cached), command_signature(&retained));

        for (id, marker_row) in ["outer", "middle", "inner"].into_iter().zip(retained_rows) {
            let node = find(document.dom(), document.dom().document(), id).expect("list item");
            let [x, _, width, _] = document.fragment_rect(node).expect("list item fragment");
            let content_right = x + width - 4.0 - 48.0;
            assert!(
                (marker_row.into_iter().reduce(f32::min).unwrap() + marker_advance - content_right)
                    .abs()
                    < 0.01,
                "{id} marker ends at its own content edge"
            );
        }

        let text = find(document.dom(), document.dom().document(), "text").expect("visible span");
        let source = document
            .dom()
            .dom_children(text)
            .find(|node| document.dom().kind(*node) == NodeKind::Text)
            .expect("visible text source");
        let selected = document
            .selection_for_range(TextRange {
                anchor_node: source,
                anchor_offset: 0,
                focus_node: source,
                focus_offset: "List item text.".len(),
            })
            .expect("visible source range");
        assert_eq!(selected.text, "List item text.");
    }
}
