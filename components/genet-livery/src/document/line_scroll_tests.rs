// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::QualName;

const WORDS: &str = "Retained text with a second line of words.";

fn fixture(content: &str, height: &str, line: &str) -> LiveryDocument<ScriptedDom> {
    let html = format!("<!DOCTYPE html><html><body><div id=a>{content}</div></body></html>");
    let sheet = format!(
        "@font-face {{ font-family: receipt-arial; src: url(/arial.ttf); }}
         html, body {{ margin: 0; padding: 0; }}
         #a {{ font: 16px receipt-arial; width: 220px; height: {height};
               line-height: {line}; overflow: auto; color: black !important; }}
         #a:hover {{ color: red; }}"
    );
    let mut dom = ScriptedDom::from_serialized_document(&html);
    dom.drain_mutations(&mut Vec::new());
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&[&sheet]),
        Device::screen(300.0, 100.0),
    );
    let windows = std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SystemRoot"))
        .unwrap();
    document.set_font_resource(
        "/arial.ttf",
        std::fs::read(std::path::PathBuf::from(windows).join("Fonts/arial.ttf")).unwrap(),
    );
    document.frame(300, 100).unwrap();
    document
}

fn owner(document: &LiveryDocument<ScriptedDom>) -> NodeId {
    find_id(document.dom(), document.dom().document(), "a").unwrap()
}

fn range(document: &LiveryDocument<ScriptedDom>) -> f32 {
    document
        .scroll_extent(document.layout.as_ref().unwrap(), owner(document))
        .1
}

fn lines(document: &LiveryDocument<ScriptedDom>) -> buckram::PhysicalRect {
    document
        .retained_layout()
        .unwrap()
        .inline_scroll_bounds(owner(document))
        .unwrap()
}

fn style(document: &mut LiveryDocument<ScriptedDom>, value: &str) {
    let node = owner(document);
    let attr = QualName::new(None, Namespace::from(""), LocalName::from("style"));
    document.mutate_dom(|dom| dom.set_attribute(node, attr, value));
    document.frame(300, 100).unwrap();
}

#[test]
fn line_scroll_two_tall_lines_preserve_twelve_pixel_offset() {
    let mut document = fixture(WORDS, "180px", "200px");
    let node = owner(&document);
    let layout = document.retained_layout().unwrap();
    let text = document
        .dom()
        .dom_children(node)
        .find(|n| document.dom().kind(*n) == NodeKind::Text)
        .unwrap();
    let fragments = layout.fragments_for_node(text).collect::<Vec<_>>();
    assert_eq!(fragments.len(), 2);
    let font_bottom = fragments.iter().map(|f| f.y + f.height).fold(0.0, f32::max);
    assert_eq!(
        font_bottom - 180.0,
        128.0,
        "all font fragments alone remain insufficient"
    );
    assert_eq!(lines(&document).height, 400.0);
    assert_eq!(range(&document), 220.0);
    document.nested_scroll.insert(node, (0.0, 12.0));
    document.clamp_nested_scroll();
    assert_eq!(document.element_scroll()[&node], (0.0, 12.0));
    println!(
        "LINE_SCROLL two-lines=400 container=180 all-font-fragments-only=128 actual-range=220 retained-offset=12"
    );
}

#[test]
fn line_scroll_normal_single_and_empty_break_lines() {
    let normal = fixture(WORDS, "180px", "normal");
    assert_eq!(range(&normal), 0.0);
    let single = fixture("Short", "180px", "200px");
    assert_eq!(lines(&single).height, 200.0);
    assert_eq!(range(&single), 20.0);
    let breaks = fixture("<br><br>", "180px", "200px");
    assert_eq!(lines(&breaks).height, 400.0);
    assert_eq!(range(&breaks), 220.0);
    println!("LINE_SCROLL normal-range=0 single-range=20 empty-breaks-range=220");
}

#[test]
fn line_scroll_later_font_fragment_can_extend_beyond_lines() {
    let document = fixture(WORDS, "12px", "8px");
    let node = owner(&document);
    let layout = document.retained_layout().unwrap();
    let text = document
        .dom()
        .dom_children(node)
        .find(|n| document.dom().kind(*n) == NodeKind::Text)
        .unwrap();
    let fragments = layout.fragments_for_node(text).collect::<Vec<_>>();
    assert_eq!(fragments.len(), 2);
    let container = layout.get(node).unwrap();
    let last_end = fragments[1].y + fragments[1].height;
    let line_end = lines(&document).y + lines(&document).height;
    assert!(
        last_end > line_end,
        "negative leading leaves font content below the line"
    );
    assert!(last_end > fragments[0].y + fragments[0].height);
    assert_eq!(range(&document), last_end - container.y - container.height);
    println!(
        "LINE_SCROLL later-font-range={} line-only-range={}",
        range(&document),
        line_end - container.y - container.height
    );
}

#[test]
fn line_scroll_clipped_descendant_does_not_leak_its_lines() {
    let document = fixture(
        "<div style='height:20px; overflow:hidden'>Short</div>",
        "180px",
        "200px",
    );
    assert_eq!(range(&document), 0.0);
}

#[test]
fn line_scroll_partial_inline_groups_keep_formatting_owner() {
    let document = fixture(
        "before<span style='position:absolute'>out</span>after",
        "180px",
        "200px",
    );
    assert_eq!(
        lines(&document).height,
        400.0,
        "both partial groups contribute to their host despite lacking an intrinsic-cache owner"
    );
    assert_eq!(range(&document), 220.0);
    println!("LINE_SCROLL split-groups-line-union=400 range=220");
}

#[test]
fn line_scroll_retained_motion_and_reformat_keep_bounds_current() {
    let mut document = fixture(WORDS, "180px", "200px");
    style(
        &mut document,
        "position:absolute; z-index:1; left:20px; top:30px; transform:translate(1px, 2px)",
    );
    let before = lines(&document);
    assert_eq!(before.y, 30.0);
    let generation = document.layout_generation();
    style(
        &mut document,
        "position:absolute; z-index:1; left:20px; top:30px; transform:translate(50px, 60px) scale(1.25)",
    );
    assert_eq!(document.layout_generation(), generation);
    assert_eq!(
        lines(&document),
        before,
        "paint transforms do not change layout coordinates"
    );
    let mut fresh = fixture(WORDS, "180px", "200px");
    style(
        &mut fresh,
        "position:absolute; z-index:1; left:20px; top:30px; transform:translate(50px, 60px) scale(1.25)",
    );
    assert_eq!(lines(&document), lines(&fresh));
    assert_eq!(range(&document), 220.0);
    let node = owner(&document);
    document.nested_scroll.insert(node, (0.0, 12.0));
    style(
        &mut document,
        "position:absolute; z-index:1; left:20px; top:30px; line-height:normal",
    );
    assert!(document.layout_generation() > generation);
    assert!(lines(&document).height < before.height);
    assert_eq!(range(&document), 0.0);
    assert_eq!(document.element_scroll()[&node], (0.0, 0.0));
    style(
        &mut fresh,
        "position:absolute; z-index:1; left:20px; top:30px; line-height:normal",
    );
    assert_eq!(lines(&document), lines(&fresh));
    println!(
        "LINE_SCROLL positioned-origin=30 transform-layout-reused=true reformatted-normal-range=0 fresh-equal=true"
    );
}

#[test]
fn line_scroll_retained_frame_translation_and_replacement() {
    let mut document = fixture(WORDS, "180px", "200px");
    let node = owner(&document);
    let mut frame = document
        .retained_layout()
        .unwrap()
        .text_frame()
        .unwrap()
        .clone();
    let before = frame.line_bounds(node).unwrap();
    frame.translate_subtree(document.dom(), node, (11.0, 23.0));
    let moved = frame.line_bounds(node).unwrap();
    assert_eq!(
        (moved.x, moved.y, moved.height),
        (before.x + 11.0, before.y + 23.0, before.height)
    );
    style(&mut document, "line-height:normal");
    let fresh_frame = document.retained_layout().unwrap().text_frame().unwrap();
    let replaced = HashSet::from([node]);
    frame.replace_subtree_from(fresh_frame, &replaced, &[]);
    assert_eq!(frame.line_bounds(node), fresh_frame.line_bounds(node));
}
