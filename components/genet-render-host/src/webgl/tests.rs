// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::RenderCore;
use netrender::NetrenderOptions;

fn core_and_host() -> Option<(RenderCore, WebGlHost)> {
    let core = match RenderCore::boot(NetrenderOptions {
        tile_cache_size: Some(4),
        enable_vello: true,
        ..Default::default()
    }) {
        Ok(core) => core,
        Err(error) => {
            eprintln!("skipping native WebGL GPU test: {error}");
            return None;
        },
    };
    let host = WebGlHost::new(core.device().clone(), core.queue().clone());
    Some((core, host))
}

fn host() -> Option<WebGlHost> {
    core_and_host().map(|(_, host)| host)
}

#[test]
fn registry_tracks_two_contexts_across_resize_and_drop() {
    let Some(host) = host() else { return };
    let mut first = host.make_handler(4, 4).expect("first context");
    let second = host.make_handler(8, 8).expect("second context");
    let first_key = first.external_texture_key().expect("first key");
    let second_key = second.external_texture_key().expect("second key");
    assert_ne!(first_key, second_key);
    assert_eq!(host.live_contexts(), 2);

    let before = {
        let context = first.context.lock().expect("WebGL context lock");
        (context.texture().size, context.texture().generation)
    };
    first.resize(6, 5);
    let after = {
        let context = first.context.lock().expect("WebGL context lock");
        (context.texture().size, context.texture().generation)
    };
    assert_eq!(first.external_texture_key(), Some(first_key));
    assert_eq!(before.0, (4, 4));
    assert_eq!(after.0, (6, 5));
    assert!(after.1 > before.1);
    drop(second);
    assert_eq!(host.live_contexts(), 1);
    drop(first);
    assert_eq!(host.live_contexts(), 0);
    let third = host.make_handler(2, 2).expect("third context");
    assert_ne!(third.external_texture_key(), Some(first_key));
    drop(third);
}

#[test]
fn document_factory_registers_and_drops_its_context() {
    let Some(host) = host() else { return };
    let handler = host.create_handler(3, 2).expect("document WebGL context");
    assert_eq!(host.live_contexts(), 1);
    drop(handler);
    assert_eq!(host.live_contexts(), 0);
}

#[test]
fn document_factory_rejects_oversized_context_without_registering_it() {
    let Some(host) = host() else { return };
    let too_wide = host
        .device
        .limits()
        .max_texture_dimension_2d
        .checked_add(1)
        .expect("finite texture dimension limit");
    assert!(host.create_handler(too_wide, 1).is_none());
    assert_eq!(host.live_contexts(), 0);
    assert_eq!(host.next_key.load(Ordering::Relaxed), 1);
    let handler = host
        .create_handler(1, 1)
        .expect("valid context after rejected request");
    assert_eq!(handler.external_texture_key(), Some(1));
    drop(handler);
    assert_eq!(host.live_contexts(), 0);
}

#[test]
fn sync_stages_only_trusted_contexts_refreshes_and_retires() {
    let Some((core, host)) = core_and_host() else {
        return;
    };
    let mut context = host.make_handler(4, 3).expect("staged context");
    let key = context.external_texture_key().expect("context key");
    let image_key = netrender::external_image_key(key);
    let mut images = WebGlImages::default();
    let draws = [key, 999];
    images.sync(&host, core.renderer(), draws).unwrap();
    assert_eq!(images.staged, HashSet::from([image_key]));

    context.clear_color(0.0, 1.0, 0.0, 1.0);
    context.clear(GL_COLOR_BUFFER_BIT);
    context.resize(6, 5);
    images.sync(&host, core.renderer(), draws).unwrap();
    assert_eq!(images.staged, HashSet::from([image_key]));

    images.sync(&host, core.renderer(), []).unwrap();
    assert!(images.staged.is_empty());
    assert!(
        !core.renderer().unregister_external_image(image_key),
        "the empty frame already retired the source"
    );
}

#[test]
fn placements_retire_their_own_staged_images() {
    let Some((core, host)) = core_and_host() else {
        return;
    };
    let second = netrender::create_netrender_instance(
        core.renderer().wgpu_device.core.clone(),
        netrender::NetrenderOptions {
            tile_cache_size: Some(4),
            enable_vello: true,
            ..Default::default()
        },
    )
    .expect("renderer on the same device");
    let canvas = host.make_handler(4, 3).unwrap();
    let key = canvas.external_texture_key().unwrap();
    let image = netrender::external_image_key(key);
    let mut first_images = WebGlImages::default();
    let mut second_images = WebGlImages::default();
    first_images.sync(&host, core.renderer(), [key]).unwrap();
    second_images.sync(&host, &second, [key]).unwrap();

    first_images.sync(&host, core.renderer(), []).unwrap();
    assert!(!core.renderer().unregister_external_image(image));
    assert!(second.unregister_external_image(image));
}

#[test]
fn production_handler_draws_a_triangle() {
    let Some(host) = host() else { return };
    let mut gl = host.make_handler(32, 32).expect("draw context");
    gl.viewport(0, 0, 32, 32);
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL_COLOR_BUFFER_BIT);

    let vertex = gl.create_shader(GL_VERTEX_SHADER);
    gl.shader_source(
        vertex,
        "attribute vec2 a_position; void main() { gl_Position = vec4(a_position, 0.0, 1.0); }",
    );
    gl.compile_shader(vertex);
    assert!(gl.get_shader_compile_status(vertex));
    let fragment = gl.create_shader(GL_FRAGMENT_SHADER);
    gl.shader_source(
        fragment,
        "precision mediump float; uniform vec4 u_color; void main() { gl_FragColor = u_color; }",
    );
    gl.compile_shader(fragment);
    assert!(gl.get_shader_compile_status(fragment));
    let program = gl.create_program();
    gl.attach_shader(program, vertex);
    gl.attach_shader(program, fragment);
    gl.link_program(program);
    assert!(gl.get_program_link_status(program));
    gl.use_program(Some(program));
    let color = gl.get_uniform_location(program, "u_color");
    assert!(color >= 0);
    gl.uniform4f(color, 0.0, 1.0, 0.0, 1.0);
    let position = gl.get_attrib_location(program, "a_position");
    assert_eq!(position, 0);
    let buffer = gl.create_buffer();
    gl.bind_buffer(GL_ARRAY_BUFFER, Some(buffer));
    gl.buffer_data_f32(GL_ARRAY_BUFFER, &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8], 0x88E4);
    gl.enable_vertex_attrib_array(position as u32);
    gl.vertex_attrib_pointer_f32(position as u32, 2, false, 0, 0);
    gl.draw_arrays(GL_TRIANGLES, 0, 3);
    assert_eq!(gl.get_error(), GL_NO_ERROR);
    assert_eq!(gl.read_pixels_rgba8(16, 16, 1, 1), vec![0, 255, 0, 255]);
}
