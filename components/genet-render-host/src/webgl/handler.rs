// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use script_runtime_api::WebGlHandler;
use webgl_wgpu::{
    BufferTarget, BufferUsage, PrimitiveMode, ShaderStage, WebGlBufferId, WebGlError,
    WebGlProgramId, WebGlShaderId, WebGlTextureId, WebGlUniformLocation,
};

use super::{
    Context, GL_ARRAY_BUFFER, GL_COLOR_BUFFER_BIT, GL_CONTEXT_LOST_WEBGL, GL_DITHER,
    GL_ELEMENT_ARRAY_BUFFER, GL_FRAGMENT_SHADER, GL_INVALID_ENUM, GL_INVALID_FRAMEBUFFER_OPERATION,
    GL_INVALID_OPERATION, GL_INVALID_VALUE, GL_NO_ERROR, GL_SCISSOR_TEST, GL_TRIANGLES,
    GL_VERTEX_SHADER, Registry,
};

pub(super) struct NativeWebGl {
    pub(super) context: Context,
    external_key: u64,
    registry: Registry,
    content_generation: Arc<AtomicU64>,
    /// Next id to hand out across the JS seam for `create_*`. Distinct from
    /// the WebGl*Id namespaces because webgl-wgpu's ids start at 1 per
    /// resource kind; we keep one shared counter to avoid id-collisions
    /// across kinds on the JS side.
    next_id: RefCell<u64>,
    buffers: RefCell<HashMap<u64, WebGlBufferId>>,
    shaders: RefCell<HashMap<u64, WebGlShaderId>>,
    programs: RefCell<HashMap<u64, WebGlProgramId>>,
    textures: RefCell<HashMap<u64, WebGlTextureId>>,
    /// `i32` index into this list is the opaque uniform-location id the JS
    /// wrapper holds onto. `getUniformLocation` returns the index; setters
    /// look it up by index. `-1` is reserved for "not found", per WebGL.
    uniform_locations: RefCell<Vec<WebGlUniformLocation>>,
    /// Current `gl.clearColor` state. webgl-wgpu's `clear` takes the color
    /// directly each call (no bound-color state), so we hold it here and
    /// pass it through on `clear`. WebGL's default is transparent black.
    clear_color: RefCell<[f32; 4]>,
    /// `gl.enable` / `gl.disable` state, keyed by GLenum cap. Source of
    /// truth for `isEnabled`. Only `SCISSOR_TEST` currently forwards to the
    /// backend (webgl-wgpu's scissor toggle); the rest are recorded so the
    /// round-trip is correct even though the draw effect isn't wired yet
    /// (BLEND / CULL_FACE aren't exposed; DEPTH_TEST needs a depth canvas).
    enabled_caps: RefCell<std::collections::HashSet<u32>>,
    /// Errors discovered while decoding a raw WebGL enum before webgl-wgpu
    /// receives a method call.
    pending_error: RefCell<Option<WebGlError>>,
}

impl NativeWebGl {
    pub(super) fn new(
        context: Context,
        external_key: u64,
        registry: Registry,
        content_generation: Arc<AtomicU64>,
    ) -> Self {
        // WebGL's only default-enabled capability is DITHER.
        let mut caps = std::collections::HashSet::new();
        caps.insert(GL_DITHER);
        Self {
            context,
            external_key,
            registry,
            content_generation,
            next_id: RefCell::new(1),
            buffers: RefCell::new(HashMap::new()),
            shaders: RefCell::new(HashMap::new()),
            programs: RefCell::new(HashMap::new()),
            textures: RefCell::new(HashMap::new()),
            uniform_locations: RefCell::new(Vec::new()),
            clear_color: RefCell::new([0.0, 0.0, 0.0, 0.0]),
            enabled_caps: RefCell::new(caps),
            pending_error: RefCell::new(None),
        }
    }

    fn alloc_id(&self) -> u64 {
        let mut n = self.next_id.borrow_mut();
        let id = *n;
        *n += 1;
        id
    }

    fn buffer_target(target: u32) -> Option<BufferTarget> {
        match target {
            GL_ARRAY_BUFFER => Some(BufferTarget::ArrayBuffer),
            GL_ELEMENT_ARRAY_BUFFER => Some(BufferTarget::ElementArrayBuffer),
            _ => None,
        }
    }

    fn record_error(&self, error: WebGlError) {
        let mut pending = self.pending_error.borrow_mut();
        if pending.is_none() {
            *pending = Some(error);
        }
    }

    fn note_content_change(&self) {
        self.content_generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl Drop for NativeWebGl {
    fn drop(&mut self) {
        self.registry
            .lock()
            .expect("WebGL registry lock")
            .remove(&self.external_key);
    }
}

impl WebGlHandler for NativeWebGl {
    fn external_texture_key(&self) -> Option<u64> {
        Some(self.external_key)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.context
            .lock()
            .expect("WebGL context lock")
            .resize(width.max(1), height.max(1))
            .expect("WebGL drawing buffer resizes");
        self.note_content_change();
    }

    fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        // webgl-wgpu's clear takes the color directly each call (no
        // bound-color state), so hold it and pass it through on `clear`.
        *self.clear_color.borrow_mut() = [r, g, b, a];
    }

    fn clear(&mut self, mask: u32) {
        if mask & GL_COLOR_BUFFER_BIT != 0 {
            let c = *self.clear_color.borrow();
            self.context
                .lock()
                .expect("webgl context lock")
                .clear(wgpu::Color {
                    r: c[0] as f64,
                    g: c[1] as f64,
                    b: c[2] as f64,
                    a: c[3] as f64,
                });
            self.note_content_change();
        }
    }

    fn enable(&mut self, cap: u32) {
        self.enabled_caps.borrow_mut().insert(cap);
        if cap == GL_SCISSOR_TEST {
            self.context
                .lock()
                .expect("webgl context lock")
                .set_scissor_test_enabled(true);
        }
        // DEPTH_TEST / BLEND / CULL_FACE recorded only â€” see struct doc.
    }

    fn disable(&mut self, cap: u32) {
        self.enabled_caps.borrow_mut().remove(&cap);
        if cap == GL_SCISSOR_TEST {
            self.context
                .lock()
                .expect("webgl context lock")
                .set_scissor_test_enabled(false);
        }
    }

    fn is_enabled(&mut self, cap: u32) -> bool {
        self.enabled_caps.borrow().contains(&cap)
    }

    fn color_mask(&mut self, r: bool, g: bool, b: bool, a: bool) {
        self.context
            .lock()
            .expect("webgl context lock")
            .set_color_mask(r, g, b, a);
    }

    fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        if x < 0 || y < 0 {
            self.record_error(WebGlError::InvalidValue);
            return;
        }
        self.context
            .lock()
            .expect("WebGL context lock")
            .viewport(x as u32, y as u32, width, height);
    }

    fn create_buffer(&mut self) -> u64 {
        let id = self.alloc_id();
        let buffer = self
            .context
            .lock()
            .expect("webgl context lock")
            .create_buffer();
        self.buffers.borrow_mut().insert(id, buffer);
        id
    }
    fn bind_buffer(&mut self, target: u32, buffer: Option<u64>) {
        let Some(target) = Self::buffer_target(target) else {
            self.record_error(WebGlError::InvalidEnum);
            return;
        };
        let resolved = buffer.and_then(|id| self.buffers.borrow().get(&id).copied());
        self.context
            .lock()
            .expect("webgl context lock")
            .bind_buffer(target, resolved);
    }
    fn buffer_data_f32(&mut self, target: u32, data: &[f32], _usage: u32) {
        let Some(target) = Self::buffer_target(target) else {
            self.record_error(WebGlError::InvalidEnum);
            return;
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .buffer_data_f32(target, data, BufferUsage::StaticDraw);
    }

    fn create_shader(&mut self, stage: u32) -> u64 {
        let stage = match stage {
            GL_VERTEX_SHADER => ShaderStage::Vertex,
            GL_FRAGMENT_SHADER => ShaderStage::Fragment,
            _ => {
                self.record_error(WebGlError::InvalidEnum);
                return 0;
            },
        };
        let id = self.alloc_id();
        let shader = self
            .context
            .lock()
            .expect("webgl context lock")
            .create_shader(stage);
        self.shaders.borrow_mut().insert(id, shader);
        id
    }
    fn shader_source(&mut self, shader: u64, source: &str) {
        let Some(&shader_id) = self.shaders.borrow().get(&shader) else {
            return;
        };
        self.context
            .lock()
            .expect("webgl context lock")
            .shader_source(shader_id, source);
    }
    fn compile_shader(&mut self, shader: u64) {
        let Some(&shader_id) = self.shaders.borrow().get(&shader) else {
            return;
        };
        self.context
            .lock()
            .expect("webgl context lock")
            .compile_shader(shader_id);
    }
    fn get_shader_compile_status(&mut self, shader: u64) -> bool {
        let Some(&shader_id) = self.shaders.borrow().get(&shader) else {
            return false;
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .get_shader_compile_status(shader_id)
    }
    fn get_shader_info_log(&mut self, shader: u64) -> String {
        let Some(&shader_id) = self.shaders.borrow().get(&shader) else {
            return String::new();
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .get_shader_info_log(shader_id)
            .unwrap_or_default()
    }

    fn create_program(&mut self) -> u64 {
        let id = self.alloc_id();
        let program = self
            .context
            .lock()
            .expect("webgl context lock")
            .create_program();
        self.programs.borrow_mut().insert(id, program);
        id
    }
    fn attach_shader(&mut self, program: u64, shader: u64) {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return;
        };
        let Some(&shader_id) = self.shaders.borrow().get(&shader) else {
            return;
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .attach_shader(program_id, shader_id);
    }
    fn link_program(&mut self, program: u64) {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return;
        };
        self.context
            .lock()
            .expect("webgl context lock")
            .link_program(program_id);
    }
    fn get_program_link_status(&mut self, program: u64) -> bool {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return false;
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .get_program_link_status(program_id)
    }
    fn get_program_info_log(&mut self, program: u64) -> String {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return String::new();
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .get_program_info_log(program_id)
            .unwrap_or_default()
    }
    fn use_program(&mut self, program: Option<u64>) {
        let resolved = program.and_then(|id| self.programs.borrow().get(&id).copied());
        self.context
            .lock()
            .expect("webgl context lock")
            .use_program(resolved);
    }

    fn get_attrib_location(&mut self, program: u64, name: &str) -> i32 {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return -1;
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .get_attrib_location(program_id, name)
    }
    fn get_uniform_location(&mut self, program: u64, name: &str) -> i32 {
        let Some(&program_id) = self.programs.borrow().get(&program) else {
            return -1;
        };
        let Some(loc) = self
            .context
            .lock()
            .expect("WebGL context lock")
            .get_uniform_location(program_id, name)
        else {
            return -1;
        };
        let mut locs = self.uniform_locations.borrow_mut();
        let index = locs.len() as i32;
        locs.push(loc);
        index
    }

    fn enable_vertex_attrib_array(&mut self, index: u32) {
        self.context
            .lock()
            .expect("webgl context lock")
            .enable_vertex_attrib_array(index);
    }
    fn vertex_attrib_pointer_f32(
        &mut self,
        index: u32,
        size: u32,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        self.context
            .lock()
            .expect("webgl context lock")
            .vertex_attrib_pointer_f32(index, size, normalized, stride as u64, offset as u64);
    }

    fn uniform4f(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        if location < 0 {
            return;
        }
        let loc = match self.uniform_locations.borrow().get(location as usize) {
            Some(loc) => *loc,
            None => return,
        };
        self.context
            .lock()
            .expect("webgl context lock")
            .uniform4f(loc, x, y, z, w);
    }
    fn uniform_matrix4fv(&mut self, location: i32, _transpose: bool, value: &[f32]) {
        if location < 0 || value.len() < 16 {
            return;
        }
        let loc = match self.uniform_locations.borrow().get(location as usize) {
            Some(loc) => *loc,
            None => return,
        };
        let mut m = [0.0f32; 16];
        m.copy_from_slice(&value[..16]);
        self.context
            .lock()
            .expect("webgl context lock")
            .uniform_matrix4fv(loc, &m);
    }
    fn uniform1i(&mut self, location: i32, value: i32) {
        if location < 0 {
            return;
        }
        let loc = match self.uniform_locations.borrow().get(location as usize) {
            Some(loc) => *loc,
            None => return,
        };
        self.context
            .lock()
            .expect("webgl context lock")
            .uniform1i(loc, value);
    }

    fn create_texture(&mut self) -> u64 {
        let id = self.alloc_id();
        let texture = self
            .context
            .lock()
            .expect("webgl context lock")
            .create_texture();
        self.textures.borrow_mut().insert(id, texture);
        id
    }
    fn bind_texture_2d(&mut self, texture: Option<u64>) {
        let resolved = texture.and_then(|id| self.textures.borrow().get(&id).copied());
        self.context
            .lock()
            .expect("webgl context lock")
            .bind_texture_2d(resolved);
    }
    fn active_texture(&mut self, unit: u32) {
        self.context
            .lock()
            .expect("webgl context lock")
            .active_texture(unit);
    }
    fn tex_image_2d_rgba8(&mut self, width: u32, height: u32, pixels: &[u8]) {
        self.context
            .lock()
            .expect("WebGL context lock")
            .tex_image_2d_rgba8(width, height, pixels);
    }

    fn draw_arrays(&mut self, mode: u32, first: i32, count: i32) {
        let topology = match mode {
            GL_TRIANGLES => PrimitiveMode::Triangles,
            _ => {
                self.record_error(WebGlError::InvalidEnum);
                return;
            },
        };
        self.context
            .lock()
            .expect("WebGL context lock")
            .draw_arrays(topology, first as u32, count as u32);
        self.note_content_change();
    }

    fn get_error(&mut self) -> u32 {
        let error = self
            .pending_error
            .borrow_mut()
            .take()
            .unwrap_or_else(|| self.context.lock().expect("webgl context lock").get_error());
        match error {
            WebGlError::NoError => GL_NO_ERROR,
            WebGlError::InvalidEnum => GL_INVALID_ENUM,
            WebGlError::InvalidValue => GL_INVALID_VALUE,
            WebGlError::InvalidOperation => GL_INVALID_OPERATION,
            WebGlError::InvalidFramebufferOperation => GL_INVALID_FRAMEBUFFER_OPERATION,
            WebGlError::ContextLostWebgl => GL_CONTEXT_LOST_WEBGL,
        }
    }

    fn read_pixels_rgba8(&mut self, x: i32, y: i32, w: u32, h: u32) -> Vec<u8> {
        if x < 0 || y < 0 {
            self.record_error(WebGlError::InvalidValue);
            return Vec::new();
        }
        self.context
            .lock()
            .expect("WebGL context lock")
            .read_pixels(x as u32, y as u32, w, h)
            .unwrap_or_default()
    }
}
