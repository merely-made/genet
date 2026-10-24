// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName};
use netrender::vello_backends::{CpuSession, SparseResourceLimits};
use paint_list_render::translate_paint_list;

fn target(dom: &ScriptedDom) -> NodeId {
    fn find(dom: &ScriptedDom, node: NodeId) -> Option<NodeId> {
        if dom.kind(node) == NodeKind::Element
            && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some("target")
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child))
    }
    find(dom, dom.document()).expect("target element")
}

fn pixel_at(bytes: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    bytes[offset..offset + 4].try_into().expect("RGBA pixel")
}

#[test]
fn retained_background_color_repaints_cpu_pixel_without_relayout() {
    let mut dom =
        ScriptedDom::from_serialized_document("<html><body><div id=target></div></body></html>");
    dom.drain_mutations(&mut Vec::new());
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&["html, body { margin: 0; padding: 0; } \
             #target { width: 32px; height: 32px; background-color: rgb(255, 0, 0); }"]),
        Device::screen(64.0, 64.0),
    );
    let target = target(document.dom());
    let mut cpu = CpuSession::new(SparseResourceLimits::default());

    let first = document.frame(64, 64).expect("initial frame");
    let layout_generation = document.layout_generation();
    let first_scene = translate_paint_list(&first).expect("initial paint translation");
    let first_pixels = cpu.render(&first_scene).expect("initial CPU raster");
    assert_eq!(
        pixel_at(first_pixels.data_as_u8_slice(), 64, 16, 16),
        [255, 0, 0, 255]
    );
    assert_eq!(document.hit_test(16.0, 16.0), Some(target));

    document.mutate_dom(|dom| {
        dom.set_attribute(
            target,
            QualName::new(None, Namespace::from(""), LocalName::from("style")),
            "background-color: rgb(0, 0, 255)",
        );
    });
    let repainted = document.frame(64, 64).expect("retained repaint");
    assert_eq!(document.layout_generation(), layout_generation);
    let repainted_scene = translate_paint_list(&repainted).expect("repainted paint translation");
    let repainted_pixels = cpu.render(&repainted_scene).expect("repainted CPU raster");
    assert_eq!(
        pixel_at(repainted_pixels.data_as_u8_slice(), 64, 16, 16),
        [0, 0, 255, 255]
    );
    assert_eq!(document.hit_test(16.0, 16.0), Some(target));
}

fn scroll_document(canvas_style: &str) -> LiveryDocument<ScriptedDom> {
    let mut dom = ScriptedDom::from_serialized_document(
        "<html><body><div style='height:32px'></div>\
         <div id=target></div><div style='width:180px;height:180px'></div></body></html>",
    );
    dom.drain_mutations(&mut Vec::new());
    LiveryDocument::new(
        dom,
        StyleSet::cambium(&[&format!(
            "html,body {{ margin:0; padding:0; }} {canvas_style} \
             #target {{ margin-left:24px; width:16px; height:16px; background:red; }}"
        )]),
        Device::screen(64.0, 64.0),
    )
}

#[test]
fn canvas_color_covers_the_viewport_after_cached_scroll_and_retained_repaint() {
    for selector in ["html", "body"] {
        for zoom in [1.0_f32, 1.1] {
            let mut document = scroll_document(&format!("{selector} {{ background:#f4efe3; }}"));
            let mut cpu = CpuSession::new(SparseResourceLimits::default());
            document.frame(64, 64).unwrap();
            let generation = document.layout_generation();
            assert!(document.scroll_by(16.0, 24.0));
            assert_eq!(document.scroll(), (16.0, 24.0));
            let width = (64.0 * zoom).round() as u32;
            for repaint in [false, true] {
                if repaint {
                    let target = target(document.dom());
                    document.mutate_dom(|dom| {
                        dom.set_attribute(
                            target,
                            QualName::new(None, Namespace::from(""), LocalName::from("style")),
                            "background:blue",
                        );
                    });
                }
                let list = document
                    .frame(64, 64)
                    .unwrap()
                    .scaled_to(zoom, width, width);
                let scene = translate_paint_list(&list).unwrap();
                let pixels = cpu.render(&scene).unwrap();
                let bytes = pixels.data_as_u8_slice();
                let sample = (60.0 * zoom) as usize;
                assert_eq!(
                    pixel_at(bytes, width as usize, sample, sample),
                    [244, 239, 227, 255],
                    "{selector} canvas must cover the lower viewport: zoom={zoom}, repaint={repaint}"
                );
                let marker = (12.0 * zoom) as usize;
                assert_eq!(
                    pixel_at(bytes, width as usize, marker, marker),
                    if repaint {
                        [0, 0, 255, 255]
                    } else {
                        [255, 0, 0, 255]
                    },
                    "document content must still move with scroll"
                );
                assert_eq!(document.layout_generation(), generation);
            }
        }
    }
}

#[test]
fn placing_a_scrolled_surface_moves_its_canvas_with_its_content() {
    let mut document = scroll_document("body { background:#f4efe3; }");
    document.frame(64, 64).unwrap();
    assert!(document.scroll_by(16.0, 24.0));
    let list = document.frame(64, 64).unwrap().translated(8.0, 8.0);
    let scene = translate_paint_list(&list).unwrap();
    let mut cpu = CpuSession::new(SparseResourceLimits::default());
    let pixels = cpu.render(&scene).unwrap();
    assert_eq!(pixel_at(pixels.data_as_u8_slice(), 64, 4, 4), [0, 0, 0, 0]);
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 12, 12),
        [244, 239, 227, 255]
    );
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 20, 20),
        [255, 0, 0, 255]
    );
}

#[test]
fn scrolling_a_transparent_canvas_does_not_pin_the_first_element_background() {
    let mut document = scroll_document("html,body { background:transparent; }");
    document.frame(64, 64).unwrap();
    assert!(document.scroll_by(16.0, 24.0));
    let scene = translate_paint_list(&document.frame(64, 64).unwrap()).unwrap();
    let mut cpu = CpuSession::new(SparseResourceLimits::default());
    let pixels = cpu.render(&scene).unwrap();
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 12, 12),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 60, 60),
        [0, 0, 0, 0]
    );
}

#[test]
fn a_spliced_child_stays_inside_document_scroll_below_the_canvas_color() {
    use paint_list_api::{ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, RectItem};
    let mut document = LiveryDocument::new(
        ScriptedDom::from_serialized_document(
            "<html><body><div style='height:32px'></div><iframe></iframe>\
             <div style='width:180px;height:180px'></div></body></html>",
        ),
        StyleSet::cambium(&[
            "html,body { margin:0; padding:0; } body { background:#f4efe3; } \
             iframe { display:block; margin-left:24px; width:16px; height:16px; border:0; padding:0; }",
        ]),
        Device::screen(64.0, 64.0),
    );
    document.frame(64, 64).unwrap();
    assert!(document.scroll_by(16.0, 24.0));
    let mut list = document.frame(64, 64).unwrap();
    assert_eq!(list.frame_slots().len(), 1);
    list.splice_frame_slots(|_| {
        Some(vec![PaintCmd::DrawRect(RectItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(16.0, 16.0),
            )),
            color: ColorF::new(1.0, 0.0, 0.0, 1.0),
        })])
    });
    let scene = translate_paint_list(&list).unwrap();
    let mut cpu = CpuSession::new(SparseResourceLimits::default());
    let pixels = cpu.render(&scene).unwrap();
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 12, 12),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(pixels.data_as_u8_slice(), 64, 60, 60),
        [244, 239, 227, 255]
    );
}
