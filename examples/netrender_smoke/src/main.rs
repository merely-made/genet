/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Standalone embedder smoke using the current portable Scene renderer.
//!
//! Boots a real adapter, renders one opaque red rectangle into an offscreen
//! target, and checks device completion and validation. This is a renderer
//! submission receipt; it does not establish pixel or host-presentation parity.
//! The caller must also impose a process timeout around adapter/driver execution.

use netrender::{ColorLoad, NetrenderOptions, Scene, boot, create_netrender_instance};
use std::time::Duration;

const DIM: u32 = 256;
const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handles = boot()?;
    let info = handles.adapter.get_info();
    let device = handles.device.clone();
    println!("adapter: {} ({:?})", info.name, info.backend);

    let validation_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let renderer = create_netrender_instance(
        handles,
        NetrenderOptions {
            tile_cache_size: Some(DIM),
            enable_vello: true,
            ..NetrenderOptions::default()
        },
    )
    .map_err(|error| format!("renderer initialization failed: {error:?}"))?;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("netrender smoke target"),
        size: wgpu::Extent3d {
            width: DIM,
            height: DIM,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut scene = Scene::new(DIM, DIM);
    scene.push_rect(0.0, 0.0, DIM as f32, DIM as f32, [1.0, 0.0, 0.0, 1.0]);
    renderer.render_vello(&scene, &view, ColorLoad::Clear(wgpu::Color::TRANSPARENT));

    let validation = validation_scope.pop();
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(5)),
    })?;
    if let Some(error) = pollster::block_on(validation) {
        return Err(format!("render validation failed: {error}").into());
    }

    let log = format!(
        "Netrender Scene embedder smoke: PASS\n\
         adapter: {} ({:?})\n\
         target: {DIM}x{DIM} {TARGET_FORMAT:?}\n\
         scene: one opaque red viewport rectangle\n\
         result: renderer submission completed without validation error\n",
        info.name, info.backend,
    );
    let log_path = std::env::var_os("NETRENDER_SMOKE_LOG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("netrender-smoke.log"));
    std::fs::write(&log_path, &log)?;
    print!("{log}");
    println!("log written to {}", log_path.display());
    Ok(())
}
