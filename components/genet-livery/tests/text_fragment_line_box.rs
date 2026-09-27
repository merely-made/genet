// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A text run's painted fragment against its line box. Chromium paints text
//! over its content area, the font's rounded ascent plus rounded descent;
//! under `line-height: normal` the line box adds the rounded line gap to
//! those, so text stays inside it (design_docs/2026-09-25_line_box_model_plan.md,
//! 2026-09-27). The Arial rows are Chromium 153's on Windows at a device pixel
//! ratio of 1, so they run on Windows; Lato, which the tree carries, holds the
//! invariant on every platform.

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

type Node = <StaticDocument as LayoutDom>::NodeId;

fn find(dom: &StaticDocument, node: Node, needle: &str) -> Option<Node> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(needle)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, needle))
}

/// A standards-mode document of `body`, its face loaded from `bytes` at `url`.
fn lay_out(body: &str, css: &str, (url, bytes): (&str, Vec<u8>)) -> LiveryDocument<StaticDocument> {
    let html = format!("<!DOCTYPE html><html><body>{body}</body></html>");
    let mut session = LiveryDocument::new(
        StaticDocument::parse(&html),
        StyleSet::cambium(&[css]),
        Device::screen(320.0, 2400.0),
    );
    session.set_font_resource(url, bytes);
    session.frame(320, 2400).expect("frame");
    session
}

/// `#id`'s border box, `[x, y, width, height]`.
fn rect(session: &LiveryDocument<StaticDocument>, id: &str) -> [f32; 4] {
    let dom = session.dom();
    let node = find(dom, dom.document(), id).unwrap_or_else(|| panic!("#{id} is in the document"));
    session
        .fragment_rect(node)
        .unwrap_or_else(|| panic!("#{id} has a fragment"))
}

/// The line box of one-line block `#block`, which is the block's height, and
/// the painted top (from the block's) and height of the text `#holder` holds.
fn text_in_line(
    session: &LiveryDocument<StaticDocument>,
    block: &str,
    holder: &str,
) -> (f32, f32, f32) {
    let dom = session.dom();
    let owner =
        find(dom, dom.document(), holder).unwrap_or_else(|| panic!("#{holder} is in the document"));
    let text = dom
        .dom_children(owner)
        .find(|child| dom.kind(*child) == NodeKind::Text)
        .unwrap_or_else(|| panic!("#{holder} holds text"));
    let [_, top, _, line] = rect(session, block);
    let [_, text_top, _, height] = session
        .fragment_rect(text)
        .unwrap_or_else(|| panic!("#{holder}'s text has a fragment"));
    (line, text_top - top, height)
}

/// Block `#id` in `face` at `size`px and `line-height: normal`.
fn normal_block(id: &str, face: &str, size: u32, inner: &str) -> String {
    format!("<div id={id} style='font: {size}px {face}'>{inner}</div>")
}

/// No text fragment overhangs its line box under `line-height: normal`. At
/// fourteen of these sizes Parley's own extent, whose leading carries the
/// rounding of Lato's ascent and descent, was a pixel taller than the line
/// box, which rounds the line gap alone. The blocks stack in one document,
/// which is safe because every `normal` line box is a whole number of pixels.
#[test]
fn normal_text_fragments_stay_inside_their_line_boxes() {
    let body = (8..=40)
        .map(|size| normal_block(&format!("s{size}"), "receipt-lato", size, "Side panel text"))
        .collect::<String>();
    let session = lay_out(
        &body,
        "@font-face { font-family: receipt-lato; src: url(/fonts/Lato-Medium.ttf); } \
         body { margin: 0; } div { width: 300px; }",
        (
            "/fonts/Lato-Medium.ttf",
            include_bytes!("../../../tests/wpt/tests/fonts/Lato-Medium.ttf").to_vec(),
        ),
    );
    let overhanging = (8..=40)
        .filter_map(|size| {
            let id = format!("s{size}");
            let (line, top, text) = text_in_line(&session, &id, &id);
            (top < -0.01 || top + text > line + 0.01)
                .then(|| format!("{size}px Lato: text {text} at {top} in a {line} line box"))
        })
        .collect::<Vec<_>>();
    assert!(overhanging.is_empty(), "{}", overhanging.join("\n"));
}

#[cfg(windows)]
mod arial {
    use super::*;

    /// Arial as an authored face, so no system fallback stands in for it.
    const FACE: &str = "@font-face { font-family: receipt-arial; src: url(/fonts/arial.ttf); }";

    fn arial() -> Vec<u8> {
        let windows = std::env::var_os("WINDIR")
            .or_else(|| std::env::var_os("SystemRoot"))
            .expect("Windows directory is available through WINDIR or SystemRoot");
        let path = std::path::PathBuf::from(windows)
            .join("Fonts")
            .join("arial.ttf");
        std::fs::read(path).expect("Windows carries Arial")
    }

    /// `body` laid out alone in Arial, as `tests/line_box_model.rs` lays out
    /// each case: genet's block edges round, so a case stacked below a
    /// fractional one would inherit its offset.
    fn alone(body: &str, css: &str, face: &[u8]) -> LiveryDocument<StaticDocument> {
        lay_out(
            body,
            &format!("{FACE} {css}"),
            ("/fonts/arial.ttf", face.to_vec()),
        )
    }

    /// A one-line block of Arial at `line-height: normal`: its id, font size
    /// and content, Chromium 153's line box height, then for each text node,
    /// by the id of the element holding it, Chromium's text fragment height
    /// and top.
    type Case = (
        &'static str,
        u32,
        &'static str,
        f32,
        &'static [(&'static str, f32, f32)],
    );

    const NORMAL: [Case; 8] = [
        ("s11", 11, "Side panel text", 12.0, &[("s11", 12.0, 0.0)]),
        ("s12", 12, "Side panel text", 14.0, &[("s12", 14.0, 0.0)]),
        ("s13", 13, "Side panel text", 15.0, &[("s13", 15.0, 0.0)]),
        ("s14", 14, "Side panel text", 16.0, &[("s14", 16.0, 0.0)]),
        ("s16", 16, "Side panel text", 18.0, &[("s16", 17.0, 0.0)]),
        ("s18", 18, "Side panel text", 21.0, &[("s18", 20.0, 0.0)]),
        // On a line a taller box sets, text keeps its own content area.
        (
            "big",
            16,
            "small<span id=big-span style='font-size: 32px'>big</span>",
            37.0,
            &[("big", 17.0, 15.0), ("big-span", 36.0, 0.0)],
        ),
        (
            "super",
            13,
            "base<span id=super-span style='vertical-align: super'>sup</span>",
            20.33,
            &[("super", 15.0, 5.33), ("super-span", 15.0, 0.0)],
        ),
    ];

    /// Each text fragment beside its line box and Chromium's, within 0.5px:
    /// genet's block heights are whole pixels, and the `super` line is 20.33.
    #[test]
    fn normal_arial_text_fragments_match_chromium() {
        let face = arial();
        let mut failures = Vec::new();
        for (id, size, inner, line_chromium, texts) in NORMAL {
            let session = alone(
                &normal_block(id, "receipt-arial", size, inner),
                "body { margin: 0; } div { width: 300px; }",
                &face,
            );
            for &(holder, text_chromium, top_chromium) in texts {
                let (line, top, text) = text_in_line(&session, id, holder);
                let over = top < -0.5 || top + text > line + 0.5;
                eprintln!(
                    "TEXT-FRAGMENT {size:>2}px Arial #{holder:<10} \
                     line box {line:>5.2} (Chromium {line_chromium:>5.2})  \
                     text {text:>5.2} at {top:>4.2} (Chromium {text_chromium:>5.2} at {top_chromium:>4.2})  {}",
                    if over { "OVERHANGS" } else { "inside" },
                );
                if over
                    || (line - line_chromium).abs() > 0.5
                    || (text - text_chromium).abs() > 0.5
                    || (top - top_chromium).abs() > 0.5
                {
                    failures.push(format!(
                        "#{holder}: line box {line}, text {text} at {top}; \
                         Chromium {line_chromium}, {text_chromium} at {top_chromium}"
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// The plan's ground-truth table in Arial 16px/20px, a 300px block a case:
    /// markup, then Chromium's block height and `#<case>-atom`'s offset.
    const GROUND_TRUTH: [(&str, &str, f32, Option<f32>); 13] = [
        (
            "knot",
            "first words that wrap onto a second line here <span class=ib id=knot-atom></span> \
             after the atom and more words to wrap",
            85.0,
            Some(20.0),
        ),
        (
            "atom-one-line",
            "text <span class=ib id=atom-one-line-atom></span> text",
            45.0,
            Some(0.0),
        ),
        (
            "super",
            "text <span style='vertical-align: super'>sup</span> text",
            26.33,
            None,
        ),
        (
            "sub",
            "text <span style='vertical-align: sub'>sub</span> text",
            24.19,
            None,
        ),
        (
            "length",
            "text <span style='vertical-align: 10px'>up</span> text",
            30.0,
            None,
        ),
        (
            "big-font",
            "text <span style='font-size: 32px'>big</span> text",
            26.0,
            None,
        ),
        (
            "empty-tall",
            "text<span style='line-height: 40px'></span>",
            40.0,
            None,
        ),
        (
            "nested-tall",
            "text <span style='line-height: 40px'><span style='line-height: 10px'>in</span></span>",
            40.0,
            None,
        ),
        (
            "atom-top",
            "text <span class=ib id=atom-top-atom style='vertical-align: top'></span>",
            40.0,
            Some(0.0),
        ),
        (
            "atom-bottom",
            "text <span class=ib id=atom-bottom-atom style='vertical-align: bottom'></span>",
            40.0,
            Some(0.0),
        ),
        (
            "atom-middle",
            "text <span class=ib id=atom-middle-atom style='vertical-align: middle'></span>",
            40.0,
            Some(0.0),
        ),
        (
            "atom-text-top",
            "text <span class=ib id=atom-text-top-atom style='vertical-align: text-top'></span>",
            41.0,
            Some(1.0),
        ),
        (
            "atom-text-bottom",
            "text <span class=ib id=atom-text-bottom-atom style='vertical-align: text-bottom'></span>",
            42.0,
            Some(0.0),
        ),
    ];

    /// Every row of the plan's Arial table still matches Chromium, within
    /// 0.5px (genet's block heights are whole pixels).
    #[test]
    fn arial_ground_truth_rows_match_chromium() {
        let face = arial();
        let failures = GROUND_TRUTH
            .iter()
            .filter_map(|(id, inner, height, atom)| {
                let session = alone(
                    &format!("<div class=case id={id}>{inner}</div>"),
                    "body { margin: 0; font: 16px/20px receipt-arial; } .case { width: 300px; } \
                     .ib { display: inline-block; width: 80px; height: 40px; }",
                    &face,
                );
                let [_, top, _, got] = rect(&session, id);
                let got_atom = atom.map(|_| rect(&session, &format!("{id}-atom"))[1] - top);
                eprintln!(
                    "GROUND-TRUTH {id:<17} height {got:>5.2} (Chromium {height:>5.2})  \
                     atom {got_atom:?} (Chromium {atom:?})"
                );
                let atom_matches = match (atom, got_atom) {
                    (Some(atom), Some(got_atom)) => (atom - got_atom).abs() <= 0.5,
                    _ => true,
                };
                ((got - height).abs() > 0.5 || !atom_matches).then(|| {
                    format!("{id}: height {got}, atom {got_atom:?}; Chromium {height}, {atom:?}")
                })
            })
            .collect::<Vec<_>>();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Content height is a font metric, even with positive or negative leading.
    /// These are current Genet measurements, not a fresh browser comparison.
    #[test]
    fn explicit_height_keeps_font_content_bounds() {
        let face = arial();
        let mut failures = Vec::new();
        for (size, height, expected_line, expected_top, content) in [
            (16, "20px", 20.0, 1.0, 17.0),
            (16, "8px", 8.0, -5.0, 17.0),
            (16, "0", 0.0, -9.0, 17.0),
            (13, "1", 13.0, -1.0, 15.0),
        ] {
            let body = format!(
                "<div id=b style='font:{size}px receipt-arial;line-height:{height}'><span id=t>Text</span></div>"
            );
            let session = alone(&body, "body { margin:0 }", &face);
            let (line, top, actual) = text_in_line(&session, "b", "t");
            println!(
                "EXPLICIT size={size} line-height={height} line={line} content-top={top} content-height={actual}"
            );
            if (actual - content).abs() > 0.01 {
                failures.push(format!(
                    "{size}/{height}: content {actual}, expected {content}"
                ));
            }
            if (line - expected_line).abs() > 0.01 || (top - expected_top).abs() > 0.01 {
                failures.push(format!("{size}/{height}: line {line}, top {top}; expected {expected_line}, {expected_top}"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Inspect every emitted border/background rectangle, including the first
    /// and last wrapped edge. The font content area is 17px at Arial 16px.
    #[test]
    fn wrapped_decorations_use_font_content_bounds() {
        use paint_list_api::{ColorF, PaintCmd, PaintList};
        let face = arial();
        let mut failures = Vec::new();
        for line_height in ["normal", "24px", "8px"] {
            for (decoration, extra) in [
                ("border:1px solid red", 2.0),
                ("padding:2px 4px;background:red", 4.0),
            ] {
                let body = format!(
                    "<div id=b style='width:90px;font:16px receipt-arial;line-height:{line_height}'><span id=t style='{decoration}'>alpha beta gamma delta epsilon zeta</span></div>"
                );
                let mut session = alone(&body, "body { margin:0 }", &face);
                let list = session.frame(320, 2400).unwrap();
                let boxes: Vec<_> = list
                    .commands()
                    .iter()
                    .filter_map(|cmd| match cmd {
                        PaintCmd::DrawBorder(item) => Some(item.placement.bounds),
                        PaintCmd::DrawRect(item)
                            if item.color == ColorF::new(1.0, 0.0, 0.0, 1.0) =>
                        {
                            Some(item.placement.bounds)
                        },
                        _ => None,
                    })
                    .collect();
                println!(
                    "WRAPPED line-height={line_height} decoration={decoration} boxes={boxes:?}"
                );
                assert!(
                    boxes.len() >= 3,
                    "fixture must wrap across at least three painted fragments"
                );
                for (i, bounds) in boxes.iter().enumerate() {
                    let actual = bounds.max.y - bounds.min.y;
                    if (actual - (17.0 + extra)).abs() > 0.01 {
                        failures.push(format!(
                            "{line_height}/{decoration} fragment {i}: height {actual}, expected {}",
                            17.0 + extra
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn decorated_edges_follow_their_own_aligned_baseline() {
        use paint_list_api::{PaintCmd, PaintList};
        let face = arial();
        for align in ["baseline", "super", "sub", "top", "bottom"] {
            for empty in [false, true] {
                let content = if empty { "" } else { "Text" };
                let body = format!(
                    "<div id=b style='font:32px receipt-arial;line-height:40px'>Big<span id=t style='font-size:16px;line-height:20px;vertical-align:{align};border:1px solid red'>{content}</span><span id=reference style='font-size:16px;line-height:20px;vertical-align:{align}'>Text</span></div>"
                );
                let mut session = alone(&body, "body { margin:0 }", &face);
                let (_, text_top, text_height) = text_in_line(&session, "b", "reference");
                let block_top = rect(&session, "b")[1];
                let list = session.frame(320, 2400).unwrap();
                let borders: Vec<_> = list
                    .commands()
                    .iter()
                    .filter_map(|cmd| match cmd {
                        PaintCmd::DrawBorder(item) => Some(item.placement.bounds),
                        _ => None,
                    })
                    .collect();
                assert_eq!(borders.len(), 1);
                let border = borders[0];
                let top = border.min.y - block_top;
                let height = border.max.y - border.min.y;
                println!(
                    "ALIGNED align={align} empty={empty} border-top={top} border-height={height} reference-top={text_top} reference-height={text_height}"
                );
                assert!(
                    (height - 19.0).abs() < 0.01,
                    "edge must use its own 16px font, not the 32px parent's"
                );
                assert!(
                    (top - (text_top - 1.0)).abs() < 0.01,
                    "edge must use its own aligned baseline"
                );
            }
        }
    }
}
