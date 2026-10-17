use webgl_essl::ast::PrecisionQualifier;
use webgl_essl::validate::ShaderStage;
use webgl_essl::{CompileError, ErrorKind, LinkedCompileError, compile, compile_linked};

const VERTEX: &str =
    "attribute vec2 a_position; void main() { gl_Position = vec4(a_position, 0.0, 1.0); }";

#[test]
fn fragment_primary_draw_buffer_alias_uses_parsed_index() {
    let source = "precision mediump float; void main() { gl_FragData /* comment */ [ 0 ] = vec4(0.0, 1.0, 0.0, 1.0); }";
    let single = compile(source, ShaderStage::Fragment).expect("fragment compiles");
    let linked = compile_linked(VERTEX, source).expect("fragment links");
    assert!(single.wgsl.contains("@fragment"));
    assert!(linked.fragment_wgsl.contains("@fragment"));
    assert_eq!(
        linked.reflection.fragment_float_precision,
        Some(PrecisionQualifier::Medium)
    );
}

#[test]
fn non_primary_fragment_draw_buffer_is_explicitly_unsupported() {
    let source = "precision mediump float; void main() { gl_FragData[1] = vec4(1.0); }";
    let CompileError::Parse(error) = compile(source, ShaderStage::Fragment).unwrap_err() else {
        panic!("second output must fail during parsing");
    };
    assert!(matches!(error.kind, ErrorKind::Unsupported { .. }));
    let LinkedCompileError::Parse(error) = compile_linked(VERTEX, source).unwrap_err() else {
        panic!("linked second output must fail during parsing");
    };
    assert!(matches!(error.kind, ErrorKind::Unsupported { .. }));
}

#[test]
fn fragment_precision_comes_from_declarations_only() {
    let source = "precision lowp float; /* precision highp float; */ precision mediump int; void main() { gl_FragColor = vec4(1.0); }";
    let linked = compile_linked(VERTEX, source).expect("fragment links");
    assert_eq!(
        linked.reflection.fragment_float_precision,
        Some(PrecisionQualifier::Low)
    );
}
