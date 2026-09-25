// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Intrinsic sizes for form controls, per the HTML rendering section's
//! "Form controls" and "The input element as a text entry widget". Livery's
//! only prior UA rule for these elements was
//! `button, input, select, textarea { display: inline-block; }`
//! (`components/genet-livery/src/lib.rs`), which left an unstyled control at
//! zero extent -- the root cause behind `tests/form_control_hit.rs`'s
//! 2026-09-16 pointer-placement regression. These tests check the sizes
//! themselves; the hit-test angle stays in `form_control_hit.rs`.

use genet_livery::{InteractionStates, LiveryLayout, StyleSet, layout, resolve_styles};
use livery::values::Size as CssSize;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::LayoutDom;
use livery::media::Device;

type Node = <ScriptedDom as LayoutDom>::NodeId;

fn layout_body(body: &str) -> (ScriptedDom, LiveryLayout<Node>) {
    let dom = ScriptedDom::from_serialized_document(&format!(
        "<!DOCTYPE html><html><head><title>sizing</title></head><body>{body}</body></html>"
    ));
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[]),
        &Device::screen(800.0, 600.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&dom, &styles, 800.0, 600.0).unwrap();
    (dom, fragments)
}

fn rect(dom: &ScriptedDom, fragments: &LiveryLayout<Node>, id: &str) -> (f32, f32, f32, f32) {
    fn find(dom: &ScriptedDom, node: Node, id: &str) -> Option<Node> {
        if dom
            .attributes(node)
            .any(|a| a.name.local.as_ref() == "id" && a.value == id)
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child, id))
    }
    let node = find(dom, dom.document(), id).unwrap();
    let fragment = fragments.get(node).unwrap();
    (fragment.x, fragment.y, fragment.width, fragment.height)
}

fn size(body: &str) -> (f32, f32) {
    let (dom, fragments) = layout_body(body);
    let (_, _, width, height) = rect(&dom, &fragments, "target");
    (width, height)
}

/// The natural size feeds Taffy directly (`apply_form_control_intrinsic_style`
/// in `components/genet-livery/src/layout.rs`) rather than the cascade, so
/// `computed.width`/`height` must stay `auto` regardless of `size`/`cols`/
/// `rows`, matching `html/rendering/widgets/input-text-size.html`'s "Size
/// attribute value is not a presentational hint" and
/// `.../textarea-cols-rows.html`'s equivalent -- both of which regressed
/// under an earlier, presentational-hint-based version of this lane and are
/// the reason this test exists. `getComputedStyle` itself is not available
/// (no DOM/script binding here), so this checks the same
/// `ComputedValues.width`/`height` CSSOM's resolved-value algorithm reads
/// for a `display: none` element (no box, no used value, falls back to the
/// computed value).
#[test]
fn size_cols_rows_never_reach_computed_style() {
    fn find(dom: &ScriptedDom, node: Node, id: &str) -> Option<Node> {
        if dom
            .attributes(node)
            .any(|a| a.name.local.as_ref() == "id" && a.value == id)
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child, id))
    }
    for element in [
        r#"<input id="target" size="17">"#,
        r#"<input id="target" style="display: none" size="17">"#,
        r#"<textarea id="target" cols="17" rows="9"></textarea>"#,
        r#"<textarea id="target" style="display: none" cols="17" rows="9"></textarea>"#,
        r#"<button id="target" style="display: none"></button>"#,
        r#"<select id="target" style="display: none"></select>"#,
    ] {
        let dom = ScriptedDom::from_serialized_document(&format!(
            "<!DOCTYPE html><html><body>{element}</body></html>"
        ));
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[]),
            &Device::screen(800.0, 600.0),
            &InteractionStates::default(),
        );
        let node = find(&dom, dom.document(), "target").unwrap();
        let computed = styles.get(node).unwrap();
        assert_eq!(computed.width, CssSize::Auto, "{element}");
        assert_eq!(computed.height, CssSize::Auto, "{element}");
    }
}

#[test]
fn text_entry_types_are_non_zero_and_default_to_twenty_characters() {
    for input in [
        r#"<input id="target">"#,
        r#"<input id="target" type="text">"#,
        r#"<input id="target" type="password">"#,
        r#"<input id="target" type="search">"#,
        r#"<input id="target" type="url">"#,
        r#"<input id="target" type="email">"#,
        r#"<input id="target" type="tel">"#,
        r#"<input id="target" type="number">"#,
        r#"<input id="target" type="not-a-real-type">"#,
    ] {
        let (width, height) = size(input);
        assert!(width > 0.0 && height > 0.0, "{input}: {width}x{height}");

        let (default_width, _) = size(input);
        let (wide_width, _) =
            size(&input.replace("id=\"target\"", "id=\"target\" size=\"40\""));
        assert!(
            wide_width > default_width,
            "{input}: size=40 ({wide_width}) should widen the default ({default_width})"
        );
        let (narrow_width, _) =
            size(&input.replace("id=\"target\"", "id=\"target\" size=\"5\""));
        assert!(
            narrow_width < default_width,
            "{input}: size=5 ({narrow_width}) should narrow the default ({default_width})"
        );
    }
}

#[test]
fn size_zero_is_invalid_and_falls_back_to_the_default() {
    let (default_width, _) = size(r#"<input id="target">"#);
    let (zero_width, _) = size(r#"<input id="target" size="0">"#);
    assert_eq!(zero_width, default_width, "size=0 is not a valid size");
}

#[test]
fn checkbox_and_radio_are_a_thirteen_pixel_square() {
    for input in [
        r#"<input id="target" type="checkbox">"#,
        r#"<input id="target" type="radio">"#,
        r#"<input id="target" type="CHECKBOX">"#,
    ] {
        let (width, height) = size(input);
        assert_eq!((width, height), (13.0, 13.0), "{input}");
    }
}

#[test]
fn button_like_controls_have_a_non_zero_shrink_to_fit_minimum() {
    for control in [
        r#"<input id="target" type="button">"#,
        r#"<input id="target" type="submit">"#,
        r#"<input id="target" type="reset">"#,
        r#"<button id="target"></button>"#,
        r#"<button id="target">press</button>"#,
    ] {
        let (width, height) = size(control);
        assert!(width > 0.0 && height > 0.0, "{control}: {width}x{height}");
    }

    // Shrink-to-fit: a labeled button is at least as wide as an empty one.
    let (empty_width, _) = size(r#"<button id="target"></button>"#);
    let (labeled_width, _) = size(r#"<button id="target">a rather long label</button>"#);
    assert!(
        labeled_width > empty_width,
        "empty {empty_width} vs labeled {labeled_width}"
    );

    // White space is no content: such a button keeps the empty one's floor.
    assert_eq!(
        size(r#"<button id="target"> </button>"#),
        size(r#"<button id="target"></button>"#)
    );
}

/// A button with content takes the automatic minimum in a shrinking flex
/// column (css-flexbox-1 4.5), with its padding and border inside it under the
/// UA's `border-box`. The floor used to stand in for that minimum as a
/// content-box length read as border-box, and the button fell to 22 of its 38.
#[test]
fn a_button_in_a_shrinking_flex_column_keeps_its_content_height() {
    let (dom, fragments) = layout_body(
        "<div style='display:flex; flex-direction:column; max-height:100px; overflow:auto; \
         width:300px; font-size:16px; line-height:20px'><span id=a>one</span>\
         <button id=b style='padding:8px; border:1px solid'>Refresh</button>\
         <div id=c style='height:300px'>tall</div></div>",
    );
    let heights: Vec<f32> = ["a", "b", "c"]
        .iter()
        .map(|id| rect(&dom, &fragments, id).3)
        .collect();
    assert_eq!(heights, [20.0, 38.0, 42.0]);
}

/// A text input's and a textarea's natural size is content-box, so an author's
/// `box-sizing: border-box` adds their padding and border rather than carving
/// them out of it.
#[test]
fn border_box_text_controls_keep_their_padding_and_border() {
    for (control, expected) in [
        (
            "<input id=target style='line-height:20px; padding:8px; border:1px solid; box-sizing:{}'>",
            (178.0, 38.0),
        ),
        (
            "<textarea id=target style='line-height:20px; padding:8px; border:1px solid; box-sizing:{}'></textarea>",
            (178.0, 58.0),
        ),
    ] {
        for sizing in ["content-box", "border-box"] {
            assert_eq!(
                size(&control.replace("{}", sizing)),
                expected,
                "{sizing}: {control}"
            );
        }
    }
}

#[test]
fn select_is_one_line_with_a_non_zero_width() {
    for select in [
        r#"<select id="target"></select>"#,
        r#"<select id="target"><option>one</option></select>"#,
    ] {
        let (width, height) = size(select);
        assert!(width > 0.0 && height > 0.0, "{select}: {width}x{height}");
    }
}

#[test]
fn textarea_defaults_to_twenty_columns_and_two_rows() {
    let (default_width, default_height) = size(r#"<textarea id="target"></textarea>"#);
    assert!(default_width > 0.0 && default_height > 0.0);

    let (wide_width, default_height_2) =
        size(r#"<textarea id="target" cols="40"></textarea>"#);
    assert_eq!(default_height_2, default_height, "cols does not change height");
    assert!(wide_width > default_width, "cols=40 should widen the default");

    let (default_width_2, tall_height) =
        size(r#"<textarea id="target" rows="8"></textarea>"#);
    assert_eq!(default_width_2, default_width, "rows does not change width");
    assert!(tall_height > default_height, "rows=8 should heighten the default");
}

#[test]
fn author_width_and_height_win_over_every_default_and_hint() {
    // `box-sizing: border-box` isolates the author `width`/`height` from the
    // UA stylesheet's own `padding` on button-like controls, so every case
    // here checks the same content-box-independent border-box number.
    for control in [
        r#"<input id="target" style="box-sizing: border-box; width: 321px; height: 55px">"#,
        r#"<input id="target" type="checkbox" style="box-sizing: border-box; width: 321px; height: 55px">"#,
        r#"<input id="target" type="button" style="box-sizing: border-box; width: 321px; height: 55px">"#,
        r#"<button id="target" style="box-sizing: border-box; width: 321px; height: 55px"></button>"#,
        r#"<select id="target" style="box-sizing: border-box; width: 321px; height: 55px"></select>"#,
        r#"<textarea id="target" cols="40" rows="8" style="box-sizing: border-box; width: 321px; height: 55px"></textarea>"#,
        r#"<input id="target" size="40" style="box-sizing: border-box; width: 321px; height: 55px">"#,
    ] {
        assert_eq!(size(control), (321.0, 55.0), "{control}");
    }
}

#[test]
fn display_block_input_does_not_collapse_to_a_line() {
    let (width, height) = size(r#"<input id="target" style="display: block">"#);
    assert!(width > 0.0 && height > 0.0, "{width}x{height}");
}

#[test]
fn hidden_inputs_stay_boxless() {
    let (dom, fragments) = layout_body(r#"<input id="target" type="hidden" value="x">"#);
    fn find(dom: &ScriptedDom, node: Node, id: &str) -> Option<Node> {
        if dom
            .attributes(node)
            .any(|a| a.name.local.as_ref() == "id" && a.value == id)
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child, id))
    }
    let node = find(&dom, dom.document(), "target").unwrap();
    assert!(
        fragments.get(node).is_none(),
        "input[type=hidden] should generate no fragment"
    );
}

#[test]
fn image_input_keeps_its_natural_size_path_untouched() {
    // `type=image` is a replaced element sized from its natural image size
    // (`apply_replaced_intrinsic_style`), not a text-entry widget; sizing it
    // when it has no natural size is a separate, pre-existing gap outside
    // this lane (only `img`/`canvas` are recognized natural-size sources).
    // The point here is narrower: the new text-entry size hint must not fire
    // for it and clobber `width`/`height` staying `auto`, which is what that
    // path depends on to reach for a natural size at all.
    fn find(dom: &ScriptedDom, node: Node, id: &str) -> Option<Node> {
        if dom
            .attributes(node)
            .any(|a| a.name.local.as_ref() == "id" && a.value == id)
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child, id))
    }
    let dom = ScriptedDom::from_serialized_document(
        "<!DOCTYPE html><html><body><input id=\"target\" type=\"image\"></body></html>",
    );
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[]),
        &Device::screen(800.0, 600.0),
        &InteractionStates::default(),
    );
    let node = find(&dom, dom.document(), "target").unwrap();
    let computed = styles.get(node).unwrap();
    assert_eq!(computed.width, CssSize::Auto);
    assert_eq!(computed.height, CssSize::Auto);
}
