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
    let first_scene = translate_paint_list(&first);
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
    let repainted_scene = translate_paint_list(&repainted);
    let repainted_pixels = cpu.render(&repainted_scene).expect("repainted CPU raster");
    assert_eq!(
        pixel_at(repainted_pixels.data_as_u8_slice(), 64, 16, 16),
        [0, 0, 255, 255]
    );
    assert_eq!(document.hit_test(16.0, 16.0), Some(target));
}
