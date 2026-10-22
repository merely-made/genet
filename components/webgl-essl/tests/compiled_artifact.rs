/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use webgl_essl::validate::ShaderStage;
use webgl_essl::{LinkedCompileError, compile_artifact, compile_linked, link_compiled};

fn entry_input_locations(wgsl: &str, stage: naga::ShaderStage) -> Vec<u32> {
    let module = naga::front::wgsl::parse_str(wgsl).expect("emitted WGSL parses");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("emitted WGSL validates");
    let entry = module
        .entry_points
        .iter()
        .find(|entry| entry.stage == stage)
        .expect("expected stage entry point");
    let mut locations: Vec<_> = entry
        .function
        .arguments
        .iter()
        .filter_map(|argument| match &argument.binding {
            Some(naga::Binding::Location { location, .. }) => Some(*location),
            _ => None,
        })
        .collect();
    locations.sort_unstable();
    locations
}

#[test]
fn retained_stages_link_with_the_same_interface_and_reflection_as_source_compilation() {
    let vertex = r#"
attribute vec2 a_position;
varying vec4 v_red;
varying vec4 v_green;
uniform vec4 u_shared;
uniform float u_vertex_only;
void main() {
    v_red = u_shared;
    v_green = vec4(u_vertex_only, 1.0, 0.0, 1.0);
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;
    let fragment = r#"
precision mediump float;
varying vec4 v_green;
varying vec4 v_red;
uniform float u_fragment_only;
uniform vec4 u_shared;
void main() {
    gl_FragColor = vec4(v_red.r, v_green.g, u_shared.b + u_fragment_only, 1.0);
}
"#;
    let vertex_artifact = compile_artifact(vertex, ShaderStage::Vertex).expect("vertex compiles");
    let fragment_artifact =
        compile_artifact(fragment, ShaderStage::Fragment).expect("fragment compiles");
    assert_eq!(vertex_artifact.source(), vertex);
    assert_eq!(vertex_artifact.stage(), ShaderStage::Vertex);
    let linked = link_compiled(&vertex_artifact, &fragment_artifact).expect("artifacts link");
    let source_linked = compile_linked(vertex, fragment).expect("source pair links");
    assert_eq!(linked.reflection, source_linked.reflection);
    for (stage, retained, from_source, expected_locations) in [
        (
            naga::ShaderStage::Vertex,
            &linked.vertex_wgsl,
            &source_linked.vertex_wgsl,
            vec![0],
        ),
        (
            naga::ShaderStage::Fragment,
            &linked.fragment_wgsl,
            &source_linked.fragment_wgsl,
            vec![0, 1],
        ),
    ] {
        assert_eq!(entry_input_locations(retained, stage), expected_locations);
        assert_eq!(
            entry_input_locations(from_source, stage),
            expected_locations
        );
    }
    assert_eq!(linked.reflection.varyings.len(), 2);
    assert_eq!(linked.reflection.uniforms.len(), 3);
}

#[test]
fn retained_stages_still_reject_an_interface_conflict() {
    let vertex = compile_artifact(
        "varying vec4 v_color; void main() { v_color = vec4(1.0); gl_Position = vec4(0.0); }",
        ShaderStage::Vertex,
    )
    .expect("vertex compiles independently");
    let fragment = compile_artifact(
        "precision mediump float; varying vec2 v_color; void main() { gl_FragColor = vec4(v_color, 0.0, 1.0); }",
        ShaderStage::Fragment,
    )
    .expect("fragment compiles independently");
    assert!(matches!(
        link_compiled(&vertex, &fragment),
        Err(LinkedCompileError::Link(_))
    ));
}
