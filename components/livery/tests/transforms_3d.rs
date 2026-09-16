// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! T1: CSS Transforms Level 2 parse, compute, serialize and interpolate.

use livery::values::{
    BackfaceVisibility, Interpolate, Matrix3D, Perspective, PerspectiveOrigin, Rotate, Scale,
    Transform, TransformFunction, TransformOrigin, TransformStyle, Translate,
};

const EM: f32 = 16.0;
const BOX: (f32, f32) = (200.0, 100.0);

fn matrix(css: &str) -> Matrix3D {
    css.parse::<Transform>()
        .unwrap_or_else(|error| panic!("{css}: {error}"))
        .to_matrix_3d(EM, BOX)
        .unwrap_or_else(|| panic!("{css} has no matrix"))
}

fn close(actual: &Matrix3D, expected: [f32; 16]) {
    for (index, (actual, expected)) in actual.0.iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() < 1e-4,
            "cell {index}: {actual} != {expected}"
        );
    }
}

fn round_trip(css: &str) -> String {
    let value = css
        .parse::<Transform>()
        .unwrap_or_else(|error| panic!("{css}: {error}"));
    let serialized = value.to_string();
    let reparsed = serialized
        .parse::<Transform>()
        .unwrap_or_else(|error| panic!("{css} serialized as {serialized}: {error}"));
    assert_eq!(value, reparsed, "{css} serialized as {serialized}");
    serialized
}

#[test]
fn every_level_2_function_parses_and_round_trips() {
    for css in [
        "translate3d(1px, 2px, 3px)",
        "translateZ(4px)",
        "scale3d(1, 2, 3)",
        "scaleZ(2)",
        "rotate3d(1, 1, 0, 45deg)",
        "rotateX(45deg)",
        "rotateY(0.25turn)",
        "rotateZ(100grad)",
        "perspective(500px)",
        "perspective(none)",
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)",
        "translate3d(50%, 10%, 2em) rotateY(30deg) scaleZ(0.5)",
    ] {
        round_trip(css);
    }
    assert_eq!(round_trip("translateZ(4px)"), "translateZ(4px)");
    assert_eq!(round_trip("rotateX(0)"), "rotateX(0rad)");
    assert!("rotate3d(0, 0, 0, 1deg)".parse::<Transform>().is_err());
    assert!("perspective(-1px)".parse::<Transform>().is_err());
    assert!("matrix3d(1, 2, 3)".parse::<Transform>().is_err());
    assert!("translateZ(10%)".parse::<Transform>().is_err());
}

#[test]
fn level_2_functions_compute_their_specified_matrices() {
    close(
        &matrix("translate3d(10px, 20px, 30px)"),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 30.0, 1.0,
        ],
    );
    // A translate3d percentage resolves against the reference box, Z never.
    close(
        &matrix("translate3d(50%, 10%, 3px)"),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 100.0, 10.0, 3.0, 1.0,
        ],
    );
    close(&matrix("translateZ(2em)"), {
        let mut cells = Matrix3D::IDENTITY.0;
        cells[14] = 32.0;
        cells
    });
    close(
        &matrix("scale3d(2, 3, 4)"),
        [
            2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    );
    // rotateY(90deg) takes the local X axis onto -Z.
    close(
        &matrix("rotateY(90deg)"),
        [
            0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    );
    // rotateX(90deg) takes the local Y axis onto +Z. CSS Y points down.
    close(
        &matrix("rotateX(90deg)"),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    );
    // rotateZ and rotate3d about +Z agree with the 2D rotate().
    close(&matrix("rotateZ(30deg)"), matrix("rotate(30deg)").0);
    close(
        &matrix("rotate3d(0, 0, 1, 30deg)"),
        matrix("rotate(30deg)").0,
    );
    // perspective() writes only m34.
    let perspective = matrix("perspective(400px)");
    assert_eq!(perspective.at(3, 2), -1.0 / 400.0);
    assert!(!perspective.is_affine());
    assert!(matrix("perspective(none)").is_affine());
    close(
        &matrix("matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 5, 6, 7, 1)"),
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 6.0, 7.0, 1.0,
        ],
    );
}

#[test]
fn a_transform_list_composes_left_to_right_with_the_last_applied_first() {
    // translate then scale: the translation is not scaled.
    close(&matrix("translate3d(10px, 0, 0) scale3d(2, 2, 2)"), {
        let mut cells = Matrix3D::scaling(2.0, 2.0, 2.0).0;
        cells[12] = 10.0;
        cells
    });
    // scale then translate: the translation is scaled.
    close(&matrix("scale3d(2, 2, 2) translate3d(10px, 0, 0)"), {
        let mut cells = Matrix3D::scaling(2.0, 2.0, 2.0).0;
        cells[12] = 20.0;
        cells
    });
}

#[test]
fn computed_serialization_picks_matrix_or_matrix3d() {
    assert_eq!(
        "translate(10px, 20px)"
            .parse::<Transform>()
            .expect("2d")
            .to_computed_css(EM, Some(BOX)),
        "matrix(1, 0, 0, 1, 10, 20)"
    );
    assert_eq!(
        "translate3d(10px, 20px, 30px)"
            .parse::<Transform>()
            .expect("3d")
            .to_computed_css(EM, Some(BOX)),
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)"
    );
    // A Z term that cancels out is a 2D matrix again.
    assert_eq!(
        "translateZ(0)"
            .parse::<Transform>()
            .expect("z")
            .to_computed_css(EM, Some(BOX)),
        "matrix(1, 0, 0, 1, 0, 0)"
    );
}

#[test]
fn matching_level_2_primitives_interpolate_and_mismatches_stay_discrete() {
    let from = "rotateY(0deg)".parse::<Transform>().expect("from");
    let to = "rotateY(90deg)".parse::<Transform>().expect("to");
    let Transform::Functions(middle) = from.interpolate(&to, 0.5) else {
        panic!("interpolated to none");
    };
    let [TransformFunction::RotateY(angle)] = middle.as_slice() else {
        panic!("expected a single rotateY, got {middle:?}");
    };
    assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 1e-5);

    let from = "translate3d(0, 0, 0)".parse::<Transform>().expect("from");
    let to = "translate3d(10px, 20px, 30px)"
        .parse::<Transform>()
        .expect("to");
    assert_eq!(
        from.interpolate(&to, 0.5).to_string(),
        "translate3d(5px, 10px, 15px)"
    );

    // A same-axis rotate3d interpolates; a differing axis has no primitive
    // interpolation and no 2D matrix decomposition, so it stays discrete.
    let from = "rotate3d(1, 1, 0, 0deg)".parse::<Transform>().expect("a");
    let to = "rotate3d(1, 1, 0, 90deg)".parse::<Transform>().expect("b");
    let Transform::Functions(middle) = from.interpolate(&to, 0.5) else {
        panic!("none");
    };
    assert!(matches!(
        middle.as_slice(),
        [TransformFunction::Rotate3D(1.0, 1.0, 0.0, _)]
    ));
    let other = "rotate3d(0, 1, 0, 90deg)".parse::<Transform>().expect("c");
    assert_eq!(from.interpolate(&other, 0.4), from);
    assert_eq!(from.interpolate(&other, 0.6), other);
}

#[test]
fn individual_translate_rotate_and_scale_carry_three_components() {
    assert_eq!("none".parse::<Translate>().expect("none"), Translate::None);
    assert_eq!("10px".parse::<Translate>().expect("x").to_string(), "10px");
    assert_eq!(
        "10px 20px".parse::<Translate>().expect("xy").to_string(),
        "10px 20px"
    );
    assert_eq!(
        "1px 2px 3px".parse::<Translate>().expect("xyz").to_string(),
        "1px 2px 3px"
    );
    close(
        &"1px 2px 3px"
            .parse::<Translate>()
            .expect("xyz")
            .to_matrix_3d(EM, BOX)
            .expect("matrix"),
        Matrix3D::translation(1.0, 2.0, 3.0).0,
    );
    assert!("1px 2px 3px 4px".parse::<Translate>().is_err());

    // An axis parallel to +Z normalizes to the plain angle form.
    assert!(matches!(
        "z 45deg".parse::<Rotate>().expect("z"),
        Rotate::Angle(_)
    ));
    assert!(matches!(
        "0 0 1 45deg".parse::<Rotate>().expect("numbers"),
        Rotate::Angle(_)
    ));
    let Rotate::Axis(x, y, z, _) = "x 90deg".parse::<Rotate>().expect("x") else {
        panic!("expected an axis rotation");
    };
    assert_eq!((x, y, z), (1.0, 0.0, 0.0));
    assert_eq!(
        "45deg x".parse::<Rotate>().expect("angle first"),
        "x 45deg".parse::<Rotate>().expect("axis first")
    );
    assert!("0 0 0 45deg".parse::<Rotate>().is_err());

    assert!(matches!(
        "2 2".parse::<Scale>().expect("uniform pair"),
        Scale::Uniform(2.0)
    ));
    assert_eq!(
        "1 2 3".parse::<Scale>().expect("triple"),
        Scale::Values(1.0, 2.0, 3.0)
    );
    assert_eq!(
        "1 2 3".parse::<Scale>().expect("triple").to_string(),
        "1 2 3"
    );
    close(
        &"1 2 3"
            .parse::<Scale>()
            .expect("triple")
            .to_matrix_3d()
            .expect("matrix"),
        Matrix3D::scaling(1.0, 2.0, 3.0).0,
    );
    assert!("1 2 3 4".parse::<Scale>().is_err());

    // Interpolation stays inside each family.
    assert_eq!(
        Translate::interpolate_value(
            &"0px 0px 0px".parse().expect("from"),
            &"10px 20px 30px".parse().expect("to"),
            0.5
        )
        .to_string(),
        "5px 10px 15px"
    );
    assert_eq!(
        Scale::interpolate_value(&Scale::Uniform(1.0), &Scale::Values(3.0, 5.0, 1.0), 0.5),
        Scale::Values(2.0, 3.0, 1.0)
    );
    let mixed = Rotate::interpolate_value(
        &"x 0deg".parse().expect("from"),
        &"y 90deg".parse().expect("to"),
        0.4,
    );
    assert_eq!(mixed, "x 0deg".parse::<Rotate>().expect("discrete"));
}

#[test]
fn perspective_transform_style_and_backface_visibility_parse_and_serialize() {
    assert_eq!(
        "none".parse::<Perspective>().expect("none"),
        Perspective::None
    );
    assert_eq!(
        "500px".parse::<Perspective>().expect("depth").to_string(),
        "500px"
    );
    assert_eq!(
        "2em".parse::<Perspective>().expect("em").depth_px(16.0),
        Some(32.0)
    );
    assert_eq!(Perspective::None.depth_px(16.0), None);
    assert!("-1px".parse::<Perspective>().is_err());
    assert!("0px".parse::<Perspective>().expect("zero").depth_px(16.0) == None);

    assert_eq!(
        "left top".parse::<PerspectiveOrigin>().expect("origin"),
        PerspectiveOrigin {
            x: livery::values::LengthPercentage::ZERO,
            y: livery::values::LengthPercentage::ZERO,
        }
    );
    assert_eq!(
        PerspectiveOrigin::CENTER.used(16.0, 200.0, 100.0),
        (100.0, 50.0)
    );
    assert_eq!(
        "25% 10px"
            .parse::<PerspectiveOrigin>()
            .expect("mixed")
            .to_string(),
        "25% 10px"
    );
    assert!("1px 2px 3px".parse::<PerspectiveOrigin>().is_err());

    assert_eq!(
        "preserve-3d".parse::<TransformStyle>().expect("preserve"),
        TransformStyle::Preserve3d
    );
    assert_eq!(
        "flat".parse::<TransformStyle>().expect("flat").to_string(),
        "flat"
    );
    assert!("preserve3d".parse::<TransformStyle>().is_err());

    assert_eq!(
        "hidden"
            .parse::<BackfaceVisibility>()
            .expect("hidden")
            .to_string(),
        "hidden"
    );
    assert!("collapse".parse::<BackfaceVisibility>().is_err());
}

#[test]
fn transform_origin_keeps_its_third_component() {
    let origin = "left top 20px"
        .parse::<TransformOrigin>()
        .expect("three components");
    assert_eq!(origin.z, livery::values::Length::px(20.0));
    assert_eq!(origin.to_string(), "0 0 20px");
    assert_eq!(origin.used_2d(16.0, 200.0, 100.0), (0.0, 0.0));
    assert_eq!(
        "50% 50% 0".parse::<TransformOrigin>().expect("zero z"),
        TransformOrigin::CENTER
    );
}

#[test]
fn matrix_helpers_report_affineness_depth_and_facing() {
    assert!(Matrix3D::IDENTITY.is_2d());
    assert!(Matrix3D::IDENTITY.is_affine());
    assert_eq!(Matrix3D::IDENTITY.front_facing_z(), 1.0);
    assert_eq!(Matrix3D::translation(0.0, 0.0, -30.0).origin_depth(), -30.0);
    // A half turn about Y turns the front face away from the viewer.
    let flipped = matrix("rotateY(180deg)");
    assert!(flipped.front_facing_z() < 0.0);
    assert!(matrix("rotateY(179deg)").front_facing_z() < 0.0);
    assert!(matrix("rotateY(1deg)").front_facing_z() > 0.0);
    // The affine projection drops Z rather than dividing by it.
    let projected = matrix("translate3d(4px, 5px, 6px)").to_affine_2d();
    assert_eq!((projected.e, projected.f), (4.0, 5.0));
}
