/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Translation receipts after the switch-over to `webgl-essl`.
//!
//! The old naga `glsl-in` intermediate is gone — `webgl-essl`
//! emits WGSL directly via SPIR-V → naga `spv-in` → `wgsl-out`.
//! These tests stay surface-only: they pin the shape WGSL
//! consumers (the pipeline builder, the bind-group factories,
//! the WebGL `getAttribLocation` / `getUniformLocation` lookups)
//! actually depend on. Specifically:
//!
//! * `@location(N)` / `@builtin(position)` decorations on the
//!   stage interface,
//! * `@group(0) @binding(N)` decorations for the uniform Block
//!   and any samplers, and
//! * the complete [`ProgramReflection`] attribute, uniform, and
//!   sampler lists that programs and draw paths consult.
//!
//! Anything finer-grained (which struct field name `webgl-essl`
//! lays out the uniform Block with, what naming `wgsl-out`
//! produces for synthesized locals) is the lowering's
//! implementation detail, covered by `webgl-essl`'s own suite.

use super::*;

#[test]
fn canonical_essl_pair_translates_through_webgl_essl() {
    let translated = translate_canonical_essl_pair(
        CANONICAL_TRIANGLE_VERTEX_SHADER,
        CANONICAL_TRIANGLE_FRAGMENT_SHADER,
    )
    .expect("canonical pair translates");

    assert!(translated.vertex_wgsl.contains("@vertex"));
    assert!(translated.vertex_wgsl.contains("@location(0)"));
    assert!(translated.vertex_wgsl.contains("@builtin(position)"));
    assert!(translated.fragment_wgsl.contains("@fragment"));
    assert!(translated.fragment_wgsl.contains("@location(0)"));
    assert_eq!(
        translated.reflection.attributes,
        vec![VertexAttributeReflection {
            name: "a_position".to_string(),
            location: 0,
            kind: VertexAttributeKind::Float32x2,
        }]
    );
    assert!(translated.reflection.uniforms.is_empty());
    assert!(translated.reflection.samplers.is_empty());
    assert_eq!(
        translated.reflection.fragment_float_precision,
        WebGlPrecision::Medium
    );
}

#[test]
fn varying_link_mismatch_rejected_at_pair_check() {
    let vertex = r#"
attribute vec2 a_position;
attribute vec4 a_color;
varying vec4 v_color;
void main() {
    v_color = a_color;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    // Fragment reads a varying the vertex shader never writes.
    let fragment = r#"
precision mediump float;
varying vec4 other_color;
void main() {
    gl_FragColor = other_color;
}
"#;
    let result = translate_canonical_essl_pair(vertex, fragment);
    assert!(matches!(
        result,
        Err(ShaderTranslationError::UnsupportedCanonicalPair)
    ));
}

#[test]
fn fragment_with_invalid_essl_surfaces_validation_error() {
    // `discard` in a vertex shader fails webgl-essl validator R2.
    let result =
        validate_canonical_vertex_source("void main() { discard; gl_Position = vec4(0.0); }");
    assert!(matches!(result, Err(ShaderTranslationError::Validate(_))));
}

#[test]
fn canonical_fragment_accepts_float_precision_variants() {
    for (precision, expected) in [
        ("lowp", WebGlPrecision::Low),
        ("mediump", WebGlPrecision::Medium),
        ("highp", WebGlPrecision::High),
    ] {
        let fragment = format!(
            "precision {precision} float; void main() {{ gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0); }}"
        );
        let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, &fragment)
            .expect("precision variant translates");
        assert_eq!(translated.reflection.fragment_float_precision, expected);
    }
}

#[test]
fn canonical_pair_lowers_varying_color_to_wgsl() {
    let vertex = r#"
attribute vec2 a_position;
attribute vec4 a_color;
varying vec4 v_color;
void main() {
    v_color = a_color;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
varying vec4 v_color;
void main() {
    gl_FragColor = v_color;
}
"#;

    let translated =
        translate_canonical_essl_pair(vertex, fragment).expect("varying color pair translates");

    // Two attribute locations on the vertex shader, one varying
    // crossing to the fragment shader (also at @location(0)
    // because the fragment-side counter is independent).
    assert!(translated.vertex_wgsl.contains("@location(0)"));
    assert!(translated.vertex_wgsl.contains("@location(1)"));
    assert!(translated.fragment_wgsl.contains("@location(0)"));
    assert_eq!(
        translated.reflection.attributes,
        vec![
            VertexAttributeReflection {
                name: "a_position".to_string(),
                location: 0,
                kind: VertexAttributeKind::Float32x2,
            },
            VertexAttributeReflection {
                name: "a_color".to_string(),
                location: 1,
                kind: VertexAttributeKind::Float32x4,
            },
        ]
    );
}

#[test]
fn canonical_fragment_lowers_uniform_color_with_group_zero_binding_zero() {
    let fragment = r#"
precision mediump float;
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
"#;

    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("uniform color translates");

    assert!(translated.fragment_wgsl.contains("@group(0)"));
    assert!(translated.fragment_wgsl.contains("@binding(0)"));
    assert_eq!(
        translated.reflection.uniforms,
        vec![UniformReflection {
            name: "u_color".to_string(),
            binding: 0,
            kind: UniformKind::Float32x4,
            member_index: 0,
            block_offset: 0,
            block_size: 16,
            matrix_stride: None,
        }]
    );
}

#[test]
fn canonical_pair_lowers_texture_sampler_at_bindings_one_and_two() {
    let vertex = r#"
attribute vec2 a_position;
attribute vec2 a_uv;
varying vec2 v_uv;
void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
varying vec2 v_uv;
uniform sampler2D u_texture;
void main() {
    gl_FragColor = texture2D(u_texture, v_uv);
}
"#;

    let translated =
        translate_canonical_essl_pair(vertex, fragment).expect("texture sampler pair translates");
    assert!(translated.vertex_wgsl.contains("@location(1)"));
    assert!(translated.fragment_wgsl.contains("@group(0)"));
    // Uniform Block reserves @binding(0); image binds at (1),
    // sampler at (2).
    assert!(translated.fragment_wgsl.contains("@binding(1)"));
    assert!(translated.fragment_wgsl.contains("@binding(2)"));
    assert_eq!(
        translated.reflection.attributes,
        vec![
            VertexAttributeReflection {
                name: "a_position".to_string(),
                location: 0,
                kind: VertexAttributeKind::Float32x2,
            },
            VertexAttributeReflection {
                name: "a_uv".to_string(),
                location: 1,
                kind: VertexAttributeKind::Float32x2,
            },
        ]
    );
    assert_eq!(
        translated.reflection.samplers,
        vec![SamplerReflection {
            name: "u_texture".to_string(),
            image_binding: 1,
            sampler_binding: 2,
            kind: UniformKind::Sampler2D,
        }]
    );
}

#[test]
fn renamed_position_attribute_threads_through_reflection() {
    let vertex = r#"
attribute vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

    let translated = translate_canonical_essl_pair(vertex, CANONICAL_TRIANGLE_FRAGMENT_SHADER)
        .expect("renamed vertex translates");

    assert_eq!(translated.reflection.attributes.len(), 1);
    assert_eq!(translated.reflection.attributes[0].name, "position");
    assert_eq!(translated.reflection.attributes[0].location, 0);
}

#[test]
fn vertex_without_attributes_translates_with_empty_reflection() {
    let vertex = "void main() { gl_Position = vec4(0.0, 0.0, 0.0, 1.0); }";
    let translated = translate_canonical_essl_pair(vertex, CANONICAL_TRIANGLE_FRAGMENT_SHADER)
        .expect("vertex with no attributes translates");

    assert!(translated.reflection.attributes.is_empty());
}

#[test]
fn reordered_vertex_attributes_keep_declared_locations() {
    let vertex = r#"
attribute vec4 a_color;
attribute vec2 a_position;
void main() {
    gl_Position = vec4(a_position, 0.0, 1.0) * a_color;
}
"#;
    let translated = translate_canonical_essl_pair(vertex, CANONICAL_TRIANGLE_FRAGMENT_SHADER)
        .expect("reordered attributes translate");

    assert_eq!(
        translated.reflection.attributes,
        vec![
            VertexAttributeReflection {
                name: "a_color".to_string(),
                location: 0,
                kind: VertexAttributeKind::Float32x4,
            },
            VertexAttributeReflection {
                name: "a_position".to_string(),
                location: 1,
                kind: VertexAttributeKind::Float32x2,
            },
        ]
    );
}

#[test]
fn cache_key_normalizes_on_source_text() {
    // The cache key is now source-derived (whitespace counts).
    // Equal sources produce equal keys; differently-formatted
    // sources do not. This stays correct — webgl-essl re-parses
    // each unique key, so a normalization layer is optional
    // future work, not a correctness requirement.
    let same = canonical_essl_cache_key(
        CANONICAL_TRIANGLE_VERTEX_SHADER,
        CANONICAL_TRIANGLE_FRAGMENT_SHADER,
    );
    let again = canonical_essl_cache_key(
        CANONICAL_TRIANGLE_VERTEX_SHADER,
        CANONICAL_TRIANGLE_FRAGMENT_SHADER,
    );
    assert_eq!(same, again);
}

#[test]
fn linked_program_allows_unused_bool_uniform() {
    let fragment = r#"
precision mediump float;
uniform bool u_unused;
void main() { gl_FragColor = vec4(1.0); }
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("unused bool uniform is outside the uniform block");
    assert!(translated.reflection.uniforms.is_empty());
    assert_eq!(translated.reflection.uniform_block_size, 0);
}

#[test]
fn unsupported_uniform_before_supported_uniform_does_not_shift_member_index() {
    let fragment = r#"
precision mediump float;
uniform bool u_unused;
uniform float u_alpha;
void main() { gl_FragColor = vec4(u_alpha); }
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("unsupported unused uniform does not disturb linked block members");
    assert_eq!(translated.reflection.uniforms.len(), 1);
    assert_eq!(translated.reflection.uniforms[0].name, "u_alpha");
    assert_eq!(translated.reflection.uniforms[0].member_index, 0);
    assert_eq!(translated.reflection.uniforms[0].block_offset, 0);
}

#[test]
fn vertex_validation_rejects_recursion() {
    // `webgl-essl` validator R1 rejects user-function recursion.
    let recursive = r#"
attribute vec2 a_position;
vec4 self_call(vec4 p) { return self_call(p); }
void main() {
    gl_Position = self_call(vec4(a_position, 0.0, 1.0));
}
"#;
    let result = validate_canonical_vertex_source(recursive);
    assert!(matches!(result, Err(ShaderTranslationError::Validate(_))));
}

#[test]
fn fragment_validation_rejects_reserved_identifier() {
    // R5 — `gl_*` is a reserved prefix for user identifiers.
    let bad = r#"
precision mediump float;
uniform vec4 gl_userColor;
void main() {
    gl_FragColor = gl_userColor;
}
"#;
    let result = validate_canonical_fragment_source(bad);
    assert!(matches!(result, Err(ShaderTranslationError::Validate(_))));
}

// =====================================================================
// receipts for shapes the pre-switch canonical parsers rejected but
// webgl-essl handles correctly. each test below is something
// `Canonical{Vertex,Fragment}Info::parse` would have refused.
// =====================================================================

#[test]
fn broadened_fragment_accepts_scalar_broadcast_gl_fragcolor() {
    // The old fragment parser required `vec4(c0, c1, c2, c3)`
    // exactly — `vec4(0.5)` broadcast was rejected. ESSL allows
    // it; webgl-essl now lets it through.
    let fragment = r#"
precision mediump float;
void main() {
    gl_FragColor = vec4(0.5);
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("scalar-broadcast fragment translates");
    assert!(translated.fragment_wgsl.contains("@location(0)"));
}

#[test]
fn broadened_fragment_accepts_user_defined_function() {
    // The old fragment parser only accepted a single `void main`
    // body matching one of three exact templates. Helper
    // functions like the one below were unparseable.
    let fragment = r#"
precision mediump float;
uniform vec4 u_color;
vec4 tinted(vec4 base) {
    return base * u_color;
}
void main() {
    gl_FragColor = tinted(vec4(1.0));
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("user-function fragment translates");
    assert!(translated.fragment_wgsl.contains("@binding(0)"));
}

#[test]
fn broadened_essl300_vertex_with_in_decls_lowers() {
    // The old vertex parser only matched ESSL 1.00 shape
    // (`attribute` keyword, no `#version` directive). ESSL 3.00
    // uses `in` / `out` and a `#version 300 es` header.
    let vertex = r#"#version 300 es
in vec2 a_position;
in vec4 a_color;
out vec4 v_color;
void main() {
    v_color = a_color;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"#version 300 es
precision mediump float;
in vec4 v_color;
out vec4 out_color;
void main() {
    out_color = v_color;
}
"#;
    let translated =
        translate_canonical_essl_pair(vertex, fragment).expect("essl300 pair translates");
    assert!(translated.vertex_wgsl.contains("@location(0)"));
    assert!(translated.vertex_wgsl.contains("@location(1)"));
    assert!(translated.fragment_wgsl.contains("@location(0)"));
    assert_eq!(
        translated.reflection.attributes,
        vec![
            VertexAttributeReflection {
                name: "a_position".to_string(),
                location: 0,
                kind: VertexAttributeKind::Float32x2,
            },
            VertexAttributeReflection {
                name: "a_color".to_string(),
                location: 1,
                kind: VertexAttributeKind::Float32x4,
            },
        ]
    );
}

#[test]
fn broadened_fragment_accepts_swizzle_and_constructor_mix() {
    // The old fragment parser had a tight literal-color grammar
    // (`vec4(c0, c1, c2, c3)`); `vec4(u.rgb, 1.0)` was outside
    // the accepted shape.
    let fragment = r#"
precision mediump float;
uniform vec4 u_color;
void main() {
    gl_FragColor = vec4(u_color.rgb, 1.0);
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("swizzle+constructor fragment translates");
    assert!(translated.fragment_wgsl.contains("@location(0)"));
}

#[test]
fn broadened_fragment_reflects_multiple_uniforms() {
    // The old parser allowed at most one uniform of one shape;
    // a `vec4 + float` pair would have been rejected. Reflection
    // carries both members with their distinct Block offsets.
    let fragment = r#"
precision mediump float;
uniform vec4 u_color;
uniform float u_brightness;
void main() {
    gl_FragColor = u_color * u_brightness;
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("multi-uniform fragment translates");
    assert_eq!(
        translated.reflection.uniforms,
        vec![
            UniformReflection {
                name: "u_color".to_string(),
                binding: 0,
                kind: UniformKind::Float32x4,
                member_index: 0,
                block_offset: 0,
                block_size: 16,
                matrix_stride: None,
            },
            UniformReflection {
                name: "u_brightness".to_string(),
                binding: 0,
                kind: UniformKind::Float32,
                member_index: 1,
                block_offset: 16,
                block_size: 4,
                matrix_stride: None,
            },
        ]
    );
}

#[test]
fn broadened_fragment_accepts_built_in_function_in_body() {
    // `mix` / `smoothstep` / `clamp` etc. were all outside the
    // narrow parser's grammar. webgl-essl wires the §8 ESSL
    // built-in registry into typecheck + lower.
    let fragment = r#"
precision mediump float;
uniform vec4 u_a;
uniform vec4 u_b;
void main() {
    gl_FragColor = mix(u_a, u_b, 0.5);
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("built-in function fragment translates");
    assert!(translated.fragment_wgsl.contains("@location(0)"));
}

#[test]
fn linked_uniform_reflection_unions_shared_and_stage_local_uniforms() {
    let vertex = r#"
attribute vec2 a_position;
uniform vec4 u_shared;
uniform float u_vertex_only;
void main() {
    gl_Position = vec4(a_position + u_shared.xy * 0.05 + vec2(u_vertex_only * 0.01), 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
uniform float u_fragment_only;
uniform vec4 u_shared;
void main() {
    gl_FragColor = vec4(u_shared.rgb, u_fragment_only);
}
"#;
    let translated = translate_canonical_essl_pair(vertex, fragment)
        .expect("linked uniform declarations translate");

    assert_eq!(
        translated
            .reflection
            .uniforms
            .iter()
            .map(|uniform| uniform.name.as_str())
            .collect::<Vec<_>>(),
        ["u_shared", "u_vertex_only", "u_fragment_only"]
    );
    assert_eq!(
        translated
            .reflection
            .uniforms
            .iter()
            .map(|uniform| uniform.block_offset)
            .collect::<Vec<_>>(),
        [0, 16, 20]
    );
}

#[test]
fn mat2_followed_by_scalar_reflects_scalar_at_emitted_wgsl_offset() {
    let fragment = r#"
precision mediump float;
uniform mat2 u_matrix;
uniform float u_alpha;
void main() {
    gl_FragColor = vec4(u_matrix[0], u_alpha, 1.0);
}
"#;
    let translated = translate_canonical_essl_pair(CANONICAL_TRIANGLE_VERTEX_SHADER, fragment)
        .expect("mat2-plus-scalar program translates");
    let alpha = translated
        .reflection
        .uniforms
        .iter()
        .find(|uniform| uniform.name == "u_alpha")
        .expect("u_alpha reflected");

    assert_eq!(alpha.block_offset, 16);
}

#[test]
fn linked_uniform_sampler_scalar_name_conflict_is_rejected() {
    let vertex = r#"
attribute vec2 a_position;
uniform float u_conflict;
void main() {
    gl_Position = vec4(a_position * u_conflict, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
uniform sampler2D u_conflict;
void main() {
    gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
}
"#;

    assert!(translate_canonical_essl_pair(vertex, fragment).is_err());
}
