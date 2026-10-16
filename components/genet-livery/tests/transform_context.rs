// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Positioned descendants use the same transform context in retained layout
//! and paint. A non-`none` individual property establishes that context even
//! when its matrix is the identity.

use genet_livery::{Device, LiveryDocument, LiveryPaintList, StyleSet, TransformGapKind};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName};
use paint_list_api::{ClipKind, ColorF, LayoutTransform, PaintCmd, PaintList};

// Positioned span children opt into block display so their CSS backgrounds
// produce DrawRect commands. The applicability subject remains the parent.
const CSS: &str = "html, body { margin: 0; padding: 0; } \
    #stage { padding-left: 100px; padding-top: 80px; } \
    #anchor { width: 120px; height: 90px; transform-origin: 0 0; } \
    #absolute, #fixed { width: 10px; height: 10px; } \
    #absolute { position: absolute; left: 10px; top: 15px; background: red; } \
    #fixed { position: fixed; left: 20px; top: 25px; background: blue; }";

fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

fn by_id(dom: &ScriptedDom, expected: &str) -> NodeId {
    fn find(dom: &ScriptedDom, node: NodeId, expected: &str) -> Option<NodeId> {
        if dom.kind(node) == NodeKind::Element
            && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(expected)
        {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| find(dom, child, expected))
    }
    find(dom, dom.document(), expected).expect("fixture id")
}

fn document(style: &str) -> LiveryDocument<ScriptedDom> {
    let html = format!(
        "<html><body><div id=stage><div id=anchor style=\"{style}\">\
         <div id=absolute></div><div id=fixed></div></div></div></body></html>"
    );
    let mut dom = ScriptedDom::from_serialized_document(&html);
    dom.drain_mutations(&mut Vec::new());
    LiveryDocument::new(dom, StyleSet::cambium(&[CSS]), Device::screen(320.0, 240.0))
}

fn set_anchor_style(document: &mut LiveryDocument<ScriptedDom>, style: &str) {
    let anchor = by_id(document.dom(), "anchor");
    document.mutate_dom(|dom| dom.set_attribute(anchor, attr("style"), style));
}

fn close(actual: (f32, f32), expected: (f32, f32), label: &str) {
    assert!(
        (actual.0 - expected.0).abs() < 0.01 && (actual.1 - expected.1).abs() < 0.01,
        "{label}: {actual:?} != {expected:?}"
    );
}

/// Read the top-left of a colored box from the actual paint stream, applying
/// each enclosing transform. Its untransformed bounds alone cannot show where
/// the renderer will place a transformed descendant.
fn painted_origin(list: &LiveryPaintList, color: ColorF) -> (f32, f32) {
    let mut transform = LayoutTransform::identity();
    let mut stack = Vec::new();
    let mut found = Vec::new();
    for command in list.commands() {
        match command {
            PaintCmd::PushTransform(spec) => {
                stack.push(transform);
                let local = spec.transform.then(&LayoutTransform::translation(
                    spec.origin.x,
                    spec.origin.y,
                    0.0,
                ));
                transform = local.then(&transform);
            },
            PaintCmd::PopTransform => transform = stack.pop().expect("balanced transform"),
            PaintCmd::DrawRect(item) if item.color == color => {
                let point = transform
                    .transform_point2d(item.placement.bounds.min)
                    .expect("affine fixture paint coordinate");
                found.push((point.x, point.y));
            },
            _ => {},
        }
    }
    assert!(stack.is_empty(), "paint transform scopes balance");
    let [origin] = found.try_into().expect("one colored descendant rectangle");
    origin
}

fn assert_descendants(
    document: &mut LiveryDocument<ScriptedDom>,
    layout_absolute: (f32, f32),
    layout_fixed: (f32, f32),
    paint_absolute: (f32, f32),
    paint_fixed: (f32, f32),
    label: &str,
) {
    let frame = document.frame(320, 240).expect("retained frame");
    let anchor = by_id(document.dom(), "anchor");
    let absolute = by_id(document.dom(), "absolute");
    let fixed = by_id(document.dom(), "fixed");
    assert_eq!(
        document.fragment_rect(anchor),
        Some([100.0, 80.0, 120.0, 90.0]),
        "{label}"
    );
    for (node, expected, name) in [
        (absolute, layout_absolute, "absolute"),
        (fixed, layout_fixed, "fixed"),
    ] {
        let rect = document.fragment_rect(node).expect("positioned fragment");
        close(
            (rect[0], rect[1]),
            expected,
            &format!("{label} {name} layout"),
        );
        close(
            (rect[2], rect[3]),
            (10.0, 10.0),
            &format!("{label} {name} size"),
        );
    }
    close(
        painted_origin(&frame, ColorF::new(1.0, 0.0, 0.0, 1.0)),
        paint_absolute,
        &format!("{label} absolute paint"),
    );
    close(
        painted_origin(&frame, ColorF::new(0.0, 0.0, 1.0, 1.0)),
        paint_fixed,
        &format!("{label} fixed paint"),
    );
}

#[test]
fn individual_properties_capture_absolute_and_fixed_descendants_in_layout_and_paint() {
    // The zero/one values deliberately have identity matrices. The CSS value
    // is nevertheless non-`none`, so both descendants use #anchor as their
    // containing block. The non-identity values also check painted placement.
    let cases = [
        ("translate: 0px 0px", (110.0, 95.0), (120.0, 105.0)),
        ("rotate: 0deg", (110.0, 95.0), (120.0, 105.0)),
        ("scale: 1", (110.0, 95.0), (120.0, 105.0)),
        ("translate: 20px 30px", (130.0, 125.0), (140.0, 135.0)),
        ("rotate: 90deg", (85.0, 90.0), (75.0, 100.0)),
        ("scale: 2", (120.0, 110.0), (140.0, 130.0)),
    ];
    for (style, painted_absolute, painted_fixed) in cases {
        let mut retained = document(style);
        assert_descendants(
            &mut retained,
            (110.0, 95.0),
            (120.0, 105.0),
            painted_absolute,
            painted_fixed,
            style,
        );
    }
}

#[test]
fn entering_and_leaving_each_individual_transform_repositions_retained_descendants() {
    for style in ["translate: 0px 0px", "rotate: 0deg", "scale: 1"] {
        let mut retained = document("");
        assert_descendants(
            &mut retained,
            (10.0, 15.0),
            (20.0, 25.0),
            (10.0, 15.0),
            (20.0, 25.0),
            "initial none",
        );
        set_anchor_style(&mut retained, style);
        assert_descendants(
            &mut retained,
            (110.0, 95.0),
            (120.0, 105.0),
            (110.0, 95.0),
            (120.0, 105.0),
            style,
        );
        set_anchor_style(&mut retained, "");
        assert_descendants(
            &mut retained,
            (10.0, 15.0),
            (20.0, 25.0),
            (10.0, 15.0),
            (20.0, 25.0),
            "restored none",
        );
    }
}

#[test]
fn inline_item_through_display_contents_captures_descendants_in_flex_and_grid() {
    // The span's computed display remains inline. Its effective flex/grid
    // parent blockifies its generated box through the contents wrapper.
    for display in ["flex", "grid"] {
        let html = "<html><body><div id=stage><div id=contents>\
                    <span id=anchor><span id=absolute></span><span id=fixed></span></span>\
                    </div></div></body></html>";
        let mut dom = ScriptedDom::from_serialized_document(html);
        dom.drain_mutations(&mut Vec::new());
        let css =
            format!("{CSS} #stage {{ display: {display}; }} #contents {{ display: contents; }}");
        let mut retained = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(320.0, 240.0),
        );
        assert_descendants(
            &mut retained,
            (10.0, 15.0),
            (20.0, 25.0),
            (10.0, 15.0),
            (20.0, 25.0),
            &format!("{display} initial none"),
        );
        set_anchor_style(&mut retained, "translate: 0px 0px");
        assert_descendants(
            &mut retained,
            (110.0, 95.0),
            (120.0, 105.0),
            (110.0, 95.0),
            (120.0, 105.0),
            &format!("{display} identity translate"),
        );
        set_anchor_style(&mut retained, "");
        assert_descendants(
            &mut retained,
            (10.0, 15.0),
            (20.0, 25.0),
            (10.0, 15.0),
            (20.0, 25.0),
            &format!("{display} restored none"),
        );
    }
}

#[test]
fn static_item_through_display_contents_keeps_its_numeric_z_index() {
    for display in ["flex", "grid"] {
        let html = "<html><body><div id=stage><div id=contents>\
                    <span id=item></span></div><div id=overlay></div></div></body></html>";
        let mut dom = ScriptedDom::from_serialized_document(html);
        dom.drain_mutations(&mut Vec::new());
        let css = format!(
            "html, body {{ margin: 0; }} \
             #stage {{ display: {display}; position: relative; width: 100px; height: 100px; }} \
             #contents {{ display: contents; }} \
             #item {{ width: 40px; height: 40px; z-index: 2; translate: 0px 0px; background: red; }} \
             #overlay {{ position: absolute; left: 0; top: 0; width: 40px; height: 40px; \
                         z-index: 1; background: blue; }}"
        );
        let mut retained = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(320.0, 240.0),
        );
        let frame = retained.frame(320, 240).expect("stacking frame");
        let item = by_id(retained.dom(), "item");
        assert_eq!(retained.fragment_rect(item), Some([0.0, 0.0, 40.0, 40.0]));
        let color_index = |color| {
            frame
                .commands()
                .iter()
                .position(|cmd| matches!(cmd, PaintCmd::DrawRect(rect) if rect.color == color))
                .expect("colored overlap rectangle")
        };
        let red = color_index(ColorF::new(1.0, 0.0, 0.0, 1.0));
        let blue = color_index(ColorF::new(0.0, 0.0, 1.0, 1.0));
        assert!(blue < red, "{display}: static item lost z-index: 2");
        assert_eq!(retained.hit_test(10.0, 10.0), Some(item), "{display}");
    }
}

#[test]
fn perspective_and_preserve_3d_capture_positioned_descendants_without_an_own_matrix() {
    for style in [
        "perspective: 400px; perspective-origin: 0 0",
        "transform-style: preserve-3d",
    ] {
        let mut retained = document(style);
        assert_descendants(
            &mut retained,
            (110.0, 95.0),
            (120.0, 105.0),
            (110.0, 95.0),
            (120.0, 105.0),
            style,
        );
    }
}

#[test]
fn non_replaced_inline_transform_values_do_not_capture_positioned_descendants() {
    // CSS transforms do not apply to a non-replaced inline. Explicit insets
    // keep both children independent of the inline's line fragment geometry.
    for style in [
        "translate: 20px 30px",
        "rotate: 90deg",
        "scale: 2",
        "perspective: 400px",
        "transform-style: preserve-3d",
    ] {
        let html = format!(
            "<html><body><div id=stage><span id=anchor style=\"{style}\">host\
             <span id=absolute></span><span id=fixed></span></span></div></body></html>"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let mut retained =
            LiveryDocument::new(dom, StyleSet::cambium(&[CSS]), Device::screen(320.0, 240.0));
        let frame = retained.frame(320, 240).expect("inline retained frame");
        for (name, expected) in [("absolute", (10.0, 15.0)), ("fixed", (20.0, 25.0))] {
            let node = by_id(retained.dom(), name);
            let rect = retained
                .fragment_rect(node)
                .expect("positioned inline child");
            close(
                (rect[0], rect[1]),
                expected,
                &format!("{style} {name} layout"),
            );
        }
        close(
            painted_origin(&frame, ColorF::new(1.0, 0.0, 0.0, 1.0)),
            (10.0, 15.0),
            &format!("{style} absolute paint"),
        );
        close(
            painted_origin(&frame, ColorF::new(0.0, 0.0, 1.0, 1.0)),
            (20.0, 25.0),
            &format!("{style} fixed paint"),
        );
    }
}

#[test]
fn non_replaced_inline_perspective_and_preserve_3d_do_not_change_child_matrix() {
    // The positioned span is blockified by box generation, while its parent
    // remains a non-replaced inline for transform applicability.
    let render = |style: &str| {
        let html = format!(
            "<html><body><div id=stage><span id=anchor style=\"{style}\">host\
             <span id=child></span></span></div></body></html>"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let css = "html, body { margin: 0; } #stage { padding: 80px 0 0 100px; } \
            #child { position: absolute; left: 20px; top: 30px; width: 40px; height: 40px; \
                     transform-origin: 0 0; rotate: x 45deg; background: red; }";
        let mut retained =
            LiveryDocument::new(dom, StyleSet::cambium(&[css]), Device::screen(320.0, 240.0));
        retained.frame(320, 240).expect("inline parent frame")
    };
    let plain = render("");
    let transform = |frame: &LiveryPaintList| {
        frame
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                PaintCmd::PushTransform(spec) => Some((spec.transform, spec.origin)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let expected = transform(&plain);
    assert_eq!(expected.len(), 1, "only the child rotates");
    for style in ["perspective: 400px", "transform-style: preserve-3d"] {
        let frame = render(style);
        assert_eq!(
            transform(&frame),
            expected,
            "{style}: non-replaced inline changed the child's paint matrix"
        );
        close(
            painted_origin(&frame, ColorF::new(1.0, 0.0, 0.0, 1.0)),
            painted_origin(&plain, ColorF::new(1.0, 0.0, 0.0, 1.0)),
            &format!("{style} child paint"),
        );
    }
}

#[test]
fn contents_does_not_interrupt_parent_perspective_or_preserve_3d() {
    let capture = |parent_style: &str, through_contents: bool| {
        let open = if through_contents {
            "<div id=contents style='display:contents'>"
        } else {
            ""
        };
        let close = if through_contents { "</div>" } else { "" };
        let html = format!(
            "<html><body><div id=stage><div id=parent>{open}\
             <span id=child></span>{close}</div></div></body></html>"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let css = format!(
            "html, body {{ margin: 0; }} \
             #stage {{ padding-left: 100px; padding-top: 80px; }} \
             #parent {{ position: relative; width: 120px; height: 90px; {parent_style} }} \
             #child {{ position: absolute; left: 20px; top: 30px; width: 40px; height: 40px; \
                       transform-origin: 0 0; rotate: x 45deg; background: red; }}"
        );
        let mut retained = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(320.0, 240.0),
        );
        let frame = retained.frame(320, 240).expect("3D frame");
        let matrices = frame
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                PaintCmd::PushTransform(spec) => Some((spec.transform, spec.origin)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(matrices.len(), 1, "only the child has its own transform");
        let painted = painted_origin(&frame, ColorF::new(1.0, 0.0, 0.0, 1.0));
        let child = by_id(retained.dom(), "child");
        let hit_child = retained.hit_test(painted.0 + 5.0, painted.1 + 5.0) == Some(child);
        (matrices, painted, hit_child)
    };

    let without_context = capture("", false);
    for style in [
        "perspective: 400px; perspective-origin: 0 0;",
        "transform-style: preserve-3d;",
    ] {
        let direct = capture(style, false);
        let through_contents = capture(style, true);
        assert_ne!(direct.0, without_context.0, "{style}: context witness");
        assert_eq!(through_contents.0, direct.0, "{style}: paint matrix");
        close(
            through_contents.1,
            direct.1,
            &format!("{style} painted origin"),
        );
        assert!(direct.2, "{style}: the rotated child must be hittable");
        assert_eq!(through_contents.2, direct.2, "{style}: hit testing");
    }
}

#[test]
fn table_caption_uses_its_wrapper_owners_3d_context() {
    let capture = |table_style: &str, caption_style: &str| {
        let html = "<html><body><table id=table><caption id=caption>Caption</caption>\
                    <tr><td>Cell</td></tr></table></body></html>";
        let mut dom = ScriptedDom::from_serialized_document(html);
        dom.drain_mutations(&mut Vec::new());
        let css = format!(
            "html, body {{ margin: 0; }} \
             #table {{ width: 120px; {table_style} }} \
             #caption {{ transform-origin: 0 0; rotate: x 45deg; \
                         background: red; {caption_style} }}"
        );
        let mut retained = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(320.0, 240.0),
        );
        let frame = retained.frame(320, 240).expect("caption frame");
        let caption = by_id(retained.dom(), "caption");
        assert!(
            retained.fragment_rect(caption).is_some(),
            "caption box exists"
        );
        let matrices = frame
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                PaintCmd::PushTransform(spec) => Some(spec.transform),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(matrices.len(), 1, "only the caption is transformed");
        (matrices[0], frame.transform_gaps().to_vec())
    };

    let flat_perspective = capture("", "transform-style: preserve-3d;");
    let with_perspective = capture(
        "perspective: 400px; perspective-origin: 0 0;",
        "transform-style: preserve-3d;",
    );
    assert_ne!(with_perspective.0, flat_perspective.0);
    assert!(
        with_perspective
            .1
            .iter()
            .any(|gap| gap.kind == TransformGapKind::PerspectiveDivide),
        "the existing affine perspective gap remains reported"
    );

    let flat_preserve = capture("", "");
    let with_preserve = capture("transform-style: preserve-3d;", "");
    assert_ne!(with_preserve.0, flat_preserve.0);
}

#[test]
fn positioned_inline_clip_encloses_hoisted_child_paint_and_hit() {
    let html = "<html><body><div id=stage><span id=clip>\
                <span id=overlay></span></span></div></body></html>";
    let mut dom = ScriptedDom::from_serialized_document(html);
    dom.drain_mutations(&mut Vec::new());
    let css = "html, body { margin: 0; } #stage { position: relative; } \
               #clip { position: absolute; left: 0; top: 0; width: 50px; height: 50px; \
                       overflow: hidden; } \
               #overlay { position: absolute; left: 40px; top: 0; width: 30px; height: 30px; \
                          z-index: 1; background: red; }";
    let mut retained =
        LiveryDocument::new(dom, StyleSet::cambium(&[css]), Device::screen(320.0, 240.0));
    let frame = retained.frame(320, 240).expect("clipped frame");
    let overlay = by_id(retained.dom(), "overlay");
    let red = frame
        .commands()
        .iter()
        .position(|cmd| matches!(cmd, PaintCmd::DrawRect(rect) if rect.color == ColorF::new(1.0, 0.0, 0.0, 1.0)))
        .expect("positioned inline child paints");
    let clip = frame.commands()[..red]
        .iter()
        .rposition(|cmd| matches!(cmd, PaintCmd::PushClip(_)))
        .expect("the hoisted child keeps its ancestor clip");
    let PaintCmd::PushClip(spec) = &frame.commands()[clip] else {
        unreachable!();
    };
    let ClipKind::Rect(rect) = &spec.kind else {
        panic!("overflow hidden uses a rectangular clip");
    };
    close(
        (rect.width(), rect.height()),
        (50.0, 50.0),
        "positioned inline clip size",
    );
    assert!(
        frame.commands()[red + 1..]
            .iter()
            .any(|cmd| matches!(cmd, PaintCmd::PopClip)),
        "the clip closes after the child paint"
    );
    assert_eq!(retained.hit_test(45.0, 5.0), Some(overlay));
    assert_ne!(retained.hit_test(60.0, 5.0), Some(overlay));
}

#[test]
fn replaced_inline_canvas_applies_an_individual_transform_to_its_paint() {
    let paint = |style: &str| {
        let html = format!(
            "<html><body><div style='padding-left:100px;padding-top:80px'>\
             <canvas width=20 height=20 style=\"{style}\"></canvas></div></body></html>"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let mut retained = LiveryDocument::new(
            dom,
            StyleSet::cambium(&["html, body { margin: 0; } canvas { background: red; }"]),
            Device::screen(320.0, 240.0),
        );
        retained.frame(320, 240).expect("inline canvas frame")
    };
    let plain = paint("");
    let moved = paint("translate: 20px 30px");
    let red = ColorF::new(1.0, 0.0, 0.0, 1.0);
    let before = painted_origin(&plain, red);
    let after = painted_origin(&moved, red);
    close(
        (after.0 - before.0, after.1 - before.1),
        (20.0, 30.0),
        "replaced inline paint translation",
    );
}

#[test]
fn individual_properties_and_perspective_keep_high_descendants_in_their_stacking_context() {
    for style in [
        "translate: 0px 0px",
        "rotate: 0deg",
        "scale: 1",
        "perspective: 400px",
        "transform-style: preserve-3d",
    ] {
        let html = format!(
            "<html><body><div id=stage><div id=anchor style=\"{style}\">\
             <div id=escape></div></div><div id=overlay></div></div></body></html>"
        );
        let mut dom = ScriptedDom::from_serialized_document(&html);
        dom.drain_mutations(&mut Vec::new());
        let css = "html, body { margin: 0; } #stage { position: relative; } \
            #anchor { width: 40px; height: 40px; } \
            #escape { position: absolute; z-index: 100; width: 20px; height: 20px; background: red; } \
            #overlay { position: absolute; z-index: 1; width: 20px; height: 20px; background: blue; }";
        let mut retained =
            LiveryDocument::new(dom, StyleSet::cambium(&[css]), Device::screen(320.0, 240.0));
        let frame = retained.frame(320, 240).expect("stacking frame");
        let color_index = |color| {
            frame
                .commands()
                .iter()
                .position(|cmd| matches!(cmd, PaintCmd::DrawRect(item) if item.color == color))
                .expect("colored stacking rectangle")
        };
        let escape = color_index(ColorF::new(1.0, 0.0, 0.0, 1.0));
        let overlay = color_index(ColorF::new(0.0, 0.0, 1.0, 1.0));
        assert!(escape < overlay, "{style}: descendant escaped its context");
    }
}
