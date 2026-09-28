// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::QualName;
use paint_list_api::{PaintCmd, PaintList};

const INITIAL_A: &str = "transform: translate(10px, 10px); z-index: 1";
const INITIAL_B: &str = "transform: translate(70px, 10px); z-index: 2";
const MOVED_A: &str = "transform: translate(50px, 30px) scale(1.25); z-index: 3";
const MOVED_B: &str = "transform: translate(50px, 30px); z-index: 1";
const SHEET: &str = "html, body { margin: 0; padding: 0; }
    #stage { position: relative; width: 180px; height: 100px; overflow: hidden; }
    .node { position: absolute; left: 0; top: 0; width: 36px; height: 36px;
        transform-origin: 0 0; background: red; }
    #b { background: blue; }
    .caption { position: absolute; left: 42px; top: 8px; white-space: nowrap; }
    #fixed { position: fixed; left: 1px; top: 1px; width: 2px; height: 2px; background: green; }";

fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

fn id(document: &LiveryDocument<ScriptedDom>, name: &str) -> NodeId {
    find_id(document.dom(), document.dom().document(), name).unwrap()
}

fn fixture(a: &str, b: &str, extra_sheet: &str) -> LiveryDocument<ScriptedDom> {
    let html = format!(
        "<html><body><div id=stage><div id=a class=node style=\"{a}\"><span id=caption class=caption>Alpha label</span><div id=fixed></div></div><div id=b class=node style=\"{b}\"><span class=caption>Beta label</span></div></div></body></html>"
    );
    let mut dom = ScriptedDom::from_serialized_document(&html);
    dom.drain_mutations(&mut Vec::new());
    LiveryDocument::new(
        dom,
        StyleSet::cambium(&[SHEET, extra_sheet]),
        Device::screen(240.0, 140.0),
    )
}

fn set_style(document: &mut LiveryDocument<ScriptedDom>, name: &str, value: &str) {
    let target = id(document, name);
    document.mutate_dom(|dom| dom.set_attribute(target, attr("style"), value));
}

fn paint(document: &mut LiveryDocument<ScriptedDom>) -> LiveryPaintList {
    document.frame(240, 140).unwrap()
}

fn assert_fresh(
    document: &mut LiveryDocument<ScriptedDom>,
    fresh: &mut LiveryDocument<ScriptedDom>,
) {
    assert_eq!(
        format!("{:?}", paint(document).commands()),
        format!("{:?}", paint(fresh).commands())
    );
}

#[test]
fn retained_motion_reuses_labelled_boxes_and_reorders_paint_and_input() {
    let mut document = fixture(INITIAL_A, INITIAL_B, "");
    let initial = paint(&mut document);
    assert!(
        initial
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawText(run) if !run.glyphs.is_empty()))
    );
    let layout_generation = document.layout_generation();
    let paint_generation = document.generation();
    let a = id(&document, "a");
    let caption = id(&document, "caption");
    let initial_box = document.fragment_rect(a);
    let initial_caption = document.fragment_rect(caption);
    // Separate mutation batches exercise cumulative eligibility.
    set_style(&mut document, "a", MOVED_A);
    set_style(&mut document, "b", MOVED_B);
    let moved = paint(&mut document);
    assert_eq!(document.layout_generation(), layout_generation);
    assert!(document.generation() > paint_generation);
    assert_eq!(document.fragment_rect(a), initial_box);
    assert_eq!(document.fragment_rect(caption), initial_caption);
    assert_ne!(
        format!("{:?}", initial.commands()),
        format!("{:?}", moved.commands())
    );
    assert_eq!(document.hit_test(60.0, 40.0), Some(a));
    let layout = document.layout.as_ref().unwrap();
    let geometry = crate::element_geometry(
        document.dom(),
        &layout.styles,
        &layout.fragments,
        &document.nested_scroll,
        a,
    )
    .unwrap();
    assert_eq!(geometry.map_to_local(60.0, 40.0), Some((8.0, 8.0)));
    let caption_geometry = crate::element_geometry(
        document.dom(),
        &layout.styles,
        &layout.fragments,
        &document.nested_scroll,
        caption,
    )
    .unwrap();
    assert_eq!(caption_geometry.map_to_local(102.5, 40.0), Some((0.0, 0.0)));
    assert_fresh(&mut document, &mut fixture(MOVED_A, MOVED_B, ""));
    // A second motion frame stays eligible; z-only changes update the winner.
    let behind = "transform: translate(50px, 30px) scale(1.25); z-index: -1";
    set_style(&mut document, "a", behind);
    paint(&mut document);
    assert_eq!(document.layout_generation(), layout_generation);
    assert_eq!(document.hit_test(60.0, 40.0), Some(id(&document, "b")));
    assert_fresh(&mut document, &mut fixture(behind, MOVED_B, ""));
}

#[test]
fn retained_motion_rejects_geometry_clip_generated_and_context_changes() {
    for suffix in [
        "; width: 60px",
        "; display: none",
        "; clip-path: polygon(0 0, 10px 0, 0 10px)",
        "; transform: none",
        "; transform: translateZ(20px)",
        "; z-index: auto",
        "; --caption: 'changed'",
    ] {
        let extra = "#a::before { content: var(--caption, 'before'); }";
        let mut document = fixture(INITIAL_A, INITIAL_B, extra);
        paint(&mut document);
        let generation = document.layout_generation();
        let changed = format!("{MOVED_A}{suffix}");
        set_style(&mut document, "a", &changed);
        paint(&mut document);
        assert!(
            document.layout_generation() > generation,
            "must reformat: {suffix}"
        );
        assert_fresh(&mut document, &mut fixture(&changed, INITIAL_B, extra));
    }
}

#[test]
fn retained_motion_does_not_hide_earlier_text_or_structure_batches() {
    for structure in [false, true] {
        let mut document = fixture(INITIAL_A, INITIAL_B, "");
        paint(&mut document);
        let generation = document.layout_generation();
        let caption = id(&document, "caption");
        let mutate = |dom: &mut ScriptedDom, caption| {
            if structure {
                dom.set_text_content(caption, "A replacement caption");
            } else {
                let text = dom.dom_children(caption).next().unwrap();
                dom.set_text(text, "A changed caption");
            }
        };
        document.mutate_dom(|dom| mutate(dom, caption));
        set_style(&mut document, "a", MOVED_A);
        paint(&mut document);
        assert!(document.layout_generation() > generation);
        let mut fresh = fixture(MOVED_A, INITIAL_B, "");
        let caption = id(&fresh, "caption");
        fresh.mutate_dom(|dom| mutate(dom, caption));
        assert_fresh(&mut document, &mut fresh);
    }
}

#[test]
fn retained_motion_rejects_resources_viewport_and_active_animation() {
    for case in ["resource", "viewport", "transition", "keyframe"] {
        let extra = if case == "keyframe" {
            "@keyframes fade { from { opacity: 0.2; } to { opacity: 0.8; } } #a { animation: fade 1000ms linear; }"
        } else {
            ""
        };
        let mut document = fixture(INITIAL_A, INITIAL_B, extra);
        paint(&mut document);
        let generation = document.layout_generation();
        if case == "resource" {
            document.set_image_resource("unused.png", vec![1, 2, 3]);
        }
        set_style(&mut document, "a", MOVED_A);
        if case == "transition" {
            assert!(document.animate_opacity(id(&document, "a"), 0.2, 0.8, 0.0, 1000.0));
        }
        if case == "viewport" {
            document.frame(260, 140).unwrap();
        } else {
            paint(&mut document);
        }
        assert!(
            document.layout_generation() > generation,
            "must reformat: {case}"
        );
    }
}

// This combines the retained-motion admission with ruling 379's font-content
// bounds. Explicit negative leading and wrapped edges must survive reuse.
#[cfg(target_os = "windows")]
#[test]
fn retained_motion_preserves_wrapped_edges_with_negative_leading() {
    fn wrapped(motion: &str, line_height: u32) -> LiveryDocument<ScriptedDom> {
        let html = format!(
            "<!DOCTYPE html><html><body><div id=a style=\"{motion}\"><span id=caption>alpha beta gamma delta epsilon zeta</span></div></body></html>"
        );
        let sheet = format!(
            "@font-face {{ font-family: receipt-arial; src: url('receipt-arial.ttf'); }}
             html, body {{ margin: 0; padding: 0; }}
             #a {{ position: absolute; left: 0; top: 0; width: 90px;
                   transform-origin: 0 0; font: 16px receipt-arial;
                   line-height: {line_height}px; }}
             #caption {{ border: 1px solid red; }}"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let mut document = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&sheet]),
            Device::screen(240.0, 140.0),
        );
        document.set_font_resource(
            "receipt-arial.ttf",
            std::fs::read("C:/Windows/Fonts/arial.ttf").expect("receipt Arial font"),
        );
        document
    }

    let mut document = wrapped(INITIAL_A, 8);
    let initial = paint(&mut document);
    let edges: Vec<_> = initial
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            PaintCmd::DrawBorder(item) => Some(item.placement.bounds),
            _ => None,
        })
        .collect();
    assert!(edges.len() >= 3, "must actually wrap: {edges:?}");
    assert!(
        initial
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawText(run) if !run.glyphs.is_empty()))
    );
    for edge in &edges {
        assert!(
            (edge.max.y - edge.min.y - 19.0).abs() < 0.01,
            "Arial 16px content plus two 1px borders: {edge:?}"
        );
    }
    let generation = document.layout_generation();
    set_style(&mut document, "a", MOVED_A);
    let moved = paint(&mut document);
    assert_eq!(document.layout_generation(), generation);
    assert_ne!(
        format!("{:?}", initial.commands()),
        format!("{:?}", moved.commands())
    );
    assert_fresh(&mut document, &mut wrapped(MOVED_A, 8));
    assert_eq!(document.layout_generation(), generation);

    set_style(&mut document, "a", &format!("{MOVED_A}; line-height: 24px"));
    paint(&mut document);
    assert!(document.layout_generation() > generation);
    assert_fresh(&mut document, &mut wrapped(MOVED_A, 24));
    println!(
        "WRAPPED-MOTION edges={} initial-line-height=8 moved-layout-reused=true changed-line-height=24 reformatted=true retained-fresh-equal=true",
        edges.len()
    );
}
