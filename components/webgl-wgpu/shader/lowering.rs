/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! ESSL → WGSL via the standalone `webgl-essl` frontend.
//!
//! The pair-interface check uses `webgl-essl::reflect` so a
//! vertex-output / fragment-input mismatch surfaces as
//! `UnsupportedCanonicalPair`. Program reflection carries all
//! attributes, uniforms, and samplers with their WGSL bindings.

use super::*;
use webgl_essl::CompileError;
use webgl_essl::ast::TypeKind as EsslTypeKind;
use webgl_essl::validate::ShaderStage as EsslStage;

pub(crate) fn translate_canonical_essl_pair(
    vertex_source: &str,
    fragment_source: &str,
) -> Result<TranslatedProgram, ShaderTranslationError> {
    let linked = webgl_essl::compile_linked(vertex_source, fragment_source)
        .map_err(map_linked_compile_error)?;
    let reflection = build_program_reflection(&linked.reflection, fragment_source)?;
    Ok(TranslatedProgram {
        vertex_wgsl: linked.vertex_wgsl,
        fragment_wgsl: linked.fragment_wgsl,
        reflection,
    })
}

pub(crate) fn validate_canonical_vertex_source(
    vertex_source: &str,
) -> Result<(), ShaderTranslationError> {
    webgl_essl::compile(vertex_source, EsslStage::Vertex)
        .map(|_| ())
        .map_err(Into::into)
}

pub(crate) fn validate_canonical_fragment_source(
    fragment_source: &str,
) -> Result<(), ShaderTranslationError> {
    webgl_essl::compile(fragment_source, EsslStage::Fragment)
        .map(|_| ())
        .map_err(Into::into)
}

pub(crate) fn canonical_essl_cache_key(
    vertex_source: &str,
    fragment_source: &str,
) -> ProgramCacheKey {
    ProgramCacheKey {
        vertex: vertex_source.to_string(),
        fragment: fragment_source.to_string(),
    }
}

fn map_linked_compile_error(error: webgl_essl::LinkedCompileError) -> ShaderTranslationError {
    match error {
        webgl_essl::LinkedCompileError::Parse(error) => {
            ShaderTranslationError::Parse(format!("{error:?}"))
        },
        webgl_essl::LinkedCompileError::Check(diagnostics) => ShaderTranslationError::Validate(
            format!("{} typecheck diagnostic(s)", diagnostics.len()),
        ),
        webgl_essl::LinkedCompileError::Validate(message) => {
            ShaderTranslationError::Validate(message)
        },
        webgl_essl::LinkedCompileError::Lower(error) => {
            ShaderTranslationError::Emit(format!("{error}"))
        },
        webgl_essl::LinkedCompileError::Link(_) => ShaderTranslationError::UnsupportedCanonicalPair,
    }
}

/// Derive [`ProgramReflection`] from `webgl-essl`'s reflection,
/// including every vertex attribute, Block uniform, and sampler.
fn build_program_reflection(
    linked: &webgl_essl::reflect::LinkedProgramReflection,
    fragment_source: &str,
) -> Result<ProgramReflection, ShaderTranslationError> {
    let mut attributes = Vec::with_capacity(linked.attributes.len());
    for input in &linked.attributes {
        attributes.push(VertexAttributeReflection {
            name: input.name.clone(),
            location: input.location,
            kind: vertex_kind_from_essl(input.kind)
                .ok_or(ShaderTranslationError::UnsupportedCanonicalPair)?,
        });
    }
    let mut uniforms: Vec<UniformReflection> = Vec::new();
    for u in &linked.uniforms {
        let kind = uniform_kind_from_essl(u.kind)
            .ok_or(ShaderTranslationError::UnsupportedCanonicalPair)?;
        uniforms.push(UniformReflection {
            name: u.name.clone(),
            binding: 0,
            kind,
            member_index: u.member_index,
            block_offset: u.byte_offset,
            block_size: u.byte_size,
            matrix_stride: u.matrix_stride,
        });
    }
    let uniform_block_size = linked.uniform_block_size;

    // Samplers only come from the fragment stage today
    // (texture sampling is fragment-only in ESSL 1.00; ESSL
    // 3.00 vertex texture is feature-flagged out for now).
    let mut samplers = Vec::with_capacity(linked.samplers.len());
    for s in &linked.samplers {
        let kind = uniform_kind_from_essl(s.kind)
            .ok_or(ShaderTranslationError::UnsupportedCanonicalPair)?;
        samplers.push(SamplerReflection {
            name: s.name.clone(),
            image_binding: s.image_binding,
            sampler_binding: s.sampler_binding,
            kind,
        });
    }

    let fragment_float_precision =
        extract_fragment_float_precision(fragment_source).unwrap_or(WebGlPrecision::Medium);
    Ok(ProgramReflection {
        attributes,
        uniforms,
        samplers,
        uniform_block_size,
        fragment_float_precision,
    })
}

fn vertex_kind_from_essl(kind: EsslTypeKind) -> Option<VertexAttributeKind> {
    match kind {
        EsslTypeKind::Float => Some(VertexAttributeKind::Float32),
        EsslTypeKind::Vec2 => Some(VertexAttributeKind::Float32x2),
        EsslTypeKind::Vec3 => Some(VertexAttributeKind::Float32x3),
        EsslTypeKind::Vec4 => Some(VertexAttributeKind::Float32x4),
        _ => None,
    }
}

fn uniform_kind_from_essl(kind: EsslTypeKind) -> Option<UniformKind> {
    match kind {
        EsslTypeKind::Float => Some(UniformKind::Float32),
        EsslTypeKind::Vec2 => Some(UniformKind::Float32x2),
        EsslTypeKind::Vec3 => Some(UniformKind::Float32x3),
        EsslTypeKind::Vec4 => Some(UniformKind::Float32x4),
        EsslTypeKind::Mat2 => Some(UniformKind::Matrix2),
        EsslTypeKind::Mat3 => Some(UniformKind::Matrix3),
        EsslTypeKind::Mat4 => Some(UniformKind::Matrix4),
        EsslTypeKind::Int => Some(UniformKind::Int),
        EsslTypeKind::Sampler2D => Some(UniformKind::Sampler2D),
        EsslTypeKind::SamplerCube => Some(UniformKind::SamplerCube),
        _ => None,
    }
}

/// Scan the fragment source for a `precision <qualifier> float;`
/// declaration so program reflection can carry it through.
/// `webgl-essl`'s validator handles the actual precision rules;
/// this is just a surface-level scrape for the public field.
fn extract_fragment_float_precision(source: &str) -> Option<WebGlPrecision> {
    let mut last = None;
    let mut rest = source;
    while let Some(start) = rest.find("precision") {
        let after = &rest[start + "precision".len()..];
        let Some(after) = after.strip_prefix(|c: char| c.is_whitespace()) else {
            rest = after;
            continue;
        };
        let mut parts = after.splitn(2, char::is_whitespace);
        let qualifier = parts.next().unwrap_or("");
        let tail = parts.next().unwrap_or("");
        let tail = tail.trim_start();
        if let Some(type_token) = tail.split(|c: char| c == ';' || c.is_whitespace()).next() {
            if type_token == "float" {
                if let Some(precision) = WebGlPrecision::parse(qualifier) {
                    last = Some(precision);
                }
            }
        }
        rest = tail;
    }
    last
}

impl From<CompileError> for ShaderTranslationError {
    fn from(error: CompileError) -> Self {
        match error {
            CompileError::Parse(error) => Self::Parse(format!("{error:?}")),
            CompileError::Check(diagnostics) => {
                Self::Validate(format!("{} typecheck diagnostic(s)", diagnostics.len()))
            },
            CompileError::Validate(result) => Self::Validate(result.info_log),
            CompileError::Lower(error) => Self::Emit(format!("{error}")),
        }
    }
}
