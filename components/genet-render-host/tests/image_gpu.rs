/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IMG-GPU: decoded document images drawn on the shared RenderCore device
//! and read back. Opaque, translucent, replacement, removal and child-frame
//! composition, each against computed expected pixels and a backdrop control.
//!
//! Set `GENET_IMG_GPU_EVIDENCE=<dir>` to write each readback as a PNG.

use genet_livery::{Device, LiveryDocument, LiveryPaintList, StyleSet};
use genet_render_host::{RenderCore, RgbaFrame};
use genet_scripted_dom::ScriptedDom;
use netrender::{ColorLoad, NetrenderOptions};
use paint_list_api::PaintList;
use std::sync::Arc;

const DIM: u32 = 64;
const WHITE: [u8; 4] = [255, 255, 255, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const MAGENTA: [u8; 4] = [255, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];
const GREEN: [u8; 4] = [0, 128, 0, 255];
const TRANSLUCENT: [u8; 4] = [0, 255, 0, 128];
// src-over of TRANSLUCENT on RED: 255 * (1 - 128/255) = 127, 255 * 128/255 = 128.
const TRANSLUCENT_OVER_RED: [u8; 4] = [127, 128, 0, 255];
// Supplier crates whose resolved revisions must agree in the consumer lock.
const SUPPLIER: [&str; 4] = ["netrender", "netrender_device", "paint_list_api", "paint_list_render"];

fn png(size: u32, color: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::RgbaImage::from_pixel(size, size, image::Rgba(color))
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("encode fixture");
    bytes
}

fn document(html: &str, css: &str) -> LiveryDocument<ScriptedDom> {
    LiveryDocument::new(
        ScriptedDom::from_serialized_document(html),
        StyleSet::cambium(&[css]),
        Device::screen(DIM as f32, DIM as f32),
    )
}

/// The one revision every supplier crate resolves to in the workspace lock.
fn consumer_netrender_rev() -> String {
    let lock_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock");
    let lock = std::fs::read_to_string(lock_path).expect("workspace Cargo.lock");
    let mut revs = Vec::new();
    for block in lock.split("[[package]]") {
        let field = |key: &str| {
            block
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix(" = \"")?.strip_suffix('"'))
        };
        let Some(name) = field("name") else { continue };
        if !SUPPLIER.contains(&name) {
            continue;
        }
        let source = field("source").unwrap_or_else(|| panic!("{name} has no lock source"));
        assert!(
            source.starts_with("git+https://github.com/merely-made/netrender.git"),
            "{name} resolves outside the published supplier: {source}"
        );
        let rev = source.rsplit_once('#').expect("git source carries a rev").1;
        revs.push((name.to_string(), rev.to_string()));
    }
    assert_eq!(revs.len(), SUPPLIER.len(), "every supplier crate is locked once: {revs:?}");
    assert!(revs.iter().all(|(_, rev)| *rev == revs[0].1), "one supplier revision: {revs:?}");
    revs.swap_remove(0).1
}

struct Gate {
    core: RenderCore,
}

impl Gate {
    /// Translate, rasterize on the shared device, read back. Also witnesses
    /// that the scene's image blobs are the document's decoded allocations.
    fn draw(&self, surface: u64, list: &LiveryPaintList, case: &str) -> RgbaFrame {
        let scene = paint_list_render::translate_paint_list(list).expect("translate paint list");
        for (key, image) in &scene.image_sources {
            eprintln!("IMG-GPU {case}: scene image key {key} {}x{} blob id {}", image.width, image.height, image.data.id());
        }
        for resource in list.images() {
            assert!(
                scene.image_sources.values().any(|image| {
                    AsRef::<[u8]>::as_ref(&image.data).as_ptr() == resource.data.as_ptr()
                }),
                "{case}: scene image must share the decoded pixel allocation"
            );
        }
        let (texture, _view) = self
            .core
            .try_rasterize_scaled_for(
                surface,
                &scene,
                DIM,
                DIM,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap_or_else(|error| panic!("{case}: checked raster refused: {error}"));
        let frame = self
            .core
            .read_rgba8_texture(&texture, DIM, DIM)
            .unwrap_or_else(|error| panic!("{case}: readback failed: {error}"));
        assert!(!frame.is_blank(), "{case}: readback is blank");
        if let Some(dir) = std::env::var_os("GENET_IMG_GPU_EVIDENCE") {
            let path = std::path::Path::new(&dir).join(format!("{case}.png"));
            image::RgbaImage::from_raw(DIM, DIM, frame.rgba.clone())
                .expect("frame dimensions")
                .save(&path)
                .expect("write evidence PNG");
        }
        eprintln!("IMG-GPU {case}: digest {:016x}", frame.digest());
        frame
    }
}

fn pixel(frame: &RgbaFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * frame.width + x) * 4) as usize;
    frame.rgba[i..i + 4].try_into().unwrap()
}

/// Mismatches are collected so one run reports the whole matrix.
static MISMATCHES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn expect(frame: &RgbaFrame, case: &str, (x, y): (u32, u32), want: [u8; 4], tolerance: u8) {
    let got = pixel(frame, x, y);
    let ok = got.iter().zip(want).all(|(g, w)| g.abs_diff(w) <= tolerance);
    eprintln!(
        "IMG-GPU {} {case} at ({x}, {y}): got {got:?}, want {want:?} +-{tolerance}",
        if ok { "PASS" } else { "FAIL" }
    );
    if !ok {
        MISMATCHES.lock().unwrap().push(format!("{case} at ({x}, {y}): got {got:?}, want {want:?}"));
    }
}

#[test]
#[ignore = "actual shared-device image draw/readback; coordinator executes serialized GPU gate"]
fn img_gpu_decoded_images_draw_and_read_back_on_the_shared_device() {
    let rev = consumer_netrender_rev();
    let core = RenderCore::boot(NetrenderOptions {
        tile_cache_size: Some(32),
        enable_vello: true,
        ..Default::default()
    })
    .expect("an actual adapter is required for this ignored gate");
    let adapter = core.renderer().wgpu_device.core.adapter.get_info();
    eprintln!(
        "IMG-GPU netrender {rev}; adapter {} ({:?}, {:?}, driver {})",
        adapter.name, adapter.backend, adapter.device_type, adapter.driver_info
    );
    let gate = Gate { core };

    // Opaque, translucent, replacement and removal share one retained document
    // and one surface key, so the renderer's retained state is reused.
    const SUBJECT: u64 = 0x494d_472d_4750_5531;
    let mut subject = document(
        "<html><body><div id=backdrop></div><img id=opaque src='opaque.png'>\
         <img id=translucent src='translucent.png'></body></html>",
        "html { background: white; } body { margin: 0; } \
         #backdrop { position: absolute; left: 32px; top: 8px; width: 24px; height: 24px; \
                     background: rgb(255, 0, 0); } \
         #opaque { position: absolute; left: 8px; top: 8px; } \
         #translucent { position: absolute; left: 36px; top: 12px; }",
    );
    subject.set_image_resource("opaque.png", png(16, BLUE));
    subject.set_image_resource("translucent.png", png(16, TRANSLUCENT));

    let first = subject.frame(DIM, DIM).expect("first frame");
    assert_eq!(first.images().len(), 2);
    let frame = gate.draw(SUBJECT, &first, "1-opaque-translucent");
    expect(&frame, "opaque", (16, 16), BLUE, 0);
    expect(&frame, "opaque control", (16, 40), WHITE, 0);
    expect(&frame, "translucent", (44, 20), TRANSLUCENT_OVER_RED, 1);
    expect(&frame, "translucent control", (34, 10), RED, 0);
    let translucent_pixels = first
        .images()
        .iter()
        .find(|image| image.width == 16 && image.data.starts_with(&TRANSLUCENT))
        .expect("translucent resource")
        .data
        .clone();

    // Replacement: same URL, new pixels and a smaller natural size.
    subject.replace_image_resources([
        ("opaque.png".into(), png(8, MAGENTA)),
        ("translucent.png".into(), png(16, TRANSLUCENT)),
    ]);
    let replaced = subject.frame(DIM, DIM).expect("replacement frame");
    assert!(
        replaced.images().iter().any(|image| Arc::ptr_eq(&image.data, &translucent_pixels)),
        "equal-byte replacement keeps the decoded allocation"
    );
    let resource = replaced.images().iter().find(|image| image.width == 8).expect("8x8 replacement resource");
    assert!(resource.data.starts_with(&MAGENTA), "paint list carries the replacement pixels");
    let frame = gate.draw(SUBJECT, &replaced, "2-replacement");
    let probe = gate.draw(SUBJECT + 100, &replaced, "2b-replacement-fresh-surface-probe");
    eprintln!("IMG-GPU probe: same scene, fresh surface key, pixel (12,12) = {:?}; subject surface = {:?}", pixel(&probe, 12, 12), pixel(&frame, 12, 12));
    expect(&frame, "replacement", (12, 12), MAGENTA, 0);
    expect(&frame, "replacement old area", (20, 20), WHITE, 0);
    expect(&frame, "replacement keeps translucent", (44, 20), TRANSLUCENT_OVER_RED, 1);

    // Removal: no resource, no stale raster.
    subject.replace_image_resources(std::iter::empty());
    let removed = subject.frame(DIM, DIM).expect("removal frame");
    assert!(removed.images().is_empty());
    let frame = gate.draw(SUBJECT, &removed, "3-removal");
    expect(&frame, "removed opaque", (12, 12), WHITE, 0);
    expect(&frame, "removed translucent", (44, 20), RED, 0);

    // Frame composition: parent and child both own image key 1; the child is
    // spliced over parent content the way genet-documents composites frames.
    const COMPOSED: u64 = 0x494d_472d_4750_5532;
    let mut parent = document(
        "<html><body><img src='parent.png'><iframe></iframe></body></html>",
        "html { background: white; } body { margin: 0; } \
         img { position: absolute; left: 0; top: 0; } \
         iframe { position: absolute; left: 32px; top: 32px; width: 24px; height: 24px; \
                  border: 0; display: block; }",
    );
    parent.set_image_resource("parent.png", png(16, YELLOW));
    let mut list = parent.frame(DIM, DIM).expect("parent frame");
    let slot = list.frame_slots()[0].clone();
    let (width, height) = (
        slot.content_rect.width().round() as u32,
        slot.content_rect.height().round() as u32,
    );
    assert_eq!((width, height), (24, 24));
    let mut child = LiveryDocument::new(
        ScriptedDom::from_serialized_document(
            "<html><body><div></div><img src='child.png'></body></html>",
        ),
        StyleSet::cambium(&[
            "body { margin: 0; } div { width: 24px; height: 24px; background: rgb(0, 128, 0); } \
             img { position: absolute; left: 0; top: 0; }",
        ]),
        Device::screen(width as f32, height as f32),
    );
    child.set_image_resource("child.png", png(8, BLUE));
    let child_list = child.frame(width, height).expect("child frame");
    assert_eq!(child_list.images()[0].key, list.images()[0].key, "keys collide before absorb");
    let commands = list.absorb_frame_commands(&child_list);
    assert!(Arc::ptr_eq(&child_list.images()[0].data, &list.images()[1].data));
    list.splice_frame_slots(|owner| (owner == slot.owner_node).then(|| commands.clone()));
    let frame = gate.draw(COMPOSED, &list, "4-frame-composition");
    expect(&frame, "parent image", (8, 8), YELLOW, 0);
    expect(&frame, "child image", (36, 36), BLUE, 0);
    expect(&frame, "child backdrop", (50, 50), GREEN, 0);
    expect(&frame, "parent around frame", (60, 60), WHITE, 0);
    expect(&frame, "parent before frame", (24, 24), WHITE, 0);

    assert!(gate.core.device_failure().is_none());
    let mismatches = MISMATCHES.lock().unwrap();
    assert!(mismatches.is_empty(), "IMG-GPU mismatches: {mismatches:#?}");
}
