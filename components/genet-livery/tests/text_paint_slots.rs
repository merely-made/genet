// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host text paint must stay inside the field's CSS paint context.

use std::collections::HashMap;

use genet_livery::{
    InteractionStates, StyleSet, TextPaintPhase, TextSystem,
    emit_paint_list_with_text_system_scrolled_with_images, layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::media::Device;
use paint_list_api::{
    ColorF, CommonPlacement, DeviceIntSize, LayoutPoint, LayoutRect, PaintCmd, PaintList, RectItem,
};

const SELECTION: ColorF = ColorF::new(0.2, 0.4, 0.9, 0.6);
const CARET: ColorF = ColorF::new(0.9, 0.1, 0.1, 1.0);
const OVERLAY: ColorF = ColorF::new(0.0, 179.0 / 255.0, 0.0, 1.0);

fn rect(color: ColorF) -> PaintCmd {
    PaintCmd::DrawRect(RectItem {
        placement: CommonPlacement::new(LayoutRect::new(
            LayoutPoint::new(24.0, 32.0),
            LayoutPoint::new(40.0, 48.0),
        )),
        color,
    })
}

fn color_index(commands: &[PaintCmd], color: ColorF) -> usize {
    commands
        .iter()
        .position(|command| matches!(command, PaintCmd::DrawRect(item) if item.color == color))
        .expect("paint color in command stream")
}

fn exercise(css: &str, scrolled: bool) {
    let dom = StaticDocument::parse(
        "<div id=viewport class=viewport><div id=field class=field>Readable text</div><div id=overlay></div></div>",
    );
    let field = dom
        .first_with_class(dom.document(), "field")
        .expect("field element");
    let viewport = dom
        .first_with_class(dom.document(), "viewport")
        .expect("viewport element");
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&["html, body, div { margin: 0; padding: 0; }", css]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 320.0, 240.0).expect("field layout");
    let scroll = if scrolled {
        HashMap::from([(viewport, (0.0, 16.0)), (field, (0.0, 4.0))])
    } else {
        HashMap::new()
    };
    let mut list = emit_paint_list_with_text_system_scrolled_with_images(
        &dom,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut TextSystem::new(),
        &scroll,
        &HashMap::new(),
    );

    // Control: the old host overlay paints after the author overlay.
    let old_overlay = {
        let mut old = list.clone();
        old.push_overlay_rect(
            LayoutRect::new(LayoutPoint::new(24.0, 32.0), LayoutPoint::new(40.0, 48.0)),
            SELECTION,
        );
        let commands = old.commands();
        color_index(commands, OVERLAY) < color_index(commands, SELECTION)
    };
    assert!(
        old_overlay,
        "control must expose the whole-document overlay order"
    );

    let owner = dom.opaque_id(field);
    list.splice_text_paint_slots(|node, phase| {
        (node == owner).then(|| {
            vec![match phase {
                TextPaintPhase::BeforeContent => rect(SELECTION),
                TextPaintPhase::AfterContent => rect(CARET),
            }]
        })
    });
    let commands = list.commands();
    let selection = color_index(commands, SELECTION);
    let caret = color_index(commands, CARET);
    let overlay = color_index(commands, OVERLAY);
    let glyph = commands
        .iter()
        .position(|command| matches!(command, PaintCmd::DrawText(run) if !run.glyphs.is_empty()))
        .expect("shaped field text");
    assert!(selection < glyph && glyph < caret && caret < overlay);

    if scrolled {
        let clip_before = commands[..selection]
            .iter()
            .filter(|command| matches!(command, PaintCmd::PushClip(_)))
            .count();
        let clip_after = commands[..selection]
            .iter()
            .filter(|command| matches!(command, PaintCmd::PopClip))
            .count();
        let scroll_before = commands[..selection]
            .iter()
            .filter(|command| matches!(command, PaintCmd::PushTransform(_)))
            .count();
        let scroll_after = commands[..selection]
            .iter()
            .filter(|command| matches!(command, PaintCmd::PopTransform))
            .count();
        assert!(
            clip_before > clip_after,
            "selection inherits overflow clipping"
        );
        assert!(
            scroll_before > scroll_after,
            "selection inherits scroll and CSS transforms"
        );
        assert!(
            commands[caret + 1..]
                .iter()
                .any(|command| matches!(command, PaintCmd::PopClip)),
            "caret paints before clipping closes"
        );
    }
}

#[test]
fn normal_field_selection_and_caret_stay_beneath_later_overlay() {
    exercise(
        "#viewport { width: 160px; height: 100px; position: relative; } \
         #field { width: 100px; height: 50px; background: white; } \
         #overlay { position: absolute; left: 0; top: 0; width: 100px; height: 50px; \
                    z-index: 2; background: rgb(0, 179, 0); }",
        false,
    );
}

#[test]
fn positioned_scrolled_field_keeps_text_paint_inside_clip_and_transform() {
    exercise(
        "#viewport { width: 160px; height: 100px; position: relative; overflow: hidden; \
                     transform: translateX(8px); } \
         #field { position: absolute; left: 20px; top: 24px; width: 100px; height: 50px; \
                  overflow: hidden; background: white; } \
         #overlay { position: absolute; left: 0; top: 0; width: 100px; height: 50px; \
                    z-index: 2; background: rgb(0, 179, 0); }",
        true,
    );
}

#[test]
fn text_slots_survive_frame_and_host_leaf_splices_and_skip_hidden_fields() {
    let dom = StaticDocument::parse(
        "<div><iframe></iframe><custom-leaf key=7></custom-leaf><div class=field>Text</div><div class=hidden>Hidden</div></div>",
    );
    let field = dom.first_with_class(dom.document(), "field").unwrap();
    let hidden = dom.first_with_class(dom.document(), "hidden").unwrap();
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[
            "html, body, div { margin: 0; } iframe, custom-leaf { display: block; width: 20px; height: 20px; } .hidden { display: none; }",
        ]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 320.0, 240.0).unwrap();
    let mut list = emit_paint_list_with_text_system_scrolled_with_images(
        &dom,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut TextSystem::new(),
        &HashMap::new(),
        &HashMap::new(),
    );
    let frame = list.frame_slots()[0].owner_node;
    let leaf_color = ColorF::new(0.1, 0.2, 0.3, 1.0);
    let frame_color = ColorF::new(0.3, 0.2, 0.1, 1.0);
    list.splice_host_leaf_slots(|key| (key == 7).then(|| vec![rect(leaf_color)]), |_| None);
    list.splice_frame_slots(|owner| (owner == frame).then(|| vec![rect(frame_color)]));
    let mut hidden_called = false;
    list.splice_text_paint_slots(|owner, phase| {
        hidden_called |= owner == dom.opaque_id(hidden);
        (owner == dom.opaque_id(field)).then(|| {
            vec![match phase {
                TextPaintPhase::BeforeContent => rect(SELECTION),
                TextPaintPhase::AfterContent => rect(CARET),
            }]
        })
    });
    assert!(
        !hidden_called,
        "display:none must not expose text paint slots"
    );
    let commands = list.commands();
    let frame_before_leaf = color_index(commands, frame_color) < color_index(commands, leaf_color);
    assert!(color_index(commands, frame_color) < color_index(commands, SELECTION));
    assert!(color_index(commands, leaf_color) < color_index(commands, SELECTION));
    assert!(color_index(commands, SELECTION) < color_index(commands, CARET));

    // The inverse splice order must also keep earlier text insertions ahead of
    // subsequent host and frame content.
    let mut inverse = emit_paint_list_with_text_system_scrolled_with_images(
        &dom,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut TextSystem::new(),
        &HashMap::new(),
        &HashMap::new(),
    );
    inverse.splice_text_paint_slots(|owner, phase| {
        (owner == dom.opaque_id(field)).then(|| {
            vec![match phase {
                TextPaintPhase::BeforeContent => rect(SELECTION),
                TextPaintPhase::AfterContent => rect(CARET),
            }]
        })
    });
    inverse.splice_frame_slots(|owner| (owner == frame).then(|| vec![rect(frame_color)]));
    inverse.splice_host_leaf_slots(|key| (key == 7).then(|| vec![rect(leaf_color)]), |_| None);
    let commands = inverse.commands();
    assert_eq!(
        color_index(commands, frame_color) < color_index(commands, leaf_color),
        frame_before_leaf,
        "splice order must not change CSS ordering"
    );
    assert!(color_index(commands, frame_color) < color_index(commands, SELECTION));
    assert!(color_index(commands, leaf_color) < color_index(commands, SELECTION));
    assert!(color_index(commands, SELECTION) < color_index(commands, CARET));
}

#[test]
fn equal_index_frame_and_leaf_slots_keep_owner_phase_order_in_either_splice_order() {
    let dom = StaticDocument::parse(
        "<div><iframe class=frame></iframe><custom-leaf class=leaf key=7></custom-leaf></div>",
    );
    let frame_node = dom.first_with_class(dom.document(), "frame").unwrap();
    let leaf_node = dom.first_with_class(dom.document(), "leaf").unwrap();
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[
            "html, body, div { margin: 0; } iframe, custom-leaf { display: block; width: 20px; height: 20px; }",
        ]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 320.0, 240.0).unwrap();
    let frame_color = ColorF::new(0.1, 0.1, 0.8, 1.0);
    let leaf_color = ColorF::new(0.1, 0.8, 0.1, 1.0);
    let frame_before = ColorF::new(0.8, 0.1, 0.1, 1.0);
    let frame_after = ColorF::new(0.8, 0.2, 0.2, 1.0);
    let leaf_before = ColorF::new(0.2, 0.8, 0.2, 1.0);
    let leaf_after = ColorF::new(0.3, 0.8, 0.3, 1.0);

    for text_first in [true, false] {
        let mut list = emit_paint_list_with_text_system_scrolled_with_images(
            &dom,
            &styles,
            &fragments,
            DeviceIntSize::new(320, 240),
            1,
            &mut TextSystem::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        let splice_text = |list: &mut genet_livery::LiveryPaintList| {
            list.splice_text_paint_slots(|owner, phase| {
                let color = if owner == dom.opaque_id(frame_node) {
                    match phase {
                        TextPaintPhase::BeforeContent => frame_before,
                        TextPaintPhase::AfterContent => frame_after,
                    }
                } else if owner == dom.opaque_id(leaf_node) {
                    match phase {
                        TextPaintPhase::BeforeContent => leaf_before,
                        TextPaintPhase::AfterContent => leaf_after,
                    }
                } else {
                    return None;
                };
                Some(vec![rect(color)])
            });
        };
        let splice_children = |list: &mut genet_livery::LiveryPaintList| {
            list.splice_frame_slots(|owner| {
                (owner == dom.opaque_id(frame_node)).then(|| vec![rect(frame_color)])
            });
            list.splice_host_leaf_slots(|key| (key == 7).then(|| vec![rect(leaf_color)]), |_| None);
        };
        if text_first {
            splice_text(&mut list);
            splice_children(&mut list);
        } else {
            splice_children(&mut list);
            splice_text(&mut list);
        }
        let commands = list.commands();
        assert!(
            color_index(commands, frame_before) < color_index(commands, frame_color)
                && color_index(commands, frame_color) < color_index(commands, frame_after),
            "iframe owner phases changed when text_first={text_first}"
        );
        assert!(
            color_index(commands, leaf_color) < color_index(commands, leaf_before)
                && color_index(commands, leaf_before) < color_index(commands, leaf_after),
            "custom leaf owner phases changed when text_first={text_first}"
        );
    }
}

#[test]
fn field_after_content_precedes_its_positive_z_descendant_and_external_overlay() {
    let dom = StaticDocument::parse(
        "<div class=stage><div class=field>Selected text<div class=inner></div></div><div class=outer></div></div>",
    );
    let field = dom.first_with_class(dom.document(), "field").unwrap();
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&["html, body, div { margin: 0; } \
             .stage { position: relative; width: 160px; height: 100px; } \
             .field { position: relative; z-index: 1; width: 100px; height: 50px; \
                      overflow: hidden; transform: translateX(8px); } \
             .inner { position: absolute; z-index: 2; left: 0; top: 0; width: 20px; height: 20px; \
                      background: rgb(0, 179, 0); } \
             .outer { position: absolute; z-index: 3; left: 0; top: 0; width: 20px; height: 20px; \
                      background: rgb(0, 0, 179); }"]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 320.0, 240.0).unwrap();
    let mut list = emit_paint_list_with_text_system_scrolled_with_images(
        &dom,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut TextSystem::new(),
        &HashMap::new(),
        &HashMap::new(),
    );
    list.splice_text_paint_slots(|owner, phase| {
        (owner == dom.opaque_id(field) && phase == TextPaintPhase::AfterContent)
            .then(|| vec![rect(SELECTION), rect(CARET)])
    });
    let commands = list.commands();
    let glyph = commands
        .iter()
        .position(|command| matches!(command, PaintCmd::DrawText(run) if !run.glyphs.is_empty()))
        .unwrap();
    let selection = color_index(commands, SELECTION);
    let caret = color_index(commands, CARET);
    let inner = color_index(commands, OVERLAY);
    let outer = color_index(commands, ColorF::new(0.0, 0.0, 179.0 / 255.0, 1.0));
    assert!(glyph < selection && selection < caret && caret < inner && inner < outer);
    assert!(
        commands[..selection]
            .iter()
            .filter(|command| matches!(command, PaintCmd::PushClip(_)))
            .count()
            > commands[..selection]
                .iter()
                .filter(|command| matches!(command, PaintCmd::PopClip))
                .count(),
        "AfterContent still inherits the field clip"
    );
}
