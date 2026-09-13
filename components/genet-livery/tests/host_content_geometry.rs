// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent CSS-pixel edge oracles for the custom-leaf host boundary.
//! These inspect actual paint commands; GPU raster acceptance is host-owned.

use genet_livery::{
    ElementGeometry, InteractionStates, LiveryLayout, LiveryPaintList, StylePlane, StyleSet,
    TextSystem, element_geometry, emit_paint_list_with_text_system_scrolled_with_images,
    hit_test_with_scroll, layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::{
    media::{Device, SystemPalette},
    values::{ColorScheme, SystemColor},
};
use paint_list_api::{
    ClipKind, CommonPlacement, DeviceIntSize, ExternalTextureItem, LayoutPoint, LayoutRect,
    LayoutTransform, PaintCmd, PaintList,
};
use std::collections::HashMap;

type Node = <StaticDocument as LayoutDom>::NodeId;

struct Fixture {
    dom: StaticDocument,
    styles: StylePlane<Node>,
    fragments: LiveryLayout<Node>,
    scroll: HashMap<Node, (f32, f32)>,
}

impl Fixture {
    fn new(html: &str, css: &str) -> Self {
        let dom = StaticDocument::parse(html);
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[
                "html, body { margin: 0; padding: 0; } custom-leaf { display: block; }",
                css,
            ]),
            &Device::screen(320.0, 240.0),
            &InteractionStates::default(),
        );
        let fragments = layout(&dom, &styles, 320.0, 240.0).unwrap();
        Self {
            dom,
            styles,
            fragments,
            scroll: HashMap::new(),
        }
    }

    fn node(&self, class: &str) -> Node {
        self.dom
            .first_with_class(self.dom.document(), class)
            .unwrap()
    }

    fn geometry(&self) -> ElementGeometry {
        element_geometry(
            &self.dom,
            &self.styles,
            &self.fragments,
            &self.scroll,
            self.node("leaf"),
        )
        .unwrap()
    }

    fn hit(&self, point: (f32, f32)) -> Option<Node> {
        hit_test_with_scroll(
            &self.dom,
            &self.styles,
            &self.fragments,
            &self.scroll,
            point.0,
            point.1,
        )
    }

    fn paint(&self) -> LiveryPaintList {
        let mut list = emit_paint_list_with_text_system_scrolled_with_images(
            &self.dom,
            &self.styles,
            &self.fragments,
            DeviceIntSize::new(320, 240),
            1,
            &mut TextSystem::new(),
            &self.scroll,
            &HashMap::new(),
        );
        let (width, height) = self.geometry().content_size();
        list.splice_host_leaf_slots(
            |key| {
                assert_eq!(key, 7);
                Some(vec![PaintCmd::DrawExternalTexture(ExternalTextureItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(0.0, 0.0),
                        LayoutPoint::new(width, height),
                    )),
                    texture_key: 71,
                    opacity: 1.0,
                    content_generation: Some(1),
                })])
            },
            |_| None,
        );
        list
    }
}

fn close(actual: (f32, f32), expected: (f32, f32)) {
    assert!(
        (actual.0 - expected.0).abs() < 0.0001 && (actual.1 - expected.1).abs() < 0.0001,
        "{actual:?} != independently calculated {expected:?}"
    );
}

fn inside(rect: LayoutRect, point: LayoutPoint) -> bool {
    point.x >= rect.min.x && point.x < rect.max.x && point.y >= rect.min.y && point.y < rect.max.y
}

/// Evaluate the emitted affine scopes through euclid, separately from Genet's
/// Matrix2D inverse. The fixtures below supply literal CSS edge coordinates.
fn painted_content(list: &LiveryPaintList, point: (f32, f32)) -> Option<(f32, f32)> {
    let mut transform = LayoutTransform::identity();
    let mut transforms = Vec::new();
    let mut clips = Vec::new();
    let point = LayoutPoint::new(point.0, point.1);
    let mut result = None;
    for command in list.commands() {
        match command {
            PaintCmd::PushTransform(spec) => {
                transforms.push(transform);
                let local = spec.transform.then(&LayoutTransform::translation(
                    spec.origin.x,
                    spec.origin.y,
                    0.0,
                ));
                transform = local.then(&transform);
            },
            PaintCmd::PopTransform => transform = transforms.pop().unwrap(),
            PaintCmd::PushClip(spec) => {
                let ClipKind::Rect(rect) = spec.kind else {
                    panic!("fixture expects rectangular clips");
                };
                clips.push((transform, rect));
            },
            PaintCmd::PopClip => {
                clips.pop().unwrap();
            },
            PaintCmd::DrawExternalTexture(item) => {
                let local = transform
                    .inverse()
                    .and_then(|inverse| inverse.transform_point2d(point));
                let visible = clips
                    .iter()
                    .all(|(matrix, rect): &(LayoutTransform, LayoutRect)| {
                        matrix
                            .inverse()
                            .and_then(|inverse| inverse.transform_point2d(point))
                            .is_some_and(|point| inside(*rect, point))
                    });
                if let Some(local) =
                    local.filter(|local| visible && inside(item.placement.bounds, *local))
                {
                    result = Some((
                        local.x - item.placement.bounds.min.x,
                        local.y - item.placement.bounds.min.y,
                    ));
                }
            },
            _ => {},
        }
    }
    assert!(
        transforms.is_empty() && clips.is_empty(),
        "paint scopes balance"
    );
    result
}

#[test]
fn border_padding_and_own_scroll_keep_the_producer_in_its_content_box() {
    let mut fixture = Fixture::new(
        "<custom-leaf class=leaf key=7></custom-leaf>",
        ".leaf { position: absolute; left: 8px; top: 112px; width: 50px; height: 40px; border: 4px solid black; padding: 4px; }",
    );
    fixture.scroll.insert(fixture.node("leaf"), (7.0, 9.0));
    let geometry = fixture.geometry();
    assert_eq!(geometry.content_rect, (16.0, 120.0, 50.0, 40.0));
    assert_eq!(geometry.border_size(), (66.0, 56.0));
    let paint = fixture.paint();
    for (point, local) in [
        ((16.5, 120.5), (0.5, 0.5)),
        ((65.5, 120.5), (49.5, 0.5)),
        ((16.5, 159.5), (0.5, 39.5)),
        ((65.5, 159.5), (49.5, 39.5)),
    ] {
        close(geometry.map_to_content(point.0, point.1).unwrap(), local);
        close(painted_content(&paint, point).unwrap(), local);
        assert_eq!(fixture.hit(point), Some(fixture.node("leaf")));
    }
    for point in [(15.5, 140.0), (66.5, 140.0), (40.0, 119.5), (40.0, 160.5)] {
        assert!(geometry.map_to_content(point.0, point.1).is_none());
        assert!(painted_content(&paint, point).is_none());
    }
    close(geometry.map_to_local(15.5, 140.0).unwrap(), (-0.5, 20.0));
    close(
        geometry.map_to_border_local(15.5, 140.0).unwrap(),
        (7.5, 28.0),
    );
}

#[test]
fn plain_retained_leaf_keeps_zero_clip_and_layer_depth() {
    let fixture = Fixture::new(
        "<custom-leaf class=leaf key=7></custom-leaf>",
        ".leaf { width: 80px; height: 40px; padding: 4px; border: 2px solid black; }",
    );
    let mut list = emit_paint_list_with_text_system_scrolled_with_images(
        &fixture.dom,
        &fixture.styles,
        &fixture.fragments,
        DeviceIntSize::new(320, 240),
        1,
        &mut TextSystem::new(),
        &fixture.scroll,
        &HashMap::new(),
    );
    list.splice_host_leaf_slots(
        |_| panic!("retained fragment takes precedence"),
        |key| {
            assert_eq!(key, 7);
            Some(91)
        },
    );
    let mut depth = 0;
    let mut seen = 0;
    for command in list.commands() {
        match command {
            PaintCmd::PushClip(_) | PaintCmd::PushLayer(_) => depth += 1,
            PaintCmd::PopClip | PaintCmd::PopLayer => depth -= 1,
            PaintCmd::PlaceRetainedFragment(fragment) => {
                assert_eq!(
                    depth, 0,
                    "the host slot must not create a retained-fragment fallback"
                );
                assert_eq!(fragment.id, 91);
                assert_eq!(fragment.origin, LayoutPoint::new(6.0, 6.0));
                seen += 1;
            },
            _ => {},
        }
    }
    assert_eq!(seen, 1);
    assert_eq!(depth, 0);
}

#[test]
fn nested_transform_with_noncentral_origins_matches_all_four_content_edges() {
    let fixture = Fixture::new("<div class=outer><custom-leaf class=leaf key=7></custom-leaf></div>",
        ".outer { position: absolute; left: 100px; top: 50px; width: 120px; height: 100px; transform-origin: 20px 10px; transform: translate(10px, 5px) scale(2); }
         .leaf { position: absolute; left: 10px; top: 20px; width: 40px; height: 20px; padding: 3px; border: 2px solid black; transform-origin: 0 0; transform: rotate(90deg); }");
    let geometry = fixture.geometry();
    let paint = fixture.paint();
    // Parent: (x,y) -> (2x-110,2y-55); child rotates about (110,70).
    // Content origin (115,75) therefore maps to (100,95).
    let world = |local: (f32, f32)| (100.0 - 2.0 * local.1, 95.0 + 2.0 * local.0);
    for local in [
        (0.5, 0.5),
        (39.5, 0.5),
        (0.5, 19.5),
        (39.5, 19.5),
        (20.0, 10.0),
    ] {
        let point = world(local);
        close(geometry.map_to_content(point.0, point.1).unwrap(), local);
        close(painted_content(&paint, point).unwrap(), local);
        assert_eq!(fixture.hit(point), Some(fixture.node("leaf")));
    }
    for local in [(-0.5, 10.0), (40.5, 10.0), (20.0, -0.5), (20.0, 20.5)] {
        let point = world(local);
        assert!(geometry.map_to_content(point.0, point.1).is_none());
        assert!(painted_content(&paint, point).is_none());
    }
}

#[test]
fn rotated_aabb_corner_does_not_steal_a_pointer() {
    let fixture = Fixture::new(
        "<custom-leaf class=leaf key=7></custom-leaf>",
        ".leaf { position: absolute; left: 60px; top: 80px; width: 80px; height: 40px; transform: rotate(45deg); }",
    );
    // AABB is approximately [57.57,142.43] in both axes. (60,60) is outside
    // the actual rotated rectangle, despite being inside that AABB.
    assert_ne!(fixture.hit((60.0, 60.0)), Some(fixture.node("leaf")));
    assert!(fixture.geometry().map_to_content(60.0, 60.0).is_none());
    assert!(painted_content(&fixture.paint(), (60.0, 60.0)).is_none());
    assert_eq!(fixture.hit((100.0, 100.0)), Some(fixture.node("leaf")));
}

#[test]
fn flattened_stacking_leaf_keeps_scroll_between_transformed_clip_ancestors() {
    let mut fixture = Fixture::new("<div class=outer><div class=inner><custom-leaf class=leaf key=7></custom-leaf></div></div>",
        ".outer { position: absolute; left: 20px; top: 30px; width: 100px; height: 80px; overflow: hidden; transform-origin: 0 0; transform: scale(2); }
         .inner { position: relative; left: 10px; top: 5px; width: 70px; height: 60px; overflow: hidden; }
         .leaf { position: absolute; left: 25px; top: 20px; width: 80px; height: 50px; z-index: 1; }");
    fixture.scroll.insert(fixture.node("outer"), (4.0, 3.0));
    fixture.scroll.insert(fixture.node("inner"), (8.0, 6.0));
    // Leaf origin: (55,55)-(4,3)-(8,6), then outer scale about (20,30).
    // Inner clip: [30,100]x[35,95] minus outer scroll, then outer scale.
    // Visible image: [66,172)x[62,154), with source origin (66,62).
    let geometry = fixture.geometry();
    let paint = fixture.paint();
    for point in [(66.5, 62.5), (171.5, 62.5), (66.5, 153.5), (171.5, 153.5)] {
        let expected = ((point.0 - 66.0) / 2.0, (point.1 - 62.0) / 2.0);
        close(geometry.map_to_content(point.0, point.1).unwrap(), expected);
        close(painted_content(&paint, point).unwrap(), expected);
        assert_eq!(fixture.hit(point), Some(fixture.node("leaf")));
    }
    for point in [(65.5, 90.0), (172.5, 90.0), (90.0, 61.5), (90.0, 154.5)] {
        assert!(
            geometry.map_to_content(point.0, point.1).is_none(),
            "{point:?}"
        );
        assert!(painted_content(&paint, point).is_none(), "{point:?}");
        assert_ne!(fixture.hit(point), Some(fixture.node("leaf")), "{point:?}");
    }
}

#[test]
fn overflow_clips_at_padding_edge_and_document_scroll_is_caller_owned() {
    let fixture = Fixture::new("<div class=outer><custom-leaf class=leaf key=7></custom-leaf></div>",
        ".outer { position: absolute; left: 20px; top: 60px; width: 80px; height: 60px; box-sizing: border-box; border: 5px solid black; overflow: hidden; }
         .leaf { position: absolute; left: -20px; top: -20px; width: 140px; height: 120px; }");
    for point in [(25.5, 65.5), (94.5, 65.5), (25.5, 114.5), (94.5, 114.5)] {
        assert_eq!(fixture.hit(point), Some(fixture.node("leaf")));
        assert!(painted_content(&fixture.paint(), point).is_some());
    }
    for point in [(24.5, 80.0), (95.5, 80.0), (40.0, 64.5), (40.0, 115.5)] {
        assert_ne!(fixture.hit(point), Some(fixture.node("leaf")));
        assert!(
            fixture
                .geometry()
                .map_to_content(point.0, point.1)
                .is_none()
        );
    }
    let viewport_point = (35.0, 40.0);
    let document_scroll = (0.0, 40.0);
    let document_point = (
        viewport_point.0 + document_scroll.0,
        viewport_point.1 + document_scroll.1,
    );
    assert_eq!(fixture.hit(document_point), Some(fixture.node("leaf")));
    assert!(
        fixture
            .geometry()
            .map_to_content(document_point.0, document_point.1)
            .is_some()
    );
}

#[test]
fn transformed_and_translucent_contexts_keep_descendants_below_later_dom_overlay() {
    for context in ["transform: translate(0px);", "opacity: .5;"] {
        let fixture = Fixture::new("<div class=group><custom-leaf class=leaf key=7></custom-leaf></div><button class=overlay></button>",
            &format!(".group {{ position: absolute; width: 80px; height: 60px; {context} }}
            .leaf {{ position: absolute; width: 80px; height: 60px; z-index: 100; }}
            .overlay {{ position: absolute; left: 10px; top: 10px; width: 40px; height: 30px; z-index: 1; background: rgb(0 0 255 / .5); }}"));
        assert_eq!(fixture.hit((20.0, 20.0)), Some(fixture.node("overlay")));
        assert_eq!(fixture.hit((5.0, 5.0)), Some(fixture.node("leaf")));
        let paint = fixture.paint();
        let leaf = paint
            .commands()
            .iter()
            .position(|cmd| matches!(cmd, PaintCmd::DrawExternalTexture(_)))
            .unwrap();
        let overlay = paint
            .commands()
            .iter()
            .rposition(|cmd| matches!(cmd, PaintCmd::DrawRect(item) if item.color.b > 0.9))
            .unwrap();
        assert!(leaf < overlay);
        if context.starts_with("opacity") {
            let mut layers = Vec::new();
            for cmd in &paint.commands()[..leaf] {
                match cmd {
                    PaintCmd::PushLayer(layer) => layers.push(layer.opacity),
                    PaintCmd::PopLayer => {
                        layers.pop().unwrap();
                    },
                    _ => {},
                }
            }
            assert_eq!(
                layers,
                [0.5],
                "group opacity wraps the producer exactly once"
            );
        }
    }
}

#[test]
fn singular_and_nonfinite_input_refuse_mapping_without_losing_content_size() {
    let fixture = Fixture::new(
        "<custom-leaf class=leaf key=7></custom-leaf>",
        ".leaf { width: 80px; height: 40px; transform: scale(0); }",
    );
    let geometry = fixture.geometry();
    assert_eq!(geometry.content_size(), (80.0, 40.0));
    assert!(geometry.map_to_local(40.0, 20.0).is_none());
    assert!(geometry.map_to_border_local(40.0, 20.0).is_none());
    assert!(geometry.map_to_content(40.0, 20.0).is_none());
    assert_ne!(fixture.hit((40.0, 20.0)), Some(fixture.node("leaf")));
    assert!(fixture.hit((f32::NAN, 0.0)).is_none());
    assert!(fixture.hit((0.0, f32::INFINITY)).is_none());
}

#[test]
fn display_none_and_hidden_ancestors_suspend_geometry_and_pointer_hits() {
    for hidden in ["display: none;", "visibility: hidden;"] {
        let fixture = Fixture::new(
            "<div class=outer><custom-leaf class=leaf key=7></custom-leaf></div>",
            &format!(
                ".outer {{ {hidden} }} .leaf {{ width: 80px; height: 40px; visibility: visible; }}"
            ),
        );
        assert!(
            element_geometry(
                &fixture.dom,
                &fixture.styles,
                &fixture.fragments,
                &fixture.scroll,
                fixture.node("leaf")
            )
            .is_none()
        );
        assert_ne!(fixture.hit((10.0, 10.0)), Some(fixture.node("leaf")));
    }
}

#[test]
fn used_foreground_keeps_encoded_channels_alpha_palette_and_element_scheme() {
    let dom = StaticDocument::parse("<div class=parent><i class=leaf></i></div>");
    let mut palette = SystemPalette::default();
    palette.set(
        ColorScheme::Light,
        SystemColor::CanvasText,
        "#102030".parse().unwrap(),
    );
    palette.set(
        ColorScheme::Dark,
        SystemColor::CanvasText,
        "rgb(64 128 192 / .5)".parse().unwrap(),
    );
    let mut device = Device::screen(320.0, 240.0);
    device.set_system_palette(palette);
    let plane = resolve_styles(
        &dom,
        &StyleSet::cambium(&[
            ".parent { color: CanvasText; color-scheme: light; } .leaf { color: CanvasText; color-scheme: dark; }",
        ]),
        &device,
        &InteractionStates::default(),
    );
    let id = |class| dom.first_with_class(dom.document(), class).unwrap();
    let color = plane.used_color(id("leaf")).unwrap();
    for (actual, expected) in
        color
            .into_iter()
            .zip([64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0, 0.5])
    {
        assert!((actual - expected).abs() < 0.00001);
    }
    assert_ne!(plane.used_color(id("parent")), plane.used_color(id("leaf")));
}
