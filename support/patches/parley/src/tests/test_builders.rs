// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test that the various builders produce the same results.

use std::{borrow::Cow, sync::Arc};

use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth, SourceCache};
use parlance::FontFamilyName;
use peniko::{Blob, color::palette};

use super::utils::{ColorBrush, asserts::assert_eq_layout_data};
use crate::{
    FontContext, FontFamily, FontFeatures, FontVariations, InlineBox, InlineBoxKind, Layout,
    LayoutContext, LineHeight, OverflowWrap, RangedBuilder, StyleProperty, StyleRunBuilder,
    TextStyle, TextWrapMode, TreeBuilder, WhiteSpaceCollapse, WordBreak,
};

// These tests compare builder output, not a particular font's metrics. Use
// the repository's deterministic Latin fixtures instead of parley_dev's
// unavailable upstream font collection.
const FONT_FAMILY_LIST: &[FontFamilyName<'_>] = &[FontFamilyName::Named(Cow::Borrowed("Lato"))];

fn create_font_context() -> FontContext {
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: false,
    });
    for bytes in [
        include_bytes!("../../../../../tests/wpt/tests/fonts/Lato-Medium.ttf").as_slice(),
        include_bytes!("../../../../../tests/wpt/tests/fonts/Lato-Bold.ttf").as_slice(),
    ] {
        collection.register_fonts(Blob::new(Arc::new(bytes.to_vec())), None);
    }
    for font in FONT_FAMILY_LIST {
        if let FontFamilyName::Named(font_name) = font {
            collection
                .family_id(font_name)
                .unwrap_or_else(|| panic!("{font_name} font not found"));
        }
    }
    FontContext {
        collection,
        source_cache: SourceCache::default(),
    }
}

/// Set of options for [`build_layout_with_ranged`].
struct RangedOptions<'a> {
    scale: f32,
    quantize: bool,
    max_advance: Option<f32>,
    text: &'a str,
}

/// Set of options for [`build_layout_with_tree`].
struct TreeOptions<'a, 'b> {
    scale: f32,
    quantize: bool,
    max_advance: Option<f32>,
    root_style: &'a TextStyle<'b, 'b, ColorBrush>,
}

/// Generates a `Layout` with a ranged builder.
fn build_layout_with_ranged(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<ColorBrush>,
    opts: &RangedOptions<'_>,
    with_builder: impl Fn(&mut RangedBuilder<'_, ColorBrush>),
) -> Layout<ColorBrush> {
    let mut rb = lcx.ranged_builder(fcx, opts.text, opts.scale, opts.quantize);
    with_builder(&mut rb);
    let mut layout = rb.build(opts.text);
    layout.break_all_lines(opts.max_advance);
    layout
}

/// Generates a `Layout` with a tree builder.
fn build_layout_with_tree(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<ColorBrush>,
    opts: &TreeOptions<'_, '_>,
    with_builder: impl Fn(&mut TreeBuilder<'_, ColorBrush>),
) -> Layout<ColorBrush> {
    let mut tb = lcx.tree_builder(fcx, opts.scale, opts.quantize, opts.root_style);
    with_builder(&mut tb);
    let (mut layout, _) = tb.build();
    layout.break_all_lines(opts.max_advance);
    layout
}

/// Generates a `Layout` with a style run builder.
fn build_layout_with_style_runs(
    fcx: &mut FontContext,
    lcx: &mut LayoutContext<ColorBrush>,
    opts: &RangedOptions<'_>,
    with_builder: impl Fn(&mut StyleRunBuilder<'_, ColorBrush>),
) -> Layout<ColorBrush> {
    let mut rb = lcx.style_run_builder(fcx, opts.text, opts.scale, opts.quantize);
    with_builder(&mut rb);
    let mut layout = rb.build(opts.text);
    layout.break_all_lines(opts.max_advance);
    layout
}

/// Computes layout in various ways to ensure they all produce the same result.
///
/// ```text
/// LayoutContext A - Ranged
/// LayoutContext A - Ranged for idempotency
///
/// LayoutContext B - Tree
/// LayoutContext B - Tree for idempotency
///
/// LayoutContext C - Ranged for dirt
/// LayoutContext C - Tree from dirty
///
/// LayoutContext D - Tree for dirt
/// LayoutContext D - Ranged from dirty
/// ```
fn assert_builders_produce_same_result<'b>(
    text: &str,
    scale: f32,
    quantize: bool,
    max_advance: Option<f32>,
    root_style: &TextStyle<'b, 'b, ColorBrush>,
    with_ranged_builder: impl Fn(&mut RangedBuilder<'_, ColorBrush>),
    with_tree_builder: impl Fn(&mut TreeBuilder<'_, ColorBrush>),
    expect_empty: bool,
) {
    let mut fcx = create_font_context();

    let mut lcx_a: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut lcx_b: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut lcx_c: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut lcx_d: LayoutContext<ColorBrush> = LayoutContext::new();

    let ropts = RangedOptions {
        scale,
        quantize,
        max_advance,
        text,
    };
    let topts = TreeOptions {
        scale,
        quantize,
        max_advance,
        root_style,
    };

    // Source of truth - ranged builder from a clean layout context
    let layout_truth = build_layout_with_ranged(&mut fcx, &mut lcx_a, &ropts, &with_ranged_builder);
    assert!(
        layout_truth.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_a_rb_one"
    );

    // Testing idempotence of ranged builder creation
    let layout = build_layout_with_ranged(&mut fcx, &mut lcx_a, &ropts, &with_ranged_builder);
    assert!(
        layout.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_a_rb_two"
    );
    assert_eq_layout_data(&layout_truth.data, &layout.data, "lcx_a_rb_two");

    // Basic builder compatibility - tree builder from a clean layout context
    let layout = build_layout_with_tree(&mut fcx, &mut lcx_b, &topts, &with_tree_builder);
    assert!(
        layout.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_b_tb_one"
    );
    assert_eq_layout_data(&layout_truth.data, &layout.data, "lcx_b_tb_one");

    // Testing idempotence of tree builder creation
    let layout = build_layout_with_tree(&mut fcx, &mut lcx_b, &topts, &with_tree_builder);
    assert!(
        layout.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_b_tb_two"
    );
    assert_eq_layout_data(&layout_truth.data, &layout.data, "lcx_b_tb_two");

    // Priming a fresh layout context with ranged builder creation
    let _ = build_layout_with_ranged(&mut fcx, &mut lcx_c, &ropts, &with_ranged_builder);

    // Testing tree builder creation with a dirty layout context
    let layout = build_layout_with_tree(&mut fcx, &mut lcx_c, &topts, &with_tree_builder);
    assert!(
        layout.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_c_tb_one"
    );
    assert_eq_layout_data(&layout_truth.data, &layout.data, "lcx_c_tb_one");

    // Priming a fresh layout context with tree builder creation
    let _ = build_layout_with_tree(&mut fcx, &mut lcx_d, &topts, &with_tree_builder);

    // Testing ranged builder creation with a dirty layout context
    let layout = build_layout_with_ranged(&mut fcx, &mut lcx_d, &ropts, &with_ranged_builder);
    assert!(
        layout.data.runs.is_empty() == expect_empty,
        "expected runs to exist for lcx_d_rb_one"
    );
    assert_eq_layout_data(&layout_truth.data, &layout.data, "lcx_d_rb_one");
}

/// Returns a root style that uses non-default values.
///
/// The [`TreeBuilder`] version of [`set_root_style`].
fn create_root_style() -> TextStyle<'static, 'static, ColorBrush> {
    TextStyle {
        font_family: FontFamily::from(FONT_FAMILY_LIST),
        font_size: 20.,
        font_width: FontWidth::CONDENSED,
        font_style: FontStyle::Italic,
        font_weight: FontWeight::BOLD,
        font_variations: FontVariations::empty(), // TODO: Set a non-default value
        font_features: FontFeatures::empty(),     // TODO: Set a non-default value
        locale: Some("en-US".parse().unwrap()),
        brush: ColorBrush::new(palette::css::GREEN),
        has_underline: true,
        underline_offset: Some(2.),
        underline_size: Some(3.5),
        underline_brush: Some(ColorBrush::new(palette::css::CYAN)),
        has_strikethrough: true,
        strikethrough_offset: Some(1.3),
        strikethrough_size: Some(1.7),
        strikethrough_brush: Some(ColorBrush::new(palette::css::BEIGE)),
        line_height: LineHeight::Absolute(30.),
        word_spacing: 2.,
        letter_spacing: 1.5,
        tab_size: 64.0,
        word_break: WordBreak::BreakAll,
        overflow_wrap: OverflowWrap::Anywhere,
        text_wrap_mode: TextWrapMode::Wrap,
    }
}

/// Sets a root style with non-default values.
///
/// The [`RangedBuilder`] version of [`create_root_style`].
fn set_root_style(rb: &mut RangedBuilder<'_, ColorBrush>) {
    rb.push_default(FontFamily::from(FONT_FAMILY_LIST));
    rb.push_default(StyleProperty::FontSize(20.));
    rb.push_default(StyleProperty::FontWidth(FontWidth::CONDENSED));
    rb.push_default(StyleProperty::FontStyle(FontStyle::Italic));
    rb.push_default(StyleProperty::FontWeight(FontWeight::BOLD));
    rb.push_default(FontVariations::empty());
    rb.push_default(FontFeatures::empty());
    rb.push_default(StyleProperty::Locale(Some("en-US".parse().unwrap())));
    rb.push_default(StyleProperty::Brush(ColorBrush::new(palette::css::GREEN)));
    rb.push_default(StyleProperty::Underline(true));
    rb.push_default(StyleProperty::UnderlineOffset(Some(2.)));
    rb.push_default(StyleProperty::UnderlineSize(Some(3.5)));
    rb.push_default(StyleProperty::UnderlineBrush(Some(ColorBrush::new(
        palette::css::CYAN,
    ))));
    rb.push_default(StyleProperty::Strikethrough(true));
    rb.push_default(StyleProperty::StrikethroughOffset(Some(1.3)));
    rb.push_default(StyleProperty::StrikethroughSize(Some(1.7)));
    rb.push_default(StyleProperty::StrikethroughBrush(Some(ColorBrush::new(
        palette::css::BEIGE,
    ))));
    rb.push_default(LineHeight::Absolute(30.));
    rb.push_default(StyleProperty::WordSpacing(2.));
    rb.push_default(StyleProperty::LetterSpacing(1.5));
    rb.push_default(StyleProperty::WordBreak(WordBreak::BreakAll));
    rb.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
}

/// Test that all the builders have the same default behavior.
#[test]
fn builders_default() {
    let text = "Builders often wear hard hats for safety while working on construction sites.";
    let scale = 2.;
    let quantize = false;
    let max_advance = Some(50.);
    let root_style = TextStyle {
        font_family: FontFamily::from(FONT_FAMILY_LIST),
        ..TextStyle::default()
    };

    let with_ranged_builder = |rb: &mut RangedBuilder<'_, ColorBrush>| {
        rb.push_default(FontFamily::from(FONT_FAMILY_LIST));
    };
    let with_tree_builder = |tb: &mut TreeBuilder<'_, ColorBrush>| {
        tb.push_text(text);
    };

    assert_builders_produce_same_result(
        text,
        scale,
        quantize,
        max_advance,
        &root_style,
        with_ranged_builder,
        with_tree_builder,
        false,
    );
}

/// Test that `StyleRunBuilder` produces the same result as `RangedBuilder` when given equivalent
/// styles.
#[test]
fn builders_style_runs_match_ranged() {
    let text = "Builders often wear hard hats.";
    let scale = 2.;
    let quantize = false;
    let max_advance = Some(120.);

    let root_style: TextStyle<'static, 'static, ColorBrush> = TextStyle {
        font_family: FontFamily::from(FONT_FAMILY_LIST),
        ..TextStyle::default()
    };

    let split = text.len() / 2;
    let mut modified_style = root_style.clone();
    modified_style.font_size = 40.;
    modified_style.letter_spacing = 1.25;

    let mut fcx = create_font_context();
    let mut lcx_a: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut lcx_b: LayoutContext<ColorBrush> = LayoutContext::new();

    let ropts = RangedOptions {
        scale,
        quantize,
        max_advance,
        text,
    };

    let ranged = build_layout_with_ranged(&mut fcx, &mut lcx_a, &ropts, |rb| {
        rb.push_default(FontFamily::from(FONT_FAMILY_LIST));
        rb.push(
            StyleProperty::FontSize(modified_style.font_size),
            split..text.len(),
        );
        rb.push(
            StyleProperty::LetterSpacing(modified_style.letter_spacing),
            split..text.len(),
        );
    });

    let runs = build_layout_with_style_runs(&mut fcx, &mut lcx_b, &ropts, |rb| {
        let family: FontFamily<'static> = root_style.font_family.clone().into_owned();
        let root_run: TextStyle<'static, 'static, ColorBrush> = TextStyle {
            font_family: family.clone(),
            ..root_style.clone()
        };

        let modified_run: TextStyle<'static, 'static, ColorBrush> = TextStyle {
            font_family: family,
            ..modified_style.clone()
        };

        let root_index = rb.push_style(root_run);
        let modified_index = rb.push_style(modified_run);
        rb.push_style_run(root_index, 0..split);
        rb.push_style_run(modified_index, split..text.len());
    });

    assert_eq_layout_data(&ranged.data, &runs.data, "style_runs_match_ranged");
}

/// Test that `StyleRunBuilder` handles a first run whose style table index is not zero.
#[test]
fn style_runs_first_run_can_use_nonzero_style_index() {
    let text = "Builders often wear hard hats.";
    let scale = 2.;
    let quantize = false;
    let max_advance = Some(50.);
    let root_style = create_root_style();
    let mut modified_style = root_style.clone();
    modified_style.font_size = 40.;

    let mut fcx = create_font_context();
    let mut lcx_a: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut lcx_b: LayoutContext<ColorBrush> = LayoutContext::new();

    let ropts = RangedOptions {
        scale,
        quantize,
        max_advance,
        text,
    };

    let ranged = build_layout_with_ranged(&mut fcx, &mut lcx_a, &ropts, |rb| {
        set_root_style(rb);
        rb.push(
            StyleProperty::FontSize(modified_style.font_size),
            0..text.len(),
        );
    });

    let runs = build_layout_with_style_runs(&mut fcx, &mut lcx_b, &ropts, |rb| {
        let family: FontFamily<'static> = root_style.font_family.clone().into_owned();
        let root_run: TextStyle<'static, 'static, ColorBrush> = TextStyle {
            font_family: family.clone(),
            ..root_style.clone()
        };

        let modified_run: TextStyle<'static, 'static, ColorBrush> = TextStyle {
            font_family: family,
            ..modified_style.clone()
        };

        let _root_index = rb.push_style(root_run);
        let modified_index = rb.push_style(modified_run);
        rb.push_style_run(modified_index, 0..text.len());
    });

    assert_eq_layout_data(
        &ranged.data,
        &runs.data,
        "style_runs_first_run_can_use_nonzero_style_index",
    );
}

/// Test that all the builders behave the same when given the same root style.
#[test]
fn builders_root_only() {
    let text = "Builders often wear hard hats for safety while working on construction sites.";
    let scale = 2.;
    let quantize = false;
    let max_advance = Some(50.);
    let root_style = create_root_style();

    let with_ranged_builder = |rb: &mut RangedBuilder<'_, ColorBrush>| {
        set_root_style(rb);
    };
    let with_tree_builder = |tb: &mut TreeBuilder<'_, ColorBrush>| {
        tb.push_text(text);
    };

    assert_builders_produce_same_result(
        text,
        scale,
        quantize,
        max_advance,
        &root_style,
        with_ranged_builder,
        with_tree_builder,
        false,
    );
}

/// Test that an empty layout doesn't crash
#[test]
fn builders_empty() {
    let text = "";
    let scale = 1.;
    let quantize = false;
    let max_advance = Some(50.);
    let root_style = TextStyle::default();

    let with_ranged_builder = |_rb: &mut RangedBuilder<'_, ColorBrush>| {};
    let with_tree_builder = |_tb: &mut TreeBuilder<'_, ColorBrush>| {};

    assert_builders_produce_same_result(
        text,
        scale,
        quantize,
        max_advance,
        &root_style,
        with_ranged_builder,
        with_tree_builder,
        true,
    );
}

/// Empty tree layouts retain the resolved root style, including scaled tab stops.
#[test]
fn empty_tree_layout_keeps_custom_root_style() {
    let mut fcx = create_font_context();
    let mut lcx: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut prior = lcx.tree_builder(&mut fcx, 1.0, false, &TextStyle::default());
    prior.push_text("warmup");
    let _ = prior.build();

    let mut root_style = create_root_style();
    root_style.tab_size = 37.0;
    let mut builder = lcx.tree_builder(&mut fcx, 2.0, false, &root_style);
    builder.set_white_space_mode(WhiteSpaceCollapse::Collapse);
    let child_style = [StyleProperty::TabSize(9.0)];
    builder.push_style_modification_span(&child_style);
    builder.push_text("   ");
    builder.pop_style_span();
    let (mut layout, text) = builder.build();

    assert!(text.is_empty());
    assert!(!layout.data.runs.is_empty());
    assert_eq!(layout.data.styles.len(), 1);
    assert_eq!(layout.data.styles[0].tab_size, 74.0);
    assert_eq!(layout.data.styles[0].brush, root_style.brush);
    layout.break_all_lines(Some(1.0));
    assert_eq!(layout.get(0).unwrap().metrics().line_height, 60.0);
    assert_eq!(layout.height(), 60.0);
}

#[test]
fn empty_tree_text_with_inline_box_keeps_root_and_box_metrics() {
    let mut fcx = create_font_context();
    let mut lcx: LayoutContext<ColorBrush> = LayoutContext::new();
    let mut root_style = TextStyle::default();
    root_style.tab_size = 29.0;
    let inline_box = InlineBox {
        id: 1,
        kind: InlineBoxKind::InFlow,
        index: 0,
        width: 12.0,
        height: 7.0,
    };
    let mut builder = lcx.tree_builder(&mut fcx, 1.0, false, &root_style);
    builder.push_inline_box(inline_box.clone());
    let (mut layout, text) = builder.build();

    assert!(text.is_empty());
    assert_eq!(layout.data.styles.len(), 1);
    assert_eq!(layout.data.styles[0].tab_size, 29.0);
    assert_eq!(layout.inline_boxes(), &[inline_box]);
    layout.break_all_lines(Some(50.0));
    let metrics = *layout.get(0).unwrap().metrics();
    assert_eq!(metrics.advance, 12.0);
    assert_eq!(metrics.ascent, 7.0);
}

/// Test that all the builders behave the same with mixed styles.
#[test]
fn builders_mixed_styles() {
    let text = "Builders often wear hard hats for safety while working on construction sites.";
    let scale = 2.;
    let quantize = false;
    let max_advance = Some(50.);
    let root_style = create_root_style();

    let with_ranged_builder = |rb: &mut RangedBuilder<'_, ColorBrush>| {
        set_root_style(rb);

        // Make the first word bigger
        rb.push(StyleProperty::FontSize(68.), 0..8);
        // Push two modified styles for the same range
        rb.push(StyleProperty::LetterSpacing(4.), 12..17);
        rb.push(StyleProperty::WordSpacing(3.), 12..17);
        // Plus, change the line height for the last letter
        rb.push(StyleProperty::LineHeight(LineHeight::Absolute(40.)), 16..17);
    };
    let with_tree_builder = |tb: &mut TreeBuilder<'_, ColorBrush>| {
        // Make the first word bigger
        tb.push_style_modification_span(&[StyleProperty::FontSize(68.)]);
        tb.push_text(&text[..8]);
        tb.pop_style_span();

        tb.push_text(&text[8..12]);

        // Push two modified styles in batch
        tb.push_style_modification_span(&[
            StyleProperty::LetterSpacing(4.),
            StyleProperty::WordSpacing(3.),
        ]);
        tb.push_text(&text[12..16]);
        // Plus, change the line height for the last letter
        tb.push_style_modification_span(&[StyleProperty::LineHeight(LineHeight::Absolute(40.))]);
        tb.push_text(&text[16..17]);
        tb.pop_style_span();
        tb.pop_style_span();

        tb.push_text(&text[17..]);
    };

    assert_builders_produce_same_result(
        text,
        scale,
        quantize,
        max_advance,
        &root_style,
        with_ranged_builder,
        with_tree_builder,
        false,
    );
}
