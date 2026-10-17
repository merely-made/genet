/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::pipeline::f32_slice_to_bytes;
use super::*;
use crate::shader::{ShaderTranslationError, UniformReflection};

impl WebGlContext {
    pub fn create_shader(&mut self, stage: ShaderStage) -> WebGlShaderId {
        let id = WebGlShaderId(self.next_shader_id);
        self.next_shader_id += 1;
        self.shaders.insert(
            id,
            ShaderObject {
                stage,
                source: String::new(),
                source_revision: 0,
                compiled: None,
                compiled_revision: None,
                compile_status: false,
                info_log: String::new(),
            },
        );
        id
    }

    pub fn shader_source(&mut self, shader: WebGlShaderId, source: &str) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        let Some(shader) = self.shaders.get_mut(&shader) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        shader.source = source.to_string();
        shader.source_revision += 1;
        shader.compiled = None;
        shader.compiled_revision = None;
        shader.compile_status = false;
        shader.info_log.clear();
    }

    pub fn compile_shader(&mut self, shader: WebGlShaderId) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        let Some(shader) = self.shaders.get_mut(&shader) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        // Each invocation produces an artifact for this exact source revision.
        // A failed recompile must not leave the previous artifact linkable.
        shader.compiled = None;
        shader.compiled_revision = None;
        shader.compile_status = false;
        let stage = match shader.stage {
            ShaderStage::Vertex => webgl_essl::validate::ShaderStage::Vertex,
            ShaderStage::Fragment => webgl_essl::validate::ShaderStage::Fragment,
        };
        match compile_canonical_source(&shader.source, stage) {
            Ok(compiled) => {
                shader.compiled = Some(compiled);
                shader.compiled_revision = Some(shader.source_revision);
                shader.compile_status = true;
                shader.info_log.clear();
            },
            Err(error) => {
                shader.compile_status = false;
                shader.info_log = format!("{:?} shader compile failed: {error}", shader.stage);
            },
        }
    }

    pub fn get_shader_compile_status(&mut self, shader: WebGlShaderId) -> bool {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return false;
        }
        let Some(shader) = self.shaders.get(&shader) else {
            self.record_error(WebGlError::InvalidOperation);
            return false;
        };
        shader.compile_status
    }

    pub fn get_shader_info_log(&mut self, shader: WebGlShaderId) -> Option<String> {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return None;
        }
        let Some(shader) = self.shaders.get(&shader) else {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        };
        Some(shader.info_log.clone())
    }

    pub fn create_program(&mut self) -> WebGlProgramId {
        let id = WebGlProgramId(self.next_program_id);
        self.next_program_id += 1;
        self.programs.insert(
            id,
            ProgramObject {
                attached_vertex_shader: None,
                attached_fragment_shader: None,
                translated: None,
                reflection: None,
                pipelines: HashMap::new(),
                uniform_block_bytes: Vec::new(),
                sampler_texture_units: Vec::new(),
                link_status: false,
                info_log: String::new(),
            },
        );
        id
    }

    pub fn attach_shader(&mut self, program: WebGlProgramId, shader: WebGlShaderId) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        let Some(shader_object) = self.shaders.get(&shader) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(program) = self.programs.get_mut(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        match shader_object.stage {
            ShaderStage::Vertex => program.attached_vertex_shader = Some(shader),
            ShaderStage::Fragment => program.attached_fragment_shader = Some(shader),
        }
        program.link_status = false;
        program.translated = None;
        program.reflection = None;
        program.pipelines.clear();
        program.uniform_block_bytes.clear();
        program.sampler_texture_units.clear();
        program.info_log.clear();
    }

    pub fn link_program(&mut self, program: WebGlProgramId) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        let Some((vertex_shader_id, fragment_shader_id)) =
            self.programs.get(&program).and_then(|program| {
                Some((
                    program.attached_vertex_shader?,
                    program.attached_fragment_shader?,
                ))
            })
        else {
            self.set_program_link_failure(program, "program link failed: missing shader stage");
            return;
        };
        let Some(vertex_shader) = self.shaders.get(&vertex_shader_id) else {
            self.set_program_link_failure(program, "program link failed: missing vertex shader");
            return;
        };
        let Some(fragment_shader) = self.shaders.get(&fragment_shader_id) else {
            self.set_program_link_failure(program, "program link failed: missing fragment shader");
            return;
        };
        if !vertex_shader.compile_status
            || !fragment_shader.compile_status
            || vertex_shader.compiled.is_none()
            || fragment_shader.compiled.is_none()
            || vertex_shader.compiled_revision != Some(vertex_shader.source_revision)
            || fragment_shader.compiled_revision != Some(fragment_shader.source_revision)
        {
            self.set_program_link_failure(
                program,
                "program link failed: attached shader did not compile",
            );
            return;
        }
        let vertex_compiled = vertex_shader.compiled.as_ref().expect("checked above");
        let fragment_compiled = fragment_shader.compiled.as_ref().expect("checked above");
        debug_assert_eq!(vertex_compiled.source(), vertex_shader.source);
        debug_assert_eq!(fragment_compiled.source(), fragment_shader.source);
        let cache_key =
            canonical_essl_cache_key(vertex_compiled.source(), fragment_compiled.source());
        let cached = self.translated_programs.get(&cache_key).cloned();
        let translation = match cached {
            Some(translated) => Ok(translated),
            None => translate_compiled_essl_pair(vertex_compiled, fragment_compiled),
        };
        let translated = match translation {
            Ok(translated) => translated,
            Err(ShaderTranslationError::UnsupportedCanonicalPair) => {
                self.set_program_link_failure(program, "program link failed: interface validation");
                return;
            },
            Err(error) => {
                self.set_program_link_failure(program, &format!("program link failed: {error}"));
                return;
            },
        };
        self.translated_programs
            .entry(cache_key)
            .or_insert_with(|| translated.clone());
        let reflection = translated.reflection.clone();
        let Some(program_object) = self.programs.get_mut(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        program_object.translated = Some(translated);
        program_object.uniform_block_bytes = vec![0u8; reflection.uniform_block_size as usize];
        // WebGL initializes sampler uniforms to texture unit 0. Keeping them
        // unset here makes helper programs that rely on the default sampler
        // value fail at draw time even though link succeeds.
        program_object.sampler_texture_units = vec![Some(0); reflection.samplers.len()];
        program_object.reflection = Some(reflection);
        program_object.pipelines.clear();
        program_object.link_status = true;
        program_object.info_log.clear();
    }

    pub fn get_program_link_status(&mut self, program: WebGlProgramId) -> bool {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return false;
        }
        let Some(program) = self.programs.get(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return false;
        };
        program.link_status
    }

    pub fn get_program_info_log(&mut self, program: WebGlProgramId) -> Option<String> {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return None;
        }
        let Some(program) = self.programs.get(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        };
        Some(program.info_log.clone())
    }

    pub fn create_program_from_essl(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Option<WebGlProgramId> {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return None;
        }
        let vertex_shader = self.create_shader(ShaderStage::Vertex);
        self.shader_source(vertex_shader, vertex_source);
        self.compile_shader(vertex_shader);
        if !self.get_shader_compile_status(vertex_shader) {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        }
        let fragment_shader = self.create_shader(ShaderStage::Fragment);
        self.shader_source(fragment_shader, fragment_source);
        self.compile_shader(fragment_shader);
        if !self.get_shader_compile_status(fragment_shader) {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        }
        let program = self.create_program();
        self.attach_shader(program, vertex_shader);
        self.attach_shader(program, fragment_shader);
        self.link_program(program);
        if self.get_program_link_status(program) {
            Some(program)
        } else {
            self.record_error(WebGlError::InvalidOperation);
            None
        }
    }

    pub fn use_program(&mut self, program: Option<WebGlProgramId>) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        if let Some(id) = program {
            let Some(program) = self.programs.get(&id) else {
                self.record_error(WebGlError::InvalidOperation);
                return;
            };
            if !program.link_status {
                self.record_error(WebGlError::InvalidOperation);
                return;
            }
        }
        self.current_program = program;
    }

    pub fn get_attrib_location(&mut self, program: WebGlProgramId, name: &str) -> i32 {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return -1;
        }
        let Some(program) = self.programs.get(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return -1;
        };
        let Some(reflection) = program.reflection.as_ref() else {
            self.record_error(WebGlError::InvalidOperation);
            return -1;
        };
        reflection
            .attributes
            .iter()
            .find(|attribute| attribute.name == name)
            .map_or(-1, |attribute| attribute.location as i32)
    }

    pub fn get_uniform_location(
        &mut self,
        program: WebGlProgramId,
        name: &str,
    ) -> Option<WebGlUniformLocation> {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return None;
        }
        let Some(program_object) = self.programs.get(&program) else {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        };
        let Some(reflection) = program_object.reflection.as_ref() else {
            self.record_error(WebGlError::InvalidOperation);
            return None;
        };
        if let Some((index, _)) = reflection
            .uniforms
            .iter()
            .enumerate()
            .find(|(_, u)| u.name == name)
        {
            return Some(WebGlUniformLocation {
                program,
                slot: UniformSlot::BlockMember {
                    index: index as u32,
                },
            });
        }
        if let Some((index, _)) = reflection
            .samplers
            .iter()
            .enumerate()
            .find(|(_, s)| s.name == name)
        {
            return Some(WebGlUniformLocation {
                program,
                slot: UniformSlot::Sampler {
                    index: index as u32,
                },
            });
        }
        None
    }

    pub fn uniform1f(&mut self, location: WebGlUniformLocation, x: f32) {
        self.write_block_member(location, UniformKind::Float32, &x.to_ne_bytes());
    }

    pub fn uniform2f(&mut self, location: WebGlUniformLocation, x: f32, y: f32) {
        let bytes = [x.to_ne_bytes(), y.to_ne_bytes()].concat();
        self.write_block_member(location, UniformKind::Float32x2, &bytes);
    }

    pub fn uniform3f(&mut self, location: WebGlUniformLocation, x: f32, y: f32, z: f32) {
        let bytes = [x.to_ne_bytes(), y.to_ne_bytes(), z.to_ne_bytes()].concat();
        self.write_block_member(location, UniformKind::Float32x3, &bytes);
    }

    pub fn uniform4f(&mut self, location: WebGlUniformLocation, x: f32, y: f32, z: f32, w: f32) {
        let bytes = [
            x.to_ne_bytes(),
            y.to_ne_bytes(),
            z.to_ne_bytes(),
            w.to_ne_bytes(),
        ]
        .concat();
        self.write_block_member(location, UniformKind::Float32x4, &bytes);
    }

    pub fn uniform2fv(&mut self, location: WebGlUniformLocation, value: &[f32; 2]) {
        self.write_block_member(location, UniformKind::Float32x2, &f32_slice_to_bytes(value));
    }

    pub fn uniform3fv(&mut self, location: WebGlUniformLocation, value: &[f32; 3]) {
        self.write_block_member(location, UniformKind::Float32x3, &f32_slice_to_bytes(value));
    }

    pub fn uniform4fv(&mut self, location: WebGlUniformLocation, value: &[f32; 4]) {
        self.write_block_member(location, UniformKind::Float32x4, &f32_slice_to_bytes(value));
    }

    /// Column-major mat3. Caller passes 9 floats in column-major order;
    /// the reflected WGSL column stride determines where each column lands.
    pub fn uniform_matrix3fv(&mut self, location: WebGlUniformLocation, value: &[f32; 9]) {
        let bytes = f32_slice_to_bytes(value);
        self.write_matrix_member(location, UniformKind::Matrix3, &bytes, 3, 3);
    }

    /// Column-major mat4. Caller passes 16 floats in column-major order.
    pub fn uniform_matrix4fv(&mut self, location: WebGlUniformLocation, value: &[f32; 16]) {
        let bytes = f32_slice_to_bytes(value);
        self.write_matrix_member(location, UniformKind::Matrix4, &bytes, 4, 4);
    }

    fn write_block_member(
        &mut self,
        location: WebGlUniformLocation,
        expected_kind: UniformKind,
        bytes: &[u8],
    ) {
        self.write_block_member_with(location, expected_kind, bytes, |buffer, uniform, bytes| {
            let offset = uniform.block_offset as usize;
            let size = uniform.block_size as usize;
            let Some(end) = offset.checked_add(bytes.len()) else {
                return false;
            };
            if bytes.len() > size || end > buffer.len() {
                return false;
            }
            buffer[offset..end].copy_from_slice(bytes);
            true
        });
    }

    fn write_matrix_member(
        &mut self,
        location: WebGlUniformLocation,
        expected_kind: UniformKind,
        bytes: &[u8],
        columns: usize,
        rows: usize,
    ) {
        self.write_block_member_with(location, expected_kind, bytes, |buffer, uniform, bytes| {
            let Some(stride) = uniform.matrix_stride.map(|stride| stride as usize) else {
                return false;
            };
            let Some(column_bytes) = rows.checked_mul(std::mem::size_of::<f32>()) else {
                return false;
            };
            let Some(member_bytes) = stride.checked_mul(columns) else {
                return false;
            };
            let Some(payload_bytes) = column_bytes.checked_mul(columns) else {
                return false;
            };
            if stride < column_bytes
                || bytes.len() != payload_bytes
                || member_bytes > uniform.block_size as usize
            {
                return false;
            }
            let offset = uniform.block_offset as usize;
            let Some(member_end) = offset.checked_add(member_bytes) else {
                return false;
            };
            if member_end > buffer.len() {
                return false;
            }
            for column in 0..columns {
                let source_start = column * column_bytes;
                let destination_start = offset + column * stride;
                buffer[destination_start..destination_start + column_bytes]
                    .copy_from_slice(&bytes[source_start..source_start + column_bytes]);
            }
            true
        });
    }

    fn write_block_member_with(
        &mut self,
        location: WebGlUniformLocation,
        expected_kind: UniformKind,
        bytes: &[u8],
        writer: impl FnOnce(&mut [u8], &UniformReflection, &[u8]) -> bool,
    ) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        if self.current_program != Some(location.program) {
            self.record_error(WebGlError::InvalidOperation);
            return;
        }
        let UniformSlot::BlockMember { index } = location.slot else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(program) = self.programs.get_mut(&location.program) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(reflection) = program.reflection.as_ref() else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(uniform) = reflection.uniforms.get(index as usize) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        if uniform.kind != expected_kind {
            self.record_error(WebGlError::InvalidOperation);
            return;
        }
        if !writer(&mut program.uniform_block_bytes, uniform, bytes) {
            self.record_error(WebGlError::InvalidOperation);
        }
    }

    pub fn uniform1i(&mut self, location: WebGlUniformLocation, value: i32) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        if self.current_program != Some(location.program) {
            self.record_error(WebGlError::InvalidOperation);
            return;
        }
        if value < 0 || value as usize >= MAX_TEXTURE_IMAGE_UNITS {
            self.record_error(WebGlError::InvalidValue);
            return;
        }
        let UniformSlot::Sampler { index } = location.slot else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(program) = self.programs.get_mut(&location.program) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(reflection) = program.reflection.as_ref() else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(sampler) = reflection.samplers.get(index as usize) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        if sampler.kind != UniformKind::Sampler2D && sampler.kind != UniformKind::SamplerCube {
            self.record_error(WebGlError::InvalidOperation);
            return;
        }
        let Some(slot) = program.sampler_texture_units.get_mut(index as usize) else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        *slot = Some(value as u32);
    }

    pub fn enable_vertex_attrib_array(&mut self, index: u32) {
        let Some(attrib) = self.attrib_mut(index) else {
            self.record_error(WebGlError::InvalidValue);
            return;
        };
        attrib.enabled = true;
    }

    pub fn vertex_attrib_pointer_f32(
        &mut self,
        index: u32,
        size: u32,
        normalized: bool,
        stride: u64,
        offset: u64,
    ) {
        if self.lost {
            self.record_error(WebGlError::ContextLostWebgl);
            return;
        }
        if normalized || !matches!(size, 1 | 2 | 3 | 4) || stride % 4 != 0 || offset % 4 != 0 {
            self.record_error(WebGlError::InvalidValue);
            return;
        }
        let Some(bound) = self.bound_array_buffer else {
            self.record_error(WebGlError::InvalidOperation);
            return;
        };
        let Some(attrib) = self.attrib_mut(index) else {
            self.record_error(WebGlError::InvalidValue);
            return;
        };
        attrib.buffer = Some(bound);
        attrib.size = size;
        attrib.stride = stride;
        attrib.offset = offset;
    }
}
