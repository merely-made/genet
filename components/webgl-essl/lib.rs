/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Pure-Rust ESSL frontend.
//!
//! This crate ships the full ESSL → WGSL pipeline as separate modules
//! plus a production-shaped [`compile`] entry point that runs the
//! pipeline in order and stops at the first failure:
//!
//! 1. [`lex`] / [`parse`] — byte-level lexer and a recursive-descent +
//!    Pratt parser. ESSL 1.00 covered; the ESSL 3.00 delta is partial
//!    (shift / bitwise / `~`, `#version` directive, `in` / `out` /
//!    `centroid` / `flat` / `smooth`, `switch` / `case` / `default`).
//! 2. [`check`] — symbol resolution, literal types, identifier types,
//!    binary / unary / ternary / member / call / index types, ESSL
//!    constructor rules, swizzles, and the §8 built-in function
//!    registry.
//! 3. [`validate`] — ANGLE-inspired WebGL restrictions (R1 recursion,
//!    R2 stage-gated `discard`, R3 `main` signature, R4 Appendix A
//!    `for` loops, R5 reserved identifiers, R6 expression complexity,
//!    R7 call-stack depth, R8 fragment float-family precision)
//!    rendered as `getShaderInfoLog`-shaped lines.
//! 4. [`lower`] — ESSL → SPIR-V (rspirv) → naga `spv-in` → naga
//!    validation → WGSL (`wgsl-out`). Today's accepted shape covers
//!    vertex / fragment shaders with attribute / uniform globals,
//!    vec_n constructors, and Float/Vec_n binary arithmetic.
//!
//! Architecture note:
//! `genet/docs/2026-05-28_webgl_essl_rust_frontend_spike.md`. Borrow
//! doctrine: read mature peers (chumsky / mozangle / naga / rspirv)
//! for technique, take what fits, attribute when copying expression.

#![deny(unsafe_code)]

pub mod ast;
pub mod check;
pub mod error;
pub mod lex;
pub mod lower;
pub mod parse;
pub mod reflect;
pub mod span;
pub mod token;
pub mod validate;
pub mod visit;

pub use ast::TranslationUnit;
pub use error::{Error, ErrorKind};
pub use span::Span;

use std::collections::HashMap;

use ast::TypeKind;

/// One successful shader compilation, tied to its exact source and stage.
/// The parsed tree and resolved types are retained for pair-dependent linking.
pub struct CompiledShader {
    prepared: PreparedShader,
    wgsl: String,
    info_log: String,
}

struct PreparedShader {
    source: String,
    stage: validate::ShaderStage,
    tu: TranslationUnit,
    types: HashMap<Span, TypeKind>,
}

impl CompiledShader {
    pub fn source(&self) -> &str {
        &self.prepared.source
    }

    pub fn stage(&self) -> validate::ShaderStage {
        self.prepared.stage
    }
}

/// Compile once and retain the syntax and type facts needed by linking.
pub fn compile_artifact(
    source: &str,
    stage: validate::ShaderStage,
) -> Result<CompiledShader, CompileError> {
    let tu = parse_source_for_stage(source, stage).map_err(CompileError::Parse)?;
    let checked = check::check(&tu);
    if !checked.diagnostics.is_empty() {
        return Err(CompileError::Check(checked.diagnostics));
    }
    let validation = validate::validate_with_types(&tu, source, stage, &checked.types);
    if validation.num_errors() > 0 {
        return Err(CompileError::Validate(validation));
    }
    let wgsl = lower::lower_to_wgsl_with_types(&tu, stage, &checked.types, None)
        .map_err(CompileError::Lower)?;
    Ok(CompiledShader {
        prepared: PreparedShader {
            source: source.to_owned(),
            stage,
            tu,
            types: checked.types,
        },
        wgsl,
        info_log: validation.info_log,
    })
}

/// Compile a shader pair under one interface and uniform layout owner.
pub fn compile_linked(
    vertex_source: &str,
    fragment_source: &str,
) -> Result<LinkedCompileResult, LinkedCompileError> {
    let vertex = parse_source(vertex_source).map_err(LinkedCompileError::Parse)?;
    let fragment = parse_source_for_stage(fragment_source, validate::ShaderStage::Fragment)
        .map_err(LinkedCompileError::Parse)?;
    let vertex_checked = check::check(&vertex);
    if !vertex_checked.diagnostics.is_empty() {
        return Err(LinkedCompileError::Check(vertex_checked.diagnostics));
    }
    let fragment_checked = check::check(&fragment);
    if !fragment_checked.diagnostics.is_empty() {
        return Err(LinkedCompileError::Check(fragment_checked.diagnostics));
    }
    let plan = reflect::LinkedProgramPlan::build(&vertex, &fragment)
        .map_err(|_| LinkedCompileError::Link("stage interface or uniform type conflict".into()))?;
    let vv = validate::validate_with_types(
        &vertex,
        vertex_source,
        validate::ShaderStage::Vertex,
        &vertex_checked.types,
    );
    let fv = validate::validate_with_types(
        &fragment,
        fragment_source,
        validate::ShaderStage::Fragment,
        &fragment_checked.types,
    );
    if vv.num_errors() > 0 || fv.num_errors() > 0 {
        return Err(LinkedCompileError::Validate(format!(
            "vertex: {}\nfragment: {}",
            vv.info_log, fv.info_log
        )));
    }
    let vertex = PreparedShader {
        source: vertex_source.to_owned(),
        stage: validate::ShaderStage::Vertex,
        tu: vertex,
        types: vertex_checked.types,
    };
    let fragment = PreparedShader {
        source: fragment_source.to_owned(),
        stage: validate::ShaderStage::Fragment,
        tu: fragment,
        types: fragment_checked.types,
    };
    lower_linked(&vertex, &fragment, &plan)
}

/// Link already compiled shaders without reparsing, checking or validating.
/// Emission still runs under the pair's shared location and uniform plan.
pub fn link_compiled(
    vertex: &CompiledShader,
    fragment: &CompiledShader,
) -> Result<LinkedCompileResult, LinkedCompileError> {
    if vertex.stage() != validate::ShaderStage::Vertex
        || fragment.stage() != validate::ShaderStage::Fragment
    {
        return Err(LinkedCompileError::Link("shader stage mismatch".into()));
    }
    let plan = reflect::LinkedProgramPlan::build(&vertex.prepared.tu, &fragment.prepared.tu)
        .map_err(|_| LinkedCompileError::Link("stage interface or uniform type conflict".into()))?;
    lower_linked(&vertex.prepared, &fragment.prepared, &plan)
}

fn lower_linked(
    vertex: &PreparedShader,
    fragment: &PreparedShader,
    plan: &reflect::LinkedProgramPlan,
) -> Result<LinkedCompileResult, LinkedCompileError> {
    let vertex_wgsl =
        lower::lower_to_wgsl_with_types(&vertex.tu, vertex.stage, &vertex.types, Some(plan))
            .map_err(LinkedCompileError::Lower)?;
    let fragment_wgsl =
        lower::lower_to_wgsl_with_types(&fragment.tu, fragment.stage, &fragment.types, Some(plan))
            .map_err(LinkedCompileError::Lower)?;
    let (vertex_layout, vertex_block_size) =
        lower::reflect_emitted_uniform_layout(&vertex_wgsl).map_err(LinkedCompileError::Lower)?;
    let (fragment_layout, fragment_block_size) =
        lower::reflect_emitted_uniform_layout(&fragment_wgsl).map_err(LinkedCompileError::Lower)?;
    if vertex_layout != fragment_layout || vertex_block_size != fragment_block_size {
        return Err(LinkedCompileError::Link(
            "linked stages emitted different uniform layouts".into(),
        ));
    }
    let vertex_reflection = reflect::reflect(&vertex.tu, vertex.stage);
    let fragment_reflection = reflect::reflect(&fragment.tu, fragment.stage);
    let mut uniforms = Vec::new();
    for uniform in plan.uniforms.iter().filter(|u| u.member_index.is_some()) {
        let member_index = uniform.member_index.expect("filtered linked block member");
        let (offset, size, matrix_stride) = vertex_layout
            .get(member_index as usize)
            .copied()
            .ok_or_else(|| {
                LinkedCompileError::Link("emitted uniform block does not match linked plan".into())
            })?;
        uniforms.push(reflect::LinkedUniformLayout {
            name: uniform.name.clone(),
            kind: uniform.kind,
            member_index,
            byte_offset: offset,
            byte_size: size,
            matrix_stride,
        });
    }
    Ok(LinkedCompileResult {
        vertex_wgsl,
        fragment_wgsl,
        reflection: reflect::LinkedProgramReflection {
            attributes: vertex_reflection.inputs,
            uniforms,
            uniform_block_size: vertex_block_size,
            varyings: plan.varyings.clone(),
            samplers: fragment_reflection.samplers,
            fragment_float_precision: fragment.tu.fragment_float_precision(),
        },
    })
}

#[derive(Debug, Clone)]
pub struct LinkedCompileResult {
    pub vertex_wgsl: String,
    pub fragment_wgsl: String,
    pub reflection: reflect::LinkedProgramReflection,
}

#[derive(Debug)]
pub enum LinkedCompileError {
    Parse(Error),
    Check(Vec<check::TypeDiagnostic>),
    Validate(String),
    Lower(lower::LoweringError),
    Link(String),
}

/// Production-shaped entry: run the full ESSL → WGSL pipeline on
/// `source`. Stops at the first failing stage. Source text is
/// threaded through to the validator so info-log lines render with
/// real line numbers.
pub fn compile(source: &str, stage: validate::ShaderStage) -> Result<CompileResult, CompileError> {
    let compiled = compile_artifact(source, stage)?;
    Ok(CompileResult {
        wgsl: compiled.wgsl,
        info_log: compiled.info_log,
    })
}

/// Success result of [`compile`]. `info_log` carries any non-error
/// warnings; an empty string means a fully clean compile.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub wgsl: String,
    pub info_log: String,
}

/// Failure result of [`compile`]. Each variant identifies which
/// pipeline stage produced the failure; the inner payload is the
/// stage's native diagnostic shape so a caller that wants to render
/// `getShaderInfoLog` text has everything it needs.
#[derive(Debug)]
pub enum CompileError {
    Parse(Error),
    Check(Vec<check::TypeDiagnostic>),
    Validate(validate::ValidationResult),
    Lower(lower::LoweringError),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Parse(e) => write!(f, "parse error: {:?}", e),
            CompileError::Check(diags) => {
                write!(f, "typecheck failed with {} diagnostic(s)", diags.len())
            },
            CompileError::Validate(r) => {
                write!(f, "validation failed with {} error(s)", r.num_errors())
            },
            CompileError::Lower(e) => write!(f, "lowering failed: {e}"),
        }
    }
}

/// Lex + parse `source` into a [`TranslationUnit`]. Convenience over
/// calling [`lex::lex`] and [`parse::parse`] separately.
///
/// A leading `#version <N> <profile>` directive (the only
/// preprocessor construct ESSL exposes to authors at this layer) is
/// extracted before tokenization and attached to the resulting
/// `TranslationUnit` as the `version` field. The directive line is
/// blanked out in the lexer input so byte offsets and line numbers in
/// later diagnostics still match the original source.
pub fn parse_source(source: &str) -> Result<TranslationUnit, Error> {
    parse_source_with_fragment_output_alias(source, false)
}

fn parse_source_for_stage(
    source: &str,
    stage: validate::ShaderStage,
) -> Result<TranslationUnit, Error> {
    parse_source_with_fragment_output_alias(source, stage == validate::ShaderStage::Fragment)
}

fn parse_source_with_fragment_output_alias(
    source: &str,
    fragment_output_alias: bool,
) -> Result<TranslationUnit, Error> {
    let (version, sans_version) = extract_version_directive(source);
    let tokens = lex::lex(&sans_version)?;
    let mut tu = if fragment_output_alias {
        parse::parse_fragment(tokens, sans_version.len())?
    } else {
        parse::parse(tokens, sans_version.len())?
    };
    tu.version = version;
    Ok(tu)
}

/// Find a leading `#version <N> <profile>` line. Returns the numeric
/// version (e.g. `Some(300)` for `#version 300 es`) plus a copy of
/// the source with that line blanked to preserve byte offsets.
fn extract_version_directive(source: &str) -> (Option<u32>, String) {
    let mut version: Option<u32> = None;
    let mut consumed = false;
    let mut sans = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        if !consumed {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#version") {
                if let Some(after) = trimmed.strip_prefix("#version") {
                    if let Some(first_word) = after.split_whitespace().next() {
                        version = first_word.parse().ok();
                    }
                }
                consumed = true;
                // Blank the directive line; keep the trailing newline
                // (if any) so line numbers are preserved.
                let has_nl = line.ends_with('\n');
                let blank_len = line.len() - if has_nl { 1 } else { 0 };
                for _ in 0..blank_len {
                    sans.push(' ');
                }
                if has_nl {
                    sans.push('\n');
                }
                continue;
            }
            if !trimmed.is_empty() {
                consumed = true;
            }
        }
        sans.push_str(line);
    }
    (version, sans)
}
