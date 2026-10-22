/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::test_support::make_context;
use super::*;
use crate::{CANONICAL_TRIANGLE_FRAGMENT_SHADER, CANONICAL_TRIANGLE_VERTEX_SHADER};

#[test]
fn webgl_context_clear_triangle_read_pixels_and_get_error() {
    let mut context = make_context(32, 32);
    assert_eq!(
        context.context_attributes(),
        WebGlContextAttributes::default()
    );
    context.clear(wgpu::Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });

    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);
    assert_eq!(context.get_error(), WebGlError::InvalidOperation);
    assert_eq!(context.get_error(), WebGlError::NoError);

    let program = context
        .create_program_from_essl(
            CANONICAL_TRIANGLE_VERTEX_SHADER,
            CANONICAL_TRIANGLE_FRAGMENT_SHADER,
        )
        .expect("canonical program");
    let position_location = context.get_attrib_location(program, "a_position");
    assert_eq!(position_location, 0);
    assert_eq!(context.get_attrib_location(program, "missing"), -1);
    context.use_program(Some(program));

    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    context.enable_vertex_attrib_array(position_location as u32);
    context.vertex_attrib_pointer_f32(position_location as u32, 2, false, 0, 0);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);
    assert_eq!(context.get_error(), WebGlError::NoError);

    let center = context.read_pixels(16, 16, 1, 1).expect("center read");
    assert_eq!(&center[0..4], &[0, 255, 0, 255]);
    let corner = context.read_pixels(0, 0, 1, 1).expect("corner read");
    assert_eq!(&corner[0..4], &[255, 0, 0, 255]);
}

#[test]
fn webgl_context_rejects_invalid_shader_pair() {
    let mut context = make_context(4, 4);
    // Reserved `gl_*` identifier — webgl-essl validator R5
    // rejects this at compile time.
    let program = context.create_program_from_essl(
        "attribute vec2 a_position; uniform vec4 gl_userColor; \
         void main() { gl_Position = vec4(a_position, 0.0, 1.0) * gl_userColor; }",
        CANONICAL_TRIANGLE_FRAGMENT_SHADER,
    );
    assert!(program.is_none());
    assert_eq!(context.get_error(), WebGlError::InvalidOperation);
}

#[test]
fn webgl_context_get_attrib_location_reports_invalid_program() {
    let mut context = make_context(4, 4);

    assert_eq!(
        context.get_attrib_location(WebGlProgramId(99), "a_position"),
        -1
    );
    assert_eq!(context.get_error(), WebGlError::InvalidOperation);
}

#[test]
fn webgl_context_caches_translation_across_identical_source_pairs() {
    // Two programs built from the same source pair share the translated
    // entry. Distinct text has a distinct key; successful shader objects
    // retain their parsed and checked artifacts for a cold pair link.
    let mut context = make_context(4, 4);
    let first = context
        .create_program_from_essl(
            CANONICAL_TRIANGLE_VERTEX_SHADER,
            CANONICAL_TRIANGLE_FRAGMENT_SHADER,
        )
        .expect("first canonical program");
    let second = context
        .create_program_from_essl(
            CANONICAL_TRIANGLE_VERTEX_SHADER,
            CANONICAL_TRIANGLE_FRAGMENT_SHADER,
        )
        .expect("second canonical program");

    assert_ne!(first, second);
    assert_eq!(context.translated_programs.len(), 1);
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn compiled_shader_artifacts_survive_links_and_expire_with_source_revision() {
    let mut context = make_context(4, 4);
    let vertex = context.create_shader(ShaderStage::Vertex);
    let fragment = context.create_shader(ShaderStage::Fragment);
    context.shader_source(vertex, CANONICAL_TRIANGLE_VERTEX_SHADER);
    context.shader_source(fragment, CANONICAL_TRIANGLE_FRAGMENT_SHADER);
    context.compile_shader(vertex);
    context.compile_shader(fragment);
    assert!(context.shaders[&vertex].compiled.is_some());
    assert_eq!(
        context.shaders[&vertex].compiled_revision,
        Some(context.shaders[&vertex].source_revision)
    );

    for _ in 0..2 {
        let program = context.create_program();
        context.attach_shader(program, vertex);
        context.attach_shader(program, fragment);
        context.link_program(program);
        assert!(context.get_program_link_status(program));
        assert!(context.shaders[&vertex].compiled.is_some());
    }
    assert_eq!(context.translated_programs.len(), 1);

    // The same text still begins a new source revision and needs compilation.
    context.shader_source(vertex, CANONICAL_TRIANGLE_VERTEX_SHADER);
    assert!(context.shaders[&vertex].compiled.is_none());
    assert_eq!(context.shaders[&vertex].compiled_revision, None);
    let program = context.create_program();
    context.attach_shader(program, vertex);
    context.attach_shader(program, fragment);
    context.link_program(program);
    assert!(!context.get_program_link_status(program));

    context.compile_shader(vertex);
    assert!(context.shaders[&vertex].compiled.is_some());
    context.link_program(program);
    assert!(context.get_program_link_status(program));

    context.shader_source(vertex, "void main() { discard; }");
    context.compile_shader(vertex);
    assert!(!context.get_shader_compile_status(vertex));
    assert!(context.shaders[&vertex].compiled.is_none());
    assert_eq!(context.shaders[&vertex].compiled_revision, None);
    context.link_program(program);
    assert!(!context.get_program_link_status(program));
    assert!(
        context
            .get_program_info_log(program)
            .expect("link log")
            .contains("attached shader did not compile")
    );
}

#[test]
fn webgl_context_explicit_compile_link_pipeline_renders() {
    let mut context = make_context(32, 32);
    let vertex = context.create_shader(ShaderStage::Vertex);
    context.shader_source(vertex, CANONICAL_TRIANGLE_VERTEX_SHADER);
    context.compile_shader(vertex);
    assert!(context.get_shader_compile_status(vertex));
    assert_eq!(context.get_shader_info_log(vertex).as_deref(), Some(""));

    let fragment = context.create_shader(ShaderStage::Fragment);
    context.shader_source(fragment, CANONICAL_TRIANGLE_FRAGMENT_SHADER);
    context.compile_shader(fragment);
    assert!(context.get_shader_compile_status(fragment));
    assert_eq!(context.get_shader_info_log(fragment).as_deref(), Some(""));

    let program = context.create_program();
    context.attach_shader(program, vertex);
    context.attach_shader(program, fragment);
    context.link_program(program);
    assert!(context.get_program_link_status(program));
    assert_eq!(context.get_program_info_log(program).as_deref(), Some(""));
    context.use_program(Some(program));

    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    context.enable_vertex_attrib_array(0);
    context.vertex_attrib_pointer_f32(0, 2, false, 0, 0);
    context.clear(wgpu::Color::BLACK);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);
    assert_eq!(
        &context.read_pixels(16, 16, 1, 1).expect("center read")[0..4],
        &[0, 255, 0, 255]
    );
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn fragment_primary_draw_buffer_alias_keeps_source_and_draws() {
    let mut context = make_context(32, 32);
    let fragment_source = r#"
precision mediump float;
// gl_FragData[1] here is a comment, not an output.
void main() {
    gl_FragData /* spacing */ [ 0 ] = vec4(0.0, 0.0, 1.0, 1.0);
}
"#;
    let vertex = context.create_shader(ShaderStage::Vertex);
    context.shader_source(vertex, CANONICAL_TRIANGLE_VERTEX_SHADER);
    context.compile_shader(vertex);
    assert!(context.get_shader_compile_status(vertex));
    let fragment = context.create_shader(ShaderStage::Fragment);
    context.shader_source(fragment, fragment_source);
    assert_eq!(
        context.shaders.get(&fragment).unwrap().source,
        fragment_source
    );
    context.compile_shader(fragment);
    assert!(context.get_shader_compile_status(fragment));

    let program = context.create_program();
    context.attach_shader(program, vertex);
    context.attach_shader(program, fragment);
    context.link_program(program);
    assert!(context.get_program_link_status(program));
    context.use_program(Some(program));
    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    context.enable_vertex_attrib_array(0);
    context.vertex_attrib_pointer_f32(0, 2, false, 0, 0);
    context.clear(wgpu::Color::BLACK);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);
    assert_eq!(
        &context.read_pixels(16, 16, 1, 1).expect("center read")[0..4],
        &[0, 0, 255, 255]
    );
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn webgl_context_compile_failure_reports_shader_info_log() {
    let mut context = make_context(4, 4);
    let fragment = context.create_shader(ShaderStage::Fragment);
    // Reserved `gl_*` identifier — R5 — surfaces at compile
    // time with a non-empty info log.
    context.shader_source(
        fragment,
        "precision mediump float; uniform vec4 gl_userColor; \
         void main() { gl_FragColor = gl_userColor; }",
    );
    context.compile_shader(fragment);
    assert!(!context.get_shader_compile_status(fragment));
    let log = context
        .get_shader_info_log(fragment)
        .expect("shader info log");
    assert!(log.contains("Fragment shader compile failed"));
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn webgl_context_link_failure_reports_program_info_log() {
    let vertex_source = r#"
            attribute vec2 a_position;
            attribute vec4 a_color;
            varying vec4 v_color;
            void main() {
                v_color = a_color;
                gl_Position = vec4(a_position, 0.0, 1.0);
            }
        "#;
    let fragment_source = r#"
            precision mediump float;
            varying vec4 other_color;
            void main() {
                gl_FragColor = other_color;
            }
        "#;
    let mut context = make_context(4, 4);
    let vertex = context.create_shader(ShaderStage::Vertex);
    context.shader_source(vertex, vertex_source);
    context.compile_shader(vertex);
    assert!(context.get_shader_compile_status(vertex));
    let fragment = context.create_shader(ShaderStage::Fragment);
    context.shader_source(fragment, fragment_source);
    context.compile_shader(fragment);
    assert!(context.get_shader_compile_status(fragment));

    let program = context.create_program();
    context.attach_shader(program, vertex);
    context.attach_shader(program, fragment);
    context.link_program(program);
    assert!(!context.get_program_link_status(program));
    let log = context
        .get_program_info_log(program)
        .expect("program info log");
    assert!(log.contains("interface validation"));
    context.use_program(Some(program));
    assert_eq!(context.get_error(), WebGlError::InvalidOperation);
}

#[test]
fn linked_varyings_keep_name_locations_when_fragment_declarations_are_reordered() {
    let mut context = make_context(32, 32);
    let vertex = r#"
attribute vec2 a_position;
varying vec4 v_red;
varying vec4 v_green;
void main() {
    v_red = vec4(1.0, 0.0, 0.0, 1.0);
    v_green = vec4(0.0, 1.0, 0.0, 1.0);
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
varying vec4 v_green;
varying vec4 v_red;
void main() {
    gl_FragColor = vec4(v_red.r, v_green.g, 0.0, 1.0);
}
"#;
    let program = context
        .create_program_from_essl(vertex, fragment)
        .expect("reordered varying declarations link by name");
    context.use_program(Some(program));

    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    let position = context.get_attrib_location(program, "a_position") as u32;
    context.enable_vertex_attrib_array(position);
    context.vertex_attrib_pointer_f32(position, 2, false, 0, 0);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);

    let center = context.read_pixels(16, 16, 1, 1).expect("center read");
    assert_eq!(&center[0..4], &[255, 255, 0, 255]);
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn linked_uniforms_share_one_layout_across_reordered_stage_declarations() {
    let mut context = make_context(32, 32);
    let vertex = r#"
attribute vec2 a_position;
varying float v_from_vertex;
uniform vec4 u_shared;
uniform float u_vertex_only;
void main() {
    v_from_vertex = u_vertex_only + u_shared.r;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
varying float v_from_vertex;
uniform float u_fragment_only;
uniform vec4 u_shared;
void main() {
    gl_FragColor = vec4(u_shared.b, u_fragment_only, v_from_vertex, 1.0);
}
"#;
    let program = context
        .create_program_from_essl(vertex, fragment)
        .expect("shared and stage-local uniforms link");
    context.use_program(Some(program));
    let shared = context
        .get_uniform_location(program, "u_shared")
        .expect("u_shared uniform location");
    let vertex_only = context
        .get_uniform_location(program, "u_vertex_only")
        .expect("u_vertex_only uniform location");
    let fragment_only = context
        .get_uniform_location(program, "u_fragment_only")
        .expect("u_fragment_only uniform location");
    context.uniform4f(shared, 0.25, 0.0, 0.75, 1.0);
    context.uniform1f(vertex_only, 0.25);
    context.uniform1f(fragment_only, 0.125);

    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    let position = context.get_attrib_location(program, "a_position") as u32;
    context.enable_vertex_attrib_array(position);
    context.vertex_attrib_pointer_f32(position, 2, false, 0, 0);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);

    let center = context.read_pixels(16, 16, 1, 1).expect("center read");
    assert_eq!(&center[0..4], &[191, 32, 128, 255]);
    assert_eq!(context.get_error(), WebGlError::NoError);
}

#[test]
fn linked_uniform_name_with_different_stage_types_fails_to_link() {
    let vertex = r#"
attribute vec2 a_position;
uniform float u_conflict;
void main() {
    gl_Position = vec4(a_position * u_conflict, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
uniform vec2 u_conflict;
void main() {
    gl_FragColor = vec4(u_conflict, 0.0, 1.0);
}
"#;
    let mut context = make_context(4, 4);
    assert!(context.create_program_from_essl(vertex, fragment).is_none());
    assert_eq!(context.get_error(), WebGlError::InvalidOperation);
}

#[test]
fn unused_vertex_varying_output_does_not_prevent_link_or_draw() {
    let mut context = make_context(32, 32);
    let vertex = r#"
attribute vec2 a_position;
varying vec4 v_unused;
void main() {
    v_unused = vec4(1.0, 0.0, 1.0, 1.0);
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
void main() {
    gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
}
"#;
    let program = context
        .create_program_from_essl(vertex, fragment)
        .expect("unused vertex output does not block linking");
    context.use_program(Some(program));

    let vertices = context.create_buffer();
    context.bind_buffer(BufferTarget::ArrayBuffer, Some(vertices));
    context.buffer_data_f32(
        BufferTarget::ArrayBuffer,
        &[-0.8, -0.8, 0.8, -0.8, 0.0, 0.8],
        BufferUsage::StaticDraw,
    );
    let position = context.get_attrib_location(program, "a_position") as u32;
    context.enable_vertex_attrib_array(position);
    context.vertex_attrib_pointer_f32(position, 2, false, 0, 0);
    context.draw_arrays(PrimitiveMode::Triangles, 0, 3);

    let center = context.read_pixels(16, 16, 1, 1).expect("center read");
    assert_eq!(&center[0..4], &[0, 255, 0, 255]);
    assert_eq!(context.get_error(), WebGlError::NoError);
}
