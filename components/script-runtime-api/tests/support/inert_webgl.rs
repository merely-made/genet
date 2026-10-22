// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A WebGL context that draws nothing. Adoption tests need a canvas that has
//! minted a live context, not a GPU; `getContext('webgl')` is null without one.

use script_runtime_api::{WebGlFactory, WebGlHandler};

pub fn factory() -> WebGlFactory {
    Box::new(|_, _| Some(Box::new(InertWebGl::default())))
}

#[derive(Default)]
struct InertWebGl {
    next: u64,
}

impl InertWebGl {
    fn mint(&mut self) -> u64 {
        self.next += 1;
        self.next
    }
}

impl WebGlHandler for InertWebGl {
    fn clear_color(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn clear(&mut self, _: u32) {}
    fn viewport(&mut self, _: i32, _: i32, _: u32, _: u32) {}
    fn enable(&mut self, _: u32) {}
    fn disable(&mut self, _: u32) {}
    fn is_enabled(&mut self, _: u32) -> bool { false }
    fn color_mask(&mut self, _: bool, _: bool, _: bool, _: bool) {}
    fn create_buffer(&mut self) -> u64 { self.mint() }
    fn bind_buffer(&mut self, _: u32, _: Option<u64>) {}
    fn buffer_data_f32(&mut self, _: u32, _: &[f32], _: u32) {}
    fn create_shader(&mut self, _: u32) -> u64 { self.mint() }
    fn shader_source(&mut self, _: u64, _: &str) {}
    fn compile_shader(&mut self, _: u64) {}
    fn get_shader_compile_status(&mut self, _: u64) -> bool { false }
    fn get_shader_info_log(&mut self, _: u64) -> String { String::new() }
    fn create_program(&mut self) -> u64 { self.mint() }
    fn attach_shader(&mut self, _: u64, _: u64) {}
    fn link_program(&mut self, _: u64) {}
    fn get_program_link_status(&mut self, _: u64) -> bool { false }
    fn get_program_info_log(&mut self, _: u64) -> String { String::new() }
    fn use_program(&mut self, _: Option<u64>) {}
    fn get_attrib_location(&mut self, _: u64, _: &str) -> i32 { -1 }
    fn get_uniform_location(&mut self, _: u64, _: &str) -> i32 { -1 }
    fn enable_vertex_attrib_array(&mut self, _: u32) {}
    fn vertex_attrib_pointer_f32(&mut self, _: u32, _: u32, _: bool, _: u32, _: u32) {}
    fn uniform4f(&mut self, _: i32, _: f32, _: f32, _: f32, _: f32) {}
    fn uniform_matrix4fv(&mut self, _: i32, _: bool, _: &[f32]) {}
    fn uniform1i(&mut self, _: i32, _: i32) {}
    fn create_texture(&mut self) -> u64 { self.mint() }
    fn bind_texture_2d(&mut self, _: Option<u64>) {}
    fn active_texture(&mut self, _: u32) {}
    fn tex_image_2d_rgba8(&mut self, _: u32, _: u32, _: &[u8]) {}
    fn draw_arrays(&mut self, _: u32, _: i32, _: i32) {}
    fn get_error(&mut self) -> u32 { 0 }
    fn read_pixels_rgba8(&mut self, _: i32, _: i32, w: u32, h: u32) -> Vec<u8> {
        vec![0; (w * h * 4) as usize]
    }
}
