// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! T1: lowering CSS Transforms Level 2 into the shared paint/hit-test matrix,
//! the `preserve-3d` depth sort, the backface cull and the perspective gap.

use genet_livery::{
    Device, InteractionStates, LiveryPaintList, StyleSet, TransformGapKind, emit_paint_list,
    layout, resolve_styles,
};
use genet_static_dom::StaticDocument;
use paint_list_api::{DeviceIntSize, LayoutTransform, PaintCmd, PaintList};

const RESET: &str = "html, body { margin: 0; padding: 0; } div { position: absolute; \
                     width: 40px; height: 40px; }";

fn render(html: &str, css: &str) -> LiveryPaintList {
    let document = StaticDocument::parse(html);
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[&format!("{RESET} {css}")]),
        &Device::screen(320.0, 240.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, 320.0, 240.0).expect("layout");
    emit_paint_list(
        &document,
        &styles,
        &fragments,
        DeviceIntSize::new(320, 240),
        1,
    )
}

/// Every pushed transform, in paint order, folded back to the full matrix the
/// renderer sees: the spec's matrix followed by its placement origin.
fn transforms(list: &LiveryPaintList) -> Vec<LayoutTransform> {
    list.commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::PushTransform(spec) => {
                let mut matrix = spec.transform;
                matrix.m41 += spec.origin.x;
                matrix.m42 += spec.origin.y;
                Some(matrix)
            },
            _ => None,
        })
        .collect()
}

/// The fill colors in paint order, which is the z order of the boxes.
fn fill_order(list: &LiveryPaintList) -> Vec<(u8, u8, u8)> {
    list.commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawRect(item) if item.color.a > 0.0 => Some((
                (item.color.r * 255.0).round() as u8,
                (item.color.g * 255.0).round() as u8,
                (item.color.b * 255.0).round() as u8,
            )),
            _ => None,
        })
        .collect()
}

fn close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "{what}: {actual} != {expected}"
    );
}

#[test]
fn a_three_d_transform_reaches_the_paint_matrix_as_a_full_four_by_four() {
    // Inside a 3D rendering context the whole 4x4 is pushed. Outside one the
    // same transform is flattened into its parent's plane, which is the case
    // the second half of this test covers.
    let list = render(
        "<html><body><div id=p><div id=a></div></div></body></html>",
        "#p { transform-style: preserve-3d; } \
         #a { left: 10px; top: 20px; transform-origin: 0 0; \
         transform: translate3d(1px, 2px, 3px) rotateY(90deg); }",
    );
    let [matrix] = transforms(&list).try_into().expect("one transform");
    // rotateY(90deg) takes the local X axis onto -Z, which the 4x4 must carry.
    close(matrix.m11, 0.0, "m11");
    close(matrix.m13, -1.0, "m13");
    close(matrix.m31, 1.0, "m31");
    // transform-origin 0 0 is the border-box corner at (10, 20). The origin
    // round trip carries that corner through the rotated axes, so the Y offset
    // stays the authored 2px and the corner's X becomes depth.
    close(matrix.m41, 11.0, "m41");
    close(matrix.m42, 2.0, "m42");
    close(matrix.m43, 13.0, "m43");

    // The same box under a flat parent is projected: the Z row and column go
    // back to the identity and only the planar six cells survive.
    let flat = render(
        "<html><body><div id=a></div></body></html>",
        "#a { left: 10px; top: 20px; transform-origin: 0 0; \
         transform: translate3d(1px, 2px, 3px) rotateY(90deg); }",
    );
    let [flat] = transforms(&flat).try_into().expect("one transform");
    assert_eq!((flat.m13, flat.m31, flat.m43), (0.0, 0.0, 0.0));
    assert_eq!(flat.m33, 1.0);
    close(flat.m11, 0.0, "flat m11");
    close(flat.m41, 11.0, "flat m41");
    close(flat.m42, 2.0, "flat m42");
}

#[test]
fn a_flat_parent_flattens_its_child_instead_of_composing_in_three_d() {
    // Two nested rotateX(45deg) under the default `flat` compose as two
    // projected foreshortenings, not as one rotateX(90deg) that would
    // collapse the box to nothing.
    let list = render(
        "<html><body><div id=p><div id=c></div></div></body></html>",
        "#p { transform-origin: 0 0; transform: rotateX(45deg); } \
         #c { transform-origin: 0 0; transform: rotateX(45deg); }",
    );
    let pushed = transforms(&list);
    assert_eq!(pushed.len(), 2);
    let cos = 45.0_f32.to_radians().cos();
    for matrix in &pushed {
        close(matrix.m22, cos, "m22");
        assert_eq!(matrix.m23, 0.0, "the Z column survived a flat boundary");
        assert_eq!(matrix.m32, 0.0, "the Z row survived a flat boundary");
    }
    // Under `preserve-3d` the same pair keeps its Z terms and composes to a
    // half turn about X.
    let preserved = render(
        "<html><body><div id=p><div id=c></div></div></body></html>",
        "#p { transform-origin: 0 0; transform: rotateX(45deg); \
         transform-style: preserve-3d; } \
         #c { transform-origin: 0 0; transform: rotateX(45deg); }",
    );
    let preserved = transforms(&preserved);
    assert_eq!(preserved.len(), 2);
    let sin = 45.0_f32.to_radians().sin();
    close(preserved[0].m23, sin, "preserved parent m23");
    close(preserved[1].m23, sin, "preserved child m23");
}

#[test]
fn individual_properties_apply_before_transform_in_the_spec_order() {
    // translate, then rotate, then scale, then `transform`. With the origin at
    // the box corner and a quarter turn, the order is visible in m41/m42: the
    // translation is not rotated, and the transform list is.
    let list = render(
        "<html><body><div id=a></div></body></html>",
        "#a { left: 0; top: 0; transform-origin: 0 0; translate: 10px 0; rotate: 90deg; \
         transform: translate(5px, 0); }",
    );
    let [matrix] = transforms(&list).try_into().expect("one transform");
    close(matrix.m11, 0.0, "m11");
    close(matrix.m12, 1.0, "m12");
    // translate(10, 0) then rotate 90deg then translate(5, 0): the inner 5px
    // travels down the rotated X axis.
    close(matrix.m41, 10.0, "m41");
    close(matrix.m42, 5.0, "m42");

    // The reverse authored order gives a different matrix, so the order is a
    // real rule here and not an accident of composition.
    let reversed = render(
        "<html><body><div id=a></div></body></html>",
        "#a { left: 0; top: 0; transform-origin: 0 0; \
         transform: rotate(90deg) translate(10px, 0) translate(5px, 0); }",
    );
    let [reversed] = transforms(&reversed).try_into().expect("one transform");
    close(reversed.m41, 0.0, "reversed m41");
    close(reversed.m42, 15.0, "reversed m42");
}

#[test]
fn an_individual_rotate_moves_alone_while_the_parent_matrix_is_unchanged() {
    // T1's yaw receipt in miniature: the parent's pushed matrix is identical
    // across frames while the child's rotate changes.
    let mut parents = Vec::new();
    let mut children = Vec::new();
    for yaw in ["0deg", "20deg", "40deg"] {
        let list = render(
            "<html><body><div id=p><div id=c></div></div></body></html>",
            &format!(
                "#p {{ left: 10px; top: 10px; transform-origin: 0 0; \
                 transform: translate3d(30px, 0, 0); transform-style: preserve-3d; }} \
                 #c {{ left: 0; top: 0; transform-origin: 0 0; rotate: y {yaw}; }}"
            ),
        );
        let pushed = transforms(&list);
        assert_eq!(pushed.len(), 2, "parent and child transforms");
        parents.push(pushed[0]);
        children.push(pushed[1]);
    }
    assert_eq!(parents[0], parents[1], "parent matrix rewritten");
    assert_eq!(parents[1], parents[2], "parent matrix rewritten");
    assert_ne!(children[0], children[1], "child yaw did not move");
    assert_ne!(children[1], children[2], "child yaw did not move");
}

#[test]
fn a_preserve_3d_context_paints_its_boxes_in_transformed_depth_order() {
    let html = "<html><body><div id=p><div id=near></div><div id=far></div></div></body></html>";
    // `near` is in front, so it paints last whichever way the DOM orders them.
    let front = render(
        html,
        "#p { transform-style: preserve-3d; } \
         #near { background: rgb(255, 0, 0); transform: translateZ(50px); } \
         #far { background: rgb(0, 0, 255); transform: translateZ(-50px); }",
    );
    assert_eq!(fill_order(&front), vec![(0, 0, 255), (255, 0, 0)]);

    // Flip the rotation of the context and the order reverses with it.
    let flipped = render(
        html,
        "#p { transform-style: preserve-3d; transform-origin: 0 0; transform: rotateY(180deg); } \
         #near { background: rgb(255, 0, 0); transform: translateZ(50px); } \
         #far { background: rgb(0, 0, 255); transform: translateZ(-50px); }",
    );
    assert_eq!(fill_order(&flipped), vec![(255, 0, 0), (0, 0, 255)]);

    // Without `preserve-3d` the context flattens and DOM order stands.
    let flat = render(
        html,
        "#near { background: rgb(255, 0, 0); transform: translateZ(50px); } \
         #far { background: rgb(0, 0, 255); transform: translateZ(-50px); }",
    );
    assert_eq!(fill_order(&flat), vec![(255, 0, 0), (0, 0, 255)]);
}

#[test]
fn backface_visibility_hidden_culls_a_box_whose_normal_turned_away() {
    let html = "<html><body><div id=a></div></body></html>";
    let visible = render(
        html,
        "#a { background: rgb(255, 0, 0); backface-visibility: hidden; \
         transform: rotateY(20deg); }",
    );
    assert_eq!(fill_order(&visible), vec![(255, 0, 0)]);

    let hidden = render(
        html,
        "#a { background: rgb(255, 0, 0); backface-visibility: hidden; \
         transform: rotateY(160deg); }",
    );
    assert!(fill_order(&hidden).is_empty(), "the backface was painted");

    // Without the property the same box still paints.
    let kept = render(
        html,
        "#a { background: rgb(255, 0, 0); transform: rotateY(160deg); }",
    );
    assert_eq!(fill_order(&kept), vec![(255, 0, 0)]);

    // An ancestor's rotation turns a child's face away too: the cull reads
    // the accumulated matrix, not the element's own.
    let inherited = render(
        "<html><body><div id=p><div id=c></div></div></body></html>",
        "#p { transform-origin: 0 0; transform: rotateY(180deg); transform-style: preserve-3d; } \
         #c { background: rgb(255, 0, 0); backface-visibility: hidden; }",
    );
    assert!(
        fill_order(&inherited).is_empty(),
        "an inherited half turn did not cull the child"
    );
}

#[test]
fn a_perspective_divide_is_reported_as_a_named_gap_and_approximated() {
    // The `perspective()` function inside the list.
    let list = render(
        "<html><body><div id=a></div></body></html>",
        "#a { background: rgb(0, 255, 0); transform-origin: 0 0; \
         transform: perspective(400px) translateZ(100px); }",
    );
    let gaps = list.transform_gaps();
    assert_eq!(gaps.len(), 1, "expected one named gap, got {gaps:?}");
    assert_eq!(gaps[0].kind, TransformGapKind::PerspectiveDivide);
    close(gaps[0].projection[2], -1.0 / 400.0, "m34");
    assert!(gaps[0].message().contains("orthographic"));
    // The affine approximation is still painted, so the box is not lost.
    assert_eq!(fill_order(&list), vec![(0, 255, 0)]);

    // The `perspective` property on the parent reaches the child's matrix.
    let from_property = render(
        "<html><body><div id=p><div id=c></div></div></body></html>",
        "#p { perspective: 400px; perspective-origin: 0 0; } \
         #c { transform-origin: 0 0; transform: translateZ(100px); }",
    );
    let gaps = from_property.transform_gaps();
    assert_eq!(gaps.len(), 1, "expected one named gap, got {gaps:?}");
    close(gaps[0].projection[2], -1.0 / 400.0, "property m34");

    // An affine 3D transform reports nothing.
    let orthographic = render(
        "<html><body><div id=a></div></body></html>",
        "#a { transform: rotateY(30deg) translateZ(20px); }",
    );
    assert!(orthographic.transform_gaps().is_empty());
}

#[test]
fn orthographic_three_d_is_exact_in_the_projected_cells() {
    // The wing's case: no perspective, so the planar six cells netrender reads
    // are the exact orthographic projection of the 4x4, not an approximation.
    let list = render(
        "<html><body><div id=a></div></body></html>",
        "#a { left: 0; top: 0; transform-origin: 0 0; \
         transform: rotateY(60deg) translate3d(10px, 20px, 30px); }",
    );
    let [matrix] = transforms(&list).try_into().expect("one transform");
    let cos = 60.0_f32.to_radians().cos();
    let sin = 60.0_f32.to_radians().sin();
    close(matrix.m11, cos, "m11");
    close(matrix.m12, 0.0, "m12");
    close(matrix.m21, 0.0, "m21");
    close(matrix.m22, 1.0, "m22");
    // The X and Z translations both project onto the screen X axis.
    close(matrix.m41, 10.0 * cos + 30.0 * sin, "m41");
    close(matrix.m42, 20.0, "m42");
    assert_eq!(matrix.m14, 0.0);
    assert_eq!(matrix.m44, 1.0);
    assert!(list.transform_gaps().is_empty());
}
