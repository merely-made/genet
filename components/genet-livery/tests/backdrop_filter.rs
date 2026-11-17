// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use genet_livery::{
    Device, InteractionStates, LiveryPaintList, StyleSet, TextSystem, emit_paint_list,
    emit_paint_list_with_text_system_scrolled_with_images_and_external_textures, layout,
    resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use paint_list_api::{ColorF, DeviceIntSize, PaintCmd, PaintList};

fn render(html: &str, css: &str) -> LiveryPaintList {
    let document = StaticDocument::parse(html);
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[css]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, 320.0, 240.0).expect("layout");
    emit_paint_list(
        &document,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
    )
}

fn assert_blue_content(list: &LiveryPaintList) {
    assert!(
        list.commands().iter().any(|cmd| matches!(cmd,
            PaintCmd::DrawRect(rect) if rect.color == ColorF::new(0.0, 0.0, 1.0, 1.0)
        )),
        "foreground remains paintable: {:#?}",
        list.commands()
    );
}

fn assert_no_backdrop(list: &LiveryPaintList) {
    assert!(
        !list
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawBackdropBlur(_))),
        "unsupported context must omit the backdrop atom: {:#?}",
        list.commands()
    );
    assert_blue_content(list);
}

#[test]
fn backdrop_blur_is_one_atomic_draw_before_pane_foreground() {
    let list = render(
        "<html><body><div id=under></div><div id=pane></div></body></html>",
        "html, body { margin: 0 } #under { position: absolute; left: 4px; top: 4px; width: 100px; height: 80px; background: red } #pane { position: absolute; left: 20px; top: 30px; width: 40px; height: 24px; backdrop-filter: blur(7px); background: blue }",
    );
    let commands = list.commands();
    let atoms: Vec<_> = commands
        .iter()
        .enumerate()
        .filter_map(|(i, cmd)| match cmd {
            PaintCmd::DrawBackdropBlur(spec) => Some((i, spec)),
            _ => None,
        })
        .collect();
    assert_eq!(atoms.len(), 1, "{commands:#?}");
    let (at, spec) = atoms[0];
    assert_eq!(spec.radius_px, 14.0);
    assert_eq!(
        (
            spec.bounds.min.x,
            spec.bounds.min.y,
            spec.bounds.max.x,
            spec.bounds.max.y
        ),
        (20.0, 30.0, 60.0, 54.0)
    );
    let red = commands.iter().position(|cmd| matches!(cmd, PaintCmd::DrawRect(rect) if rect.color == ColorF::new(1.0, 0.0, 0.0, 1.0))).expect("preceding background");
    let blue = commands.iter().position(|cmd| matches!(cmd, PaintCmd::DrawRect(rect) if rect.color == ColorF::new(0.0, 0.0, 1.0, 1.0))).expect("pane foreground");
    assert!(red < at && at < blue, "{commands:#?}");
}

#[test]
fn unsupported_backdrop_contexts_keep_foreground_paintable() {
    for (outer, pane) in [
        ("opacity: .5", "left: 20px; top: 20px"),
        ("transform: translate(1px, 0px)", "left: 20px; top: 20px"),
        (
            "overflow: hidden; width: 100px; height: 100px",
            "left: 20px; top: 20px",
        ),
        ("", "left: 20.5px; top: 20px"),
        ("", "left: 20px; top: 20px; border-radius: 4px"),
    ] {
        let html = "<html><body><div id=outer><div id=pane></div></div></body></html>";
        let css = format!(
            "html, body {{ margin: 0 }} #outer {{ {outer} }} #pane {{ position: absolute; width: 40px; height: 24px; backdrop-filter: blur(7px); background: blue; {pane} }}"
        );
        let list = render(html, &css);
        assert_no_backdrop(&list);
    }
}

#[test]
fn earlier_element_filter_does_not_feed_an_unfiltered_prefix_to_backdrop() {
    let list = render(
        "<div id=under></div><div id=pane></div>",
        "html, body { margin: 0 } #under { position: absolute; width: 100px; height: 80px; background: red; filter: invert(1) } #pane { position: absolute; left: 20px; top: 30px; width: 40px; height: 24px; backdrop-filter: blur(7px); background: blue }",
    );
    assert!(list.commands().iter().any(|cmd| matches!(cmd,
        PaintCmd::PushLayer(layer) if !layer.filters.is_empty()
    )));
    assert_no_backdrop(&list);
}

#[test]
fn viewport_zoom_removes_backdrop_atom_but_keeps_pane() {
    let list = render(
        "<html><body><div id=pane></div></body></html>",
        "html, body { margin: 0 } #pane { position: absolute; left: 20px; top: 30px; width: 40px; height: 24px; backdrop-filter: blur(7px); background: blue }",
    );
    assert!(
        list.commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawBackdropBlur(_)))
    );
    let unchanged = list.clone().scaled_to(1.0, 320, 240);
    assert!(
        unchanged
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawBackdropBlur(_)))
    );
    assert_no_backdrop(&list.scaled_to(1.25, 400, 300));
}

#[test]
fn child_frame_splice_omits_backdrop_but_keeps_child_paint() {
    let child = render(
        "<html><body><div id=pane></div></body></html>",
        "html, body { margin: 0 } #pane { position: absolute; left: 2px; top: 2px; width: 30px; height: 20px; backdrop-filter: blur(7px); background: blue }",
    );
    assert!(
        child
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::DrawBackdropBlur(_)))
    );
    let mut parent = render(
        "<html><body><iframe></iframe></body></html>",
        "html, body { margin: 0 } iframe { display: block; width: 100px; height: 80px }",
    );
    assert_eq!(parent.frame_slots().len(), 1);
    let commands = parent.absorb_frame_commands(&child);
    parent.splice_frame_slots(|_| Some(commands.clone()));
    assert_no_backdrop(&parent);
}

#[test]
fn scrolled_ancestor_omits_nested_backdrop_and_keeps_content() {
    let document = StaticDocument::parse(
        "<html><body><div id=scroller><div id=pane></div></div></body></html>",
    );
    fn find<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<D::NodeId>
    where
        D::NodeId: Copy,
    {
        if dom
            .attribute(node, &Namespace::default(), &LocalName::from("id"))
            .is_some_and(|id| id == "scroller")
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child))
    }
    let scroller = find(&document, document.document()).expect("scroller");
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[
            "html, body { margin: 0 } #scroller { width: 100px; height: 100px; overflow: scroll } #pane { position: relative; width: 40px; height: 24px; backdrop-filter: blur(7px); background: blue }",
        ]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, 320.0, 240.0).expect("layout");
    let mut text = TextSystem::new();
    let list = emit_paint_list_with_text_system_scrolled_with_images_and_external_textures(
        &document,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut text,
        &HashMap::from([(scroller, (0.0, 5.0))]),
        &HashMap::new(),
        &HashMap::new(),
    );
    assert_no_backdrop(&list);
}
