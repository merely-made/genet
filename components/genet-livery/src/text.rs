// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Clean-room Parley adapter for Livery inline formatting.

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
    ops::Range,
    sync::Arc,
};

use buckram::{
    BoxId, BoxOrigin, CssBoxTree, DisplayInside, DisplayOutside, FloatLineConstraints,
    FormattingContextKind, InternalTableRole, IntrinsicSizeKind, IntrinsicSizes,
};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind, QuirksMode};
use livery::{
    ComputedValues,
    values::{
        Direction, Display, FontFamily as CssFontFamily, FontFeatureSetting,
        FontFeatureSettings as CssFontFeatureSettings, FontStyle as CssFontStyle,
        FontWeight as CssFontWeight, Hyphens, LineBreak as CssLineBreak,
        LineHeight as CssLineHeight, ListStylePosition, ListStyleType, Margin,
        OverflowWrap as CssOverflowWrap, Position, Spacing, TabSize, TextAlign, TextAlignLast,
        TextJustify, TextTransformCase, TextWrapMode, VerticalAlign, WordBreak as CssWordBreak,
    },
};
use paint_list_api::{
    ColorF, CommonPlacement, FontInstanceKey, FontResource, GlyphInstance, IdNamespace,
    LayoutPoint, PaintCmd, TextOptions, TextRunItem,
};
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, FontFeature, FontFeatures, FontStyle,
    FontWeight, GenericFamily, IndentOptions, InlineBox, InlineBoxKind, LayoutContext,
    OverflowWrap as ParleyOverflowWrap, PositionedLayoutItem, StyleProperty,
    TextWrapMode as ParleyTextWrapMode, WordBreak as ParleyWordBreak,
    layout::{BreakReason, YieldData},
};

use crate::{LiveryLayout, StylePlane, TextDirective, layout::Fragment, paint::resolve_color};

pub(crate) trait FragmentLookup<Id> {
    fn rect(&self, id: Id) -> Option<&Fragment>;

    fn atomic_rect(&self, id: Id) -> Option<&Fragment> {
        self.rect(id)
    }

    fn atomic_box_rect(&self, _box_id: BoxId) -> Option<&Fragment> {
        None
    }

    fn atomic_box_intrinsic_inline(&self, _box_id: BoxId) -> Option<IntrinsicSizes> {
        None
    }

    /// A first baseline from the atomic box's margin-box block-start. The
    /// default leaves existing atomic boxes on Parley's block-end fallback.
    fn atomic_box_baseline(&self, _box_id: BoxId) -> Option<f32> {
        None
    }
}

impl<Id> FragmentLookup<Id> for LiveryLayout<Id>
where
    Id: Copy + Eq + Hash,
{
    fn rect(&self, id: Id) -> Option<&Fragment> {
        self.get(id).map(|fragment| &**fragment)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Brush {
    color: [f32; 4],
    source_index: usize,
}

/// One retained text rectangle in viewport coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A source-node text range in rendered byte space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange<Id> {
    pub anchor_node: Id,
    pub anchor_offset: usize,
    pub focus_node: Id,
    pub focus_offset: usize,
}

/// A non-collapsed retained selection over Livery's shaped text.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSelection<Id> {
    pub range: TextRange<Id>,
    pub source_nodes: Vec<Id>,
    pub rects: Vec<TextRect>,
    pub text: String,
}

fn break_inline_lines(
    layout: &mut parley::Layout<Brush>,
    width: f32,
    wraps: bool,
    constraints: Option<&FloatLineConstraints>,
) {
    if !width.is_finite() || width <= 0.0 {
        layout.break_all_lines(None);
        return;
    }
    let Some(constraints) = constraints else {
        layout.break_all_lines(wraps.then_some(width));
        return;
    };
    let layout_max_advance = if wraps { width } else { f32::INFINITY };

    {
        let mut breaker = layout.break_lines();
        breaker
            .state_mut()
            .set_layout_max_advance(layout_max_advance);
        'lines: loop {
            let mut line_top = breaker.committed_y() as f32;
            let mut available = constraints.horizontal_physical_space(line_top, 0.0);
            for _ in 0..32 {
                {
                    let state = breaker.state_mut();
                    state.set_line_x(available.inline_start);
                    state.set_line_y(f64::from(line_top));
                    state.set_line_max_advance(if wraps {
                        available.inline_size
                    } else {
                        f32::INFINITY
                    });
                }
                let Some(yielded) = breaker.break_next() else {
                    break 'lines;
                };
                let YieldData::LineBreak(line) = yielded else {
                    debug_assert!(false, "in-flow inline layout yielded a non-line break");
                    break 'lines;
                };

                let spanning =
                    constraints.horizontal_physical_space(line_top, line.line_height.max(0.0));
                if ((spanning.inline_start - available.inline_start).abs() > 0.01
                    || (spanning.inline_size - available.inline_size).abs() > 0.01)
                    && breaker.revert()
                {
                    available = spanning;
                    continue;
                }

                if line.advance > available.inline_size + 0.01
                    && let Some(next_top) = constraints.next_wider_block_start(
                        line_top,
                        line.line_height.max(0.0),
                        line.advance,
                    )
                    && breaker.revert()
                {
                    line_top = next_top;
                    available =
                        constraints.horizontal_physical_space(line_top, line.line_height.max(0.0));
                    continue;
                }
                break;
            }
        }
    }
}

/// Retained font discovery, shaping scratch space, and font resources for one
/// Livery document session.
pub struct TextSystem {
    font_context: FontContext,
    layout_context: LayoutContext<Brush>,
    fonts: HashMap<FontInstanceKey, FontResource>,
    font_keys: HashMap<(u64, u32), FontInstanceKey>,
    ch_advances: HashMap<ChMetricKey, f32>,
    space_advances: HashMap<ChMetricKey, f32>,
    box_metrics: HashMap<ChMetricKey, FontBoxMetrics>,
    font_face_features: HashMap<String, Box<[FontFeatureSetting]>>,
    shape_count: u64,
}

/// A font at a style's size: its ascent, descent, line gap and x-height, which
/// CSS 2.1 10.8 builds an inline box's extent from.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FontBoxMetrics {
    font_size: f32,
    ascent: f32,
    descent: f32,
    line_gap: f32,
    x_height: f32,
}

fn metric_key(style: &ComputedValues, font_size: f32) -> ChMetricKey {
    ChMetricKey {
        family: style.font_family.to_string(),
        font_size: font_size.to_bits(),
        font_weight: font_weight(style).to_bits(),
        font_style: match style.font_style {
            CssFontStyle::Normal => 0,
            CssFontStyle::Italic => 1,
            CssFontStyle::Oblique => 2,
        },
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ChMetricKey {
    family: String,
    font_size: u32,
    font_weight: u32,
    font_style: u8,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> Self {
        Self {
            font_context: FontContext::new(),
            layout_context: LayoutContext::new(),
            fonts: HashMap::new(),
            font_keys: HashMap::new(),
            ch_advances: HashMap::new(),
            space_advances: HashMap::new(),
            box_metrics: HashMap::new(),
            font_face_features: HashMap::new(),
            shape_count: 0,
        }
    }

    pub fn shape_count(&self) -> u64 {
        self.shape_count
    }

    pub fn retained_font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Register host-supplied font bytes with this retained text system. A
    /// document owner rebuilds the system when its complete resource ledger
    /// changes, so removed faces cannot remain reachable.
    pub fn register_font_bytes(&mut self, bytes: Vec<u8>) {
        let Some(bytes) = normalized_font_bytes(bytes) else {
            return;
        };
        self.font_context
            .collection
            .register_fonts(parley::fontique::Blob::new(Arc::new(bytes)), None);
        self.ch_advances.clear();
        self.space_advances.clear();
        self.box_metrics.clear();
    }

    /// Register one host-resolved `@font-face` source under its authored CSS
    /// family and retain the face-level OpenType defaults for shaping.
    pub fn register_font_face_bytes(
        &mut self,
        bytes: Vec<u8>,
        family: &str,
        feature_settings: &CssFontFeatureSettings,
    ) {
        let Some(bytes) = normalized_font_bytes(bytes) else {
            return;
        };
        self.font_context.collection.register_fonts(
            parley::fontique::Blob::new(Arc::new(bytes)),
            Some(parley::fontique::FontInfoOverride {
                family_name: Some(family),
                ..Default::default()
            }),
        );
        self.font_face_features.insert(
            family.to_ascii_lowercase(),
            feature_settings.settings().into(),
        );
        self.ch_advances.clear();
        self.space_advances.clear();
        self.box_metrics.clear();
    }

    /// Resolve one CSS `ch` unit from the same font collection and matching
    /// inputs used by ordinary text shaping. CSS falls back to `0.5em` only
    /// when no usable `0` advance is available.
    pub(crate) fn ch_advance(&mut self, style: &ComputedValues) -> f32 {
        let font_size = super::paint::used_font_size(style);
        let key = metric_key(style, font_size);
        if let Some(advance) = self.ch_advances.get(&key) {
            return *advance;
        }

        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, "0", 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(font_family(style));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight(
            style,
        ))));
        builder.push_default(StyleProperty::FontStyle(font_style(style)));
        let mut layout = builder.build("0");
        layout.break_all_lines(None);
        let advance = layout
            .lines()
            .next()
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => Some(run.advance()),
                    PositionedLayoutItem::InlineBox(_) => None,
                })
            })
            .filter(|advance| advance.is_finite() && *advance > 0.0)
            .unwrap_or(font_size * 0.5);
        self.ch_advances.insert(key, advance);
        advance
    }

    /// `style`'s first available font's box metrics, probed once per font.
    fn box_metrics(&mut self, style: &ComputedValues) -> FontBoxMetrics {
        let font_size = super::paint::used_font_size(style);
        let key = metric_key(style, font_size);
        if let Some(metrics) = self.box_metrics.get(&key) {
            return *metrics;
        }
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, "x", 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(font_family(style));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight(
            style,
        ))));
        builder.push_default(StyleProperty::FontStyle(font_style(style)));
        let mut layout = builder.build("x");
        layout.break_all_lines(None);
        let metrics = layout
            .lines()
            .next()
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        let metrics = run.run().metrics();
                        Some(FontBoxMetrics {
                            font_size,
                            ascent: metrics.ascent,
                            descent: metrics.descent,
                            line_gap: metrics.leading,
                            x_height: metrics.x_height.unwrap_or(font_size * 0.5),
                        })
                    },
                    PositionedLayoutItem::InlineBox(_) => None,
                })
            })
            .unwrap_or(FontBoxMetrics {
                font_size,
                ascent: font_size * 0.8,
                descent: font_size * 0.2,
                line_gap: 0.0,
                x_height: font_size * 0.5,
            });
        self.box_metrics.insert(key, metrics);
        metrics
    }

    /// The line box one line of `style`'s own text makes: its space above the
    /// baseline and its height (CSS 2.1 10.8.1).
    pub(crate) fn line_box(&mut self, style: &ComputedValues) -> (f32, f32) {
        let metrics = self.box_metrics(style);
        let (above, below) = leading_extent(
            metrics.ascent,
            metrics.descent,
            used_line_height(style, metrics),
        );
        (above, above + below)
    }

    fn space_advance(&mut self, style: &ComputedValues) -> f32 {
        let font_size = super::paint::used_font_size(style);
        let key = metric_key(style, font_size);
        if let Some(advance) = self.space_advances.get(&key) {
            return *advance;
        }
        // Shape the space between two zeroes so font fallback cannot assign
        // an isolated whitespace run to a different face.
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, "0 0", 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(font_family(style));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight(
            style,
        ))));
        builder.push_default(StyleProperty::FontStyle(font_style(style)));
        let mut layout = builder.build("0 0");
        layout.break_all_lines(None);
        let advance = layout
            .lines()
            .next()
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => run
                        .run()
                        .clusters()
                        .find(|cluster| cluster.text_range() == (1..2))
                        .map(|cluster| cluster.advance()),
                    PositionedLayoutItem::InlineBox(_) => None,
                })
            })
            .filter(|advance| advance.is_finite() && *advance > 0.0)
            .unwrap_or(font_size * 0.5);
        self.space_advances.insert(key, advance);
        advance
    }

    fn character_advance(&mut self, character: char, style: &ComputedValues) -> f32 {
        let text = character.to_string();
        let font_size = super::paint::used_font_size(style);
        let features = effective_font_features(&self.font_face_features, style);
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, &text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(font_size));
        builder.push_default(font_family(style));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight(
            style,
        ))));
        builder.push_default(StyleProperty::FontStyle(font_style(style)));
        builder.push_default(StyleProperty::FontFeatures(FontFeatures::List(Cow::Owned(
            features,
        ))));
        if let Some(letter_spacing) = spacing_px(style.letter_spacing, font_size) {
            builder.push_default(StyleProperty::LetterSpacing(letter_spacing));
        }
        let mut layout = builder.build(&text);
        layout.break_all_lines(None);
        layout
            .lines()
            .next()
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => Some(run.advance()),
                    PositionedLayoutItem::InlineBox(_) => None,
                })
            })
            .filter(|advance| advance.is_finite() && *advance > 0.0)
            .unwrap_or(0.0)
    }

    fn tab_stop(&mut self, style: &ComputedValues) -> f32 {
        let font_size = super::paint::used_font_size(style);
        match style.tab_size {
            TabSize::Number(number) => {
                let letter = spacing_px(style.letter_spacing, font_size).unwrap_or(0.0);
                let word = spacing_px(style.word_spacing, font_size).unwrap_or(0.0);
                let space = if style
                    .font_family
                    .to_string()
                    .eq_ignore_ascii_case("monospace")
                {
                    // CSS generic monospace guarantees equal advances;
                    // use the retained `ch` metric so tab stops and `ch`
                    // lengths share that invariant even if fallback gave
                    // an isolated space a different face.
                    self.ch_advance(style)
                } else {
                    self.space_advance(style)
                };
                number.max(0.0) * (space + letter + word).max(0.0)
            },
            TabSize::Length(length) => length.unit.to_px(length.value, font_size, 16.0).max(0.0),
        }
    }

    /// Format one consecutive inline group for both layout and paint.
    pub(crate) fn format_inline_group<D, F>(
        &mut self,
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        boxes: &CssBoxTree<D::NodeId>,
        fragments: &F,
        request: InlineRequest<'_>,
    ) -> Option<InlineLayout<BoxId>>
    where
        D: LayoutDom,
        D::NodeId: Copy + Eq + Hash,
        F: FragmentLookup<D::NodeId>,
    {
        let parent_style = request.parent_style;
        let mut text = String::new();
        let mut spans = Vec::new();
        let mut inline_boxes = Vec::new();
        let mut inline_styles = HashMap::new();
        let mut owners = Vec::new();
        {
            let mut collector = BoxInlineCollector {
                dom,
                styles,
                boxes,
                fragments,
                owners: &mut owners,
                text: &mut text,
                spans: &mut spans,
                inline_boxes: &mut inline_boxes,
                inline_styles: &mut inline_styles,
                percentage_basis: request.width,
                intrinsic_kind: request.intrinsic_kind,
            };
            for root in request.roots {
                collector.collect(*root, parent_style);
            }
        }
        // A forced break adds text without a span, so a block holding only
        // `<br>` still formats its line.
        if text.is_empty() && inline_boxes.is_empty() {
            return None;
        }

        let Shaped {
            items,
            break_lines,
            baselines,
        } = self.shape(
            &text,
            &mut spans,
            &inline_boxes,
            &inline_styles,
            line_height_quirk(dom.quirks_mode(), parent_style),
            request.width,
            parent_style,
            request.line_constraints,
            request.intrinsic_kind,
        );
        let text_sources = spans
            .iter()
            .filter_map(|span| {
                span.source.map(|source| TextSource {
                    source,
                    range: span.range.clone(),
                })
            })
            .collect();
        let mut right = 0.0_f32;
        let mut top = f32::INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        for item in &items {
            let fragment = match item {
                ShapedItem::Text(run) => run.line_fragment,
                ShapedItem::InlineBox { line_fragment, .. } => *line_fragment,
            };
            right = right.max(fragment.x + fragment.width);
            top = top.min(fragment.y);
            bottom = bottom.max(fragment.y + fragment.height);
        }
        if let Some((upper, lower)) = break_lines {
            top = top.min(upper);
            bottom = bottom.max(lower);
        }
        if top.is_finite() && bottom.is_finite() {
            Some(InlineLayout {
                width: right.max(0.0),
                height: (bottom - top).max(0.0),
                baselines,
                items,
                text,
                text_sources,
            })
        } else {
            Some(InlineLayout {
                width: 0.0,
                height: 0.0,
                baselines: None,
                items,
                text,
                text_sources,
            })
        }
    }

    pub(crate) fn begin_frame<Id>(&self) -> TextFrame<Id> {
        TextFrame::default()
    }

    pub(crate) fn fonts_for<Id>(&self, frame: &TextFrame<Id>) -> Vec<FontResource>
    where
        Id: Eq + Hash,
    {
        let mut fonts = frame
            .used_fonts
            .iter()
            .filter_map(|key| self.fonts.get(key).cloned())
            .collect::<Vec<_>>();
        fonts.sort_by_key(|font| (font.key.0.0, font.key.1));
        fonts
    }

    /// Shape each consecutive inline child group into one Parley layout. The
    /// glyph runs stay keyed by their source text node so the DOM paint walk
    /// keeps source order while line breaking and baselines are shared.
    pub(crate) fn prepare_inline_children<D>(
        &mut self,
        frame: &mut TextFrame<D::NodeId>,
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        fragments: &LiveryLayout<D::NodeId>,
        parent: D::NodeId,
        parent_style: &ComputedValues,
    ) where
        D: LayoutDom,
        D::NodeId: Copy + Eq + Hash,
    {
        let Some(parent_box) = fragments.boxes().principal_box(parent) else {
            return;
        };
        let mut inline_parent_style = parent_style.clone();
        if parent_style.display == Display::ListItem
            && parent_style.list_style_position == ListStylePosition::Inside
        {
            let inline_box = if fragments.boxes()[parent_box].formatting_context
                == Some(FormattingContextKind::Inline)
            {
                Some(parent_box)
            } else {
                Self::marker_inline_child(fragments.boxes(), parent_box, parent)
            };
            let Some(inline_box) = inline_box else {
                return;
            };
            let marker_box = Self::marker_box(fragments.boxes(), inline_box, parent);
            let content_box = Self::inline_content_box(fragments.boxes(), inline_box);
            let anchor_fragment = content_box
                .and_then(|content_box| {
                    fragments.fragments().fragment_ids_for_box(content_box).last().and_then(|id| {
                        fragments.fragments().get(*id).map(|fragment| fragment.physical_rect())
                    })
                })
                .or_else(|| {
                    marker_box.and_then(|marker_box| {
                        fragments.fragments().fragment_ids_for_box(marker_box).last().and_then(|id| {
                            fragments.fragments().get(*id).map(|fragment| fragment.physical_rect())
                        })
                    })
                })
                .or_else(|| {
                    (inline_box == parent_box)
                        .then(|| frame.inline_fragments(parent).and_then(|items| items.first()).or_else(|| fragments.get(parent).map(|fragment| &**fragment)).copied())
                        .flatten()
                })
                .or_else(|| fragments.fragments().fragment_ids_for_box(inline_box).last().and_then(|id| fragments.fragments().get(*id).map(|fragment| fragment.physical_rect())));
            let Some(anchor_fragment) = anchor_fragment else {
                return;
            };
            let width = fragments.fragments().fragment_ids_for_box(parent_box).last()
                .and_then(|id| fragments.fragments().get(*id))
                .map(|fragment| crate::content_box_size(parent_style, fragment).0)
                .unwrap_or(anchor_fragment.width);
            let marker_only_anchor = content_box.is_none()
                && marker_box.is_some_and(|marker_box| {
                    fragments
                        .fragments()
                        .fragment_ids_for_box(marker_box)
                        .last()
                        .and_then(|id| fragments.fragments().get(*id))
                        .is_some()
                });
            let origin_x = if marker_only_anchor
                && parent_style.direction == Direction::Rtl
                && !parent_style.writing_mode.is_vertical()
            {
                anchor_fragment.x + anchor_fragment.width - width
            } else {
                anchor_fragment.x
            };
            let roots = fragments.boxes()[inline_box].children();
            if let Some(layout) = self.format_inline_group(
                dom,
                styles,
                fragments.boxes(),
                fragments,
                InlineRequest {
                    roots,
                    parent_style,
                    width,
                    intrinsic_kind: None,
                    line_constraints: None,
                },
            ) {
                layout.place(
                    frame,
                    styles,
                    |box_id| fragments.boxes().origin_node(box_id),
                    (origin_x, anchor_fragment.y),
                    width,
                );
            }
            return;
        }
        if fragments.boxes()[parent_box].formatting_context != Some(FormattingContextKind::Inline) {
            return;
        }
        let Some(parent_fragment) = frame
            .inline_fragments(parent)
            .and_then(|fragments| fragments.first())
            .or_else(|| fragments.get(parent).map(|fragment| &**fragment))
            .copied()
        else {
            return;
        };
        if matches!(parent_style.position, Position::Absolute | Position::Fixed) {
            inline_parent_style.vertical_align = VerticalAlign::Baseline;
        }
        let mut group = Vec::new();
        for child in dom.flat_children(parent) {
            if is_inline(dom, styles, child) {
                group.push(child);
            } else {
                self.flush_group(
                    frame,
                    dom,
                    styles,
                    fragments,
                    &group,
                    (&parent_fragment, &inline_parent_style),
                    parent,
                );
                group.clear();
            }
        }
        self.flush_group(
            frame,
            dom,
            styles,
            fragments,
            &group,
            (&parent_fragment, &inline_parent_style),
            parent,
        );
    }

    fn marker_inline_child<Id>(boxes: &CssBoxTree<Id>, parent: BoxId, owner: Id) -> Option<BoxId>
    where
        Id: Copy + Eq + Hash,
    {
        boxes[parent].children().iter().copied().find(|child| {
            boxes[*child].formatting_context == Some(FormattingContextKind::Inline)
                && boxes[*child].children().iter().any(|grandchild| {
                    matches!(
                        boxes[*grandchild].origin,
                        BoxOrigin::Pseudo {
                            owner: marker_owner,
                            pseudo: buckram::PseudoElement::Marker,
                        } if marker_owner == owner
                    )
                })
        })
    }

    fn marker_box<Id>(boxes: &CssBoxTree<Id>, inline_box: BoxId, owner: Id) -> Option<BoxId>
    where
        Id: Copy + Eq + Hash,
    {
        boxes[inline_box].children().iter().copied().find(|child| matches!(
            boxes[*child].origin,
            BoxOrigin::Pseudo { owner: marker_owner, pseudo: buckram::PseudoElement::Marker } if marker_owner == owner
        ))
    }

    fn inline_content_box<Id>(boxes: &CssBoxTree<Id>, inline_box: BoxId) -> Option<BoxId>
    where
        Id: Copy + Eq + Hash,
    {
        boxes[inline_box].children().iter().copied().find(|child| {
            boxes[*child].display.outside == Some(DisplayOutside::Inline)
                && !matches!(boxes[*child].origin, BoxOrigin::Pseudo { pseudo: buckram::PseudoElement::Marker, .. })
        })
    }

    pub(crate) fn emit_single<Id>(
        &mut self,
        frame: &mut TextFrame<Id>,
        source: &str,
        style: &ComputedValues,
        fragment: &Fragment,
        commands: &mut Vec<PaintCmd>,
    ) where
        Id: Copy + Eq + Hash,
    {
        let text = normalized_text(source, style);
        if text.is_empty() {
            return;
        }
        let mut spans = vec![SourceSpan::<Id> {
            source: None,
            owners: Vec::new(),
            style: style.clone(),
            range: 0..text.len(),
        }];
        for item in self
            .shape(
                text.as_ref(),
                &mut spans,
                &[],
                &HashMap::new(),
                false,
                fragment.width,
                style,
                None,
                None,
            )
            .items
        {
            let ShapedItem::Text(mut run) = item else {
                continue;
            };
            for glyph in &mut run.glyphs {
                glyph.point.x += fragment.x;
                glyph.point.y += fragment.y;
            }
            frame.used_fonts.insert(run.font_instance);
            commands.push(PaintCmd::DrawText(TextRunItem {
                placement: CommonPlacement::new(super::paint::bounds(fragment)),
                font_instance: run.font_instance,
                font_size: run.font_size,
                color: run.color,
                glyphs: run.glyphs,
                options: TextOptions::default(),
            }));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn flush_group<D>(
        &mut self,
        frame: &mut TextFrame<D::NodeId>,
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        fragments: &LiveryLayout<D::NodeId>,
        roots: &[D::NodeId],
        parent: (&Fragment, &ComputedValues),
        owner: D::NodeId,
    ) where
        D: LayoutDom,
        D::NodeId: Copy + Eq + Hash,
    {
        if roots.is_empty() {
            return;
        }
        let (parent_fragment, parent_style) = parent;
        let rtl_content_geometry = (parent_style.direction == Direction::Rtl
            && !parent_style.writing_mode.is_vertical())
        .then(|| fragments.get(owner))
        .flatten()
        .map(|fragment| {
            let edges = inline_decoration_edges(parent_style, fragment.width);
            (
                crate::content_box_size(parent_style, fragment).0,
                fragment.x + edges.left,
            )
        });
        let available_width = rtl_content_geometry
            .map(|(width, _)| width)
            .unwrap_or(parent_fragment.width);
        let mut text = String::new();
        let mut spans = Vec::new();
        let mut inline_boxes = Vec::new();
        let mut inline_styles = HashMap::new();
        let mut owners = vec![owner];
        {
            let mut collector = InlineCollector {
                dom,
                styles,
                fragments,
                already_prepared: &frame.prepared_sources,
                owners: &mut owners,
                text: &mut text,
                spans: &mut spans,
                inline_boxes: &mut inline_boxes,
                inline_styles: &mut inline_styles,
                percentage_basis: available_width,
            };
            for root in roots {
                collector.collect(*root, parent_style);
            }
        }
        if spans.is_empty() && inline_boxes.is_empty() {
            return;
        }

        let mut origin = spans
            .iter()
            .filter_map(|span| span.source.and_then(|id| fragments.get(id)))
            .next()
            .or_else(|| roots.iter().find_map(|id| fragments.get(*id)))
            .map_or((parent_fragment.x, parent_fragment.y), |fragment| {
                (fragment.x, fragment.y)
            });
        if let Some((_, content_x)) = rtl_content_geometry {
            origin.0 = content_x;
        }
        let mut visual_commands = Vec::new();
        let mut prepared_sources = Vec::new();
        let text_sources = spans
            .iter()
            .filter_map(|span| Some((span.source?, text.get(span.range.clone())?.to_owned())))
            .collect();
        frame.record_text_group(text_sources);
        for item in self
            .shape(
                &text,
                &mut spans,
                &inline_boxes,
                &inline_styles,
                line_height_quirk(dom.quirks_mode(), parent_style),
                available_width,
                parent_style,
                None,
                None,
            )
            .items
        {
            match item {
                ShapedItem::Text(mut run) => {
                    let Some(source) = run.source else {
                        continue;
                    };
                    translate_fragment(&mut run.fragment, origin);
                    let line_y = run.line_y + origin.1;
                    for glyph in &mut run.glyphs {
                        glyph.point.x += origin.0;
                        glyph.point.y += origin.1;
                    }
                    frame.record_inline_fragment(source, run.fragment, line_y);
                    for cluster in &run.clusters {
                        let mut cluster_fragment = cluster.fragment;
                        translate_fragment(&mut cluster_fragment, origin);
                        frame.record_text_cluster(
                            cluster.source,
                            cluster.range.clone(),
                            cluster_fragment,
                            cluster.rtl,
                        );
                    }
                    for owner in &run.owners {
                        frame.record_inline_fragment(
                            *owner,
                            decorated_inline_fragment(
                                styles,
                                *owner,
                                run.fragment,
                                available_width,
                            ),
                            line_y,
                        );
                    }
                    let command = PaintCmd::DrawText(TextRunItem {
                        placement: CommonPlacement::new(super::paint::bounds(parent_fragment)),
                        font_instance: run.font_instance,
                        font_size: run.font_size,
                        color: run.color,
                        glyphs: run.glyphs,
                        options: TextOptions::default(),
                    });
                    frame.used_fonts.insert(run.font_instance);
                    if frame.prepared_sources.insert(source) {
                        prepared_sources.push(source);
                    }
                    visual_commands.push(PreparedCommand {
                        source,
                        owners: run.owners,
                        command,
                    });
                },
                ShapedItem::InlineBox {
                    source,
                    owners,
                    mut fragment,
                    line_fragment: _,
                    edge,
                    paint,
                    mut line_y,
                } => {
                    frame.prepared_sources.insert(source);
                    translate_fragment(&mut fragment, origin);
                    line_y += origin.1;
                    if paint {
                        frame.record_inline_fragment(
                            source,
                            if edge {
                                decorated_inline_fragment(
                                    styles,
                                    source,
                                    fragment,
                                    available_width,
                                )
                            } else {
                                fragment
                            },
                            line_y,
                        );
                    }
                    for owner in owners {
                        frame.record_inline_fragment(
                            owner,
                            decorated_inline_fragment(
                                styles,
                                owner,
                                fragment,
                                available_width,
                            ),
                            line_y,
                        );
                    }
                },
            }
        }
        frame.record_prepared_group(prepared_sources, visual_commands);
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one shaping transaction carries the text, source spans, inline atoms and styles, document mode, width, root policy, and optional line constraints"
    )]
    fn shape<Id>(
        &mut self,
        text: &str,
        spans: &mut [SourceSpan<Id>],
        inline_boxes: &[InlineAtom<Id>],
        inline_styles: &HashMap<Id, ComputedValues>,
        quirks: bool,
        width: f32,
        root_style: &ComputedValues,
        line_constraints: Option<&FloatLineConstraints>,
        intrinsic_kind: Option<IntrinsicSizeKind>,
    ) -> Shaped<Id>
    where
        Id: Copy + Eq + Hash,
    {
        self.shape_count = self.shape_count.saturating_add(1);
        let default_style = spans.first().map_or(root_style, |span| &span.style);
        let default_features = effective_font_features(&self.font_face_features, default_style);
        let span_features = spans
            .iter()
            .map(|span| effective_font_features(&self.font_face_features, &span.style))
            .collect::<Vec<_>>();
        let default_tab_stop = self.tab_stop(default_style);
        let span_tab_stops = spans
            .iter()
            .map(|span| self.tab_stop(&span.style))
            .collect::<Vec<_>>();
        let first_hanging_advance = if root_style.hanging_punctuation.first {
            text.char_indices()
                .find(|(_, character)| !character.is_whitespace())
                .and_then(|(index, character)| {
                    let blocked = inline_boxes.iter().any(|inline_box| {
                        inline_box.index <= index && inline_box.line_width.abs() > f32::EPSILON
                    });
                    (!blocked && is_opening_hanging_punctuation(character)).then(|| {
                        let style = spans
                            .iter()
                            .find(|span| span.range.contains(&index))
                            .map_or(root_style, |span| &span.style);
                        self.character_advance(character, style)
                    })
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.font_context, text, 1.0, true);
        builder.set_base_level(Some(match root_style.direction {
            Direction::Ltr => 0,
            Direction::Rtl => 1,
        }));
        push_defaults(
            &mut builder,
            default_style,
            default_features,
            intrinsic_kind,
            default_tab_stop,
        );
        for (source_index, span) in spans.iter().enumerate() {
            push_span(
                &mut builder,
                &span.style,
                span.range.clone(),
                source_index,
                span_features[source_index].clone(),
                intrinsic_kind,
                span_tab_stops[source_index],
            );
        }
        for (index, inline_box) in inline_boxes.iter().enumerate() {
            if inline_box.marker {
                continue;
            }
            builder.push_inline_box(InlineBox {
                id: u64::try_from(index).unwrap_or(u64::MAX),
                kind: InlineBoxKind::InFlow,
                index: inline_box.index,
                width: inline_box.line_width,
                height: if inline_box.edge {
                    0.0
                } else {
                    inline_box.line_box_height
                },
            });
        }
        let mut layout = builder.build(text);
        let font_size = super::paint::used_font_size(root_style);
        let text_indent = root_style
            .text_indent
            .length
            .to_px(font_size, font_size, width)
            - first_hanging_advance;
        layout.set_text_indent(
            text_indent,
            IndentOptions {
                each_line: root_style.text_indent.each_line,
                hanging: root_style.text_indent.hanging,
            },
        );
        break_inline_lines(
            &mut layout,
            width,
            root_style.text_wrap_mode == TextWrapMode::Wrap,
            line_constraints,
        );
        let alignment = text_alignment(
            root_style.text_align,
            root_style.direction,
            root_style.text_justify,
        );
        layout.align(
            alignment,
            AlignmentOptions {
                last_line_alignment: last_line_alignment(root_style, alignment),
                ..AlignmentOptions::default()
            },
        );

        let mut result = Vec::new();
        let mut break_lines: Option<(f32, f32)> = None;
        // CSS 2.1 10.8: a line box is the union of its boxes' extents about
        // the baseline, starting from a strut in the root's own font.
        let root_metrics = self.box_metrics(root_style);
        let strut = leading_extent(
            root_metrics.ascent,
            root_metrics.descent,
            used_line_height(root_style, root_metrics),
        );
        // Each inline box's parent box, from the ownership chains; a box with
        // none sits in the line's root.
        let mut parents = HashMap::new();
        let chains = spans
            .iter()
            .map(|span| (span.owners.as_slice(), None))
            .chain(inline_boxes.iter().map(|inline_box| {
                let own = inline_box.edge || inline_box.empty_line;
                (
                    inline_box.owners.as_slice(),
                    own.then_some(inline_box.source),
                )
            }));
        for (owners, own) in chains {
            let mut parent = None;
            for id in owners.iter().copied().chain(own) {
                if inline_styles.contains_key(&id) {
                    if let Some(parent) = parent {
                        parents.insert(id, parent);
                    }
                    parent = Some(id);
                }
            }
        }
        let box_fonts = inline_styles
            .iter()
            .map(|(id, style)| (*id, self.box_metrics(style)))
            .collect::<HashMap<_, _>>();
        // Each box's extent, and its raise over its parent box's baseline
        // against the parent's font (CSS 2.1 10.8.1).
        let box_placements = inline_styles
            .iter()
            .map(|(id, style)| {
                let parent = parents
                    .get(id)
                    .map_or(root_metrics, |parent| box_fonts[parent]);
                (*id, inline_box_placement(style, box_fonts[id], parent))
            })
            .collect::<HashMap<_, _>>();
        let mut box_anchors = HashMap::new();
        for id in inline_styles.keys() {
            anchor_of(*id, &parents, &box_placements, &mut box_anchors);
        }
        let innermost = |owners: &[Id]| {
            owners
                .iter()
                .rev()
                .copied()
                .find(|id| box_placements.contains_key(id))
        };
        // The line height calculation quirk, as Chromium applies it: in quirks
        // mode an inline box counts on a line only where it holds text or a
        // forced break directly, or has inline-axis borders or padding (not
        // margins); the strut only where the root holds text, or a forced
        // break on a line where nothing else counts.
        let edged = quirks.then(|| {
            inline_boxes
                .iter()
                .filter(|inline_box| inline_box.edge && inline_box.paint)
                .map(|inline_box| inline_box.source)
                .collect::<HashSet<_>>()
        });
        // How far the model's line boxes have moved lines from where
        // Parley's float-aware breaking started them.
        let mut drift = 0.0_f32;
        let mut span_cursor = 0;
        let mut baselines: Option<(f32, f32)> = None;
        for line in layout.lines() {
            let source_metrics = *line.metrics();
            let content_height =
                (source_metrics.block_max_coord - source_metrics.block_min_coord).max(0.0);
            let line_top = parley_line_top(&source_metrics) + drift;
            let mut extents = LineExtents::new();
            if !quirks {
                extents.include(Anchor::root(), strut);
            }
            let mut positions = Vec::new();
            let mut seen_boxes = Vec::new();
            for item in line.items() {
                let (position, counts) = match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        // Text sits on its parent inline box's baseline.
                        let span = spans.get(run.style().brush.source_index);
                        let (anchor, mut extent) = span
                            .and_then(|span| innermost(&span.owners))
                            .map_or((Anchor::root(), strut), |owner| {
                                (box_anchors[&owner], box_placements[&owner].extent)
                            });
                        // A fallback font's own metrics count under `normal`.
                        if span.is_some_and(|span| {
                            matches!(span.style.line_height, CssLineHeight::Normal)
                        }) {
                            let metrics = run.run().metrics();
                            let (above, below) = leading_extent(
                                metrics.ascent,
                                metrics.descent,
                                normal_line_height(
                                    metrics.ascent,
                                    metrics.descent,
                                    metrics.leading,
                                ),
                            );
                            extent = (extent.0.max(above), extent.1.max(below));
                        }
                        (Some((anchor, extent)), true)
                    },
                    PositionedLayoutItem::InlineBox(positioned) => {
                        let inline_box = usize::try_from(positioned.id)
                            .ok()
                            .and_then(|index| inline_boxes.get(index));
                        let position = inline_box.and_then(|inline_box| {
                            // The box an edge belongs to is on this line, and
                            // so are an atom's inline ancestors.
                            if inline_box.edge {
                                extents.include_boxes(
                                    &mut seen_boxes,
                                    std::slice::from_ref(&inline_box.source),
                                    (&box_placements, &box_anchors),
                                    edged.as_ref(),
                                );
                            }
                            extents.include_boxes(
                                &mut seen_boxes,
                                &inline_box.owners,
                                (&box_placements, &box_anchors),
                                edged.as_ref(),
                            );
                            if inline_box.edge {
                                None
                            } else if inline_box.empty_line {
                                box_placements.get(&inline_box.source).map(|placement| {
                                    (box_anchors[&inline_box.source], placement.extent)
                                })
                            } else {
                                // An atom aligns within its parent inline box.
                                let parent = innermost(&inline_box.owners);
                                let extent = (
                                    inline_box.baseline,
                                    inline_box.line_box_height - inline_box.baseline,
                                );
                                let raise = vertical_align_raise(
                                    inline_box.vertical_align,
                                    inline_box.font_size,
                                    inline_box.line_height,
                                    extent,
                                    parent.map_or(root_metrics, |parent| box_fonts[&parent]),
                                );
                                let anchor = match raise {
                                    Some(raise) => parent
                                        .map_or(Anchor::root(), |parent| box_anchors[&parent])
                                        .raised(raise),
                                    None => Anchor {
                                        group: Some((inline_box.source, inline_box.vertical_align)),
                                        raise: 0.0,
                                    },
                                };
                                Some((anchor, extent))
                            }
                        });
                        // An empty inline box holds no text.
                        let counts =
                            !(quirks && inline_box.is_some_and(|inline_box| inline_box.empty_line));
                        (position, counts)
                    },
                };
                if counts && let Some((anchor, extent)) = position {
                    extents.include(anchor, extent);
                }
                positions.push(position);
            }
            // Text on the line puts its inline boxes there, with their own
            // extents, even a box whose only content here is a preserved
            // newline or forced break that Parley gives no item. An atom-only
            // line's range is empty.
            let range = line.text_range();
            let mut root_break = false;
            if range.start < range.end {
                while spans
                    .get(span_cursor)
                    .is_some_and(|span| span.range.end <= range.start)
                {
                    span_cursor += 1;
                }
                for span in spans[span_cursor..]
                    .iter()
                    .take_while(|span| span.range.start < range.end)
                {
                    let part = span.range.start.max(range.start)..span.range.end.min(range.end);
                    match innermost(&span.owners) {
                        Some(owner) => {
                            extents.include(box_anchors[&owner], box_placements[&owner].extent)
                        },
                        None if quirks && text[part].bytes().all(|byte| byte == b'\n') => {
                            root_break = true;
                        },
                        None => extents.include(Anchor::root(), strut),
                    }
                    extents.include_boxes(
                        &mut seen_boxes,
                        &span.owners,
                        (&box_placements, &box_anchors),
                        edged.as_ref(),
                    );
                }
            }
            if root_break && extents.is_empty() {
                extents.include(Anchor::root(), strut);
            }
            let (above, line_height) = extents.resolve();
            let baseline = line_top + above;
            drift += line_height - source_metrics.line_height;
            if line.break_reason() == BreakReason::Explicit {
                let (top, bottom) = (line_top, line_top + line_height);
                break_lines = Some(break_lines.map_or((top, bottom), |(upper, lower)| {
                    (upper.min(top), lower.max(bottom))
                }));
            }
            let emitted = result.len();
            for (item, position) in line.items().zip(positions) {
                let own_baseline = position.map_or(baseline, |(anchor, _)| {
                    extents.baseline_at(anchor, baseline, line_top, line_height)
                });
                match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        let parley_run = run.run();
                        let brush = &run.style().brush;
                        let span = spans.get(brush.source_index);
                        let mut glyphs = run
                            .positioned_glyphs()
                            .map(|glyph| GlyphInstance {
                                index: glyph.id,
                                point: LayoutPoint::new(glyph.x, glyph.y),
                            })
                            .collect::<Vec<_>>();
                        if glyphs.is_empty() {
                            continue;
                        }
                        // Parley set every glyph on its own line's baseline.
                        for glyph in &mut glyphs {
                            glyph.point.y += own_baseline - source_metrics.baseline;
                        }
                        let line_fragment_y = line_top + (own_baseline - baseline);
                        let mut cluster_x = run.offset();
                        let mut clusters = Vec::new();
                        for cluster in parley_run.visual_clusters() {
                            let advance = cluster.advance().max(0.0);
                            let global = cluster.text_range();
                            let source_span = spans
                                .get(cluster.first_style().brush.source_index)
                                .and_then(|span| span.source.map(|source| (source, span)));
                            if let Some((source, source_span)) = source_span {
                                let start = global.start.max(source_span.range.start);
                                let end = global.end.min(source_span.range.end);
                                if start < end {
                                    clusters.push(ShapedCluster {
                                        source,
                                        range: (start - source_span.range.start)
                                            ..(end - source_span.range.start),
                                        fragment: Fragment {
                                            x: cluster_x,
                                            y: line_fragment_y,
                                            width: advance,
                                            height: line_height.max(0.0),
                                        },
                                        rtl: cluster.is_rtl(),
                                    });
                                }
                            }
                            cluster_x += advance;
                        }
                        let source = span.and_then(|span| span.source);
                        let trailing_content_end = source.and_then(|source| {
                            let source_end = span
                                .and_then(|span| text.get(span.range.clone()))
                                .map(|text| text.trim_end_matches(is_css_whitespace).len())?;
                            clusters
                                .iter()
                                .filter(|cluster| {
                                    cluster.source == source && cluster.range.start < source_end
                                })
                                .map(|cluster| cluster.fragment.x + cluster.fragment.width)
                                .max_by(|left, right| left.total_cmp(right))
                        });
                        let [red, green, blue, alpha] = brush.color;
                        result.push(ShapedItem::Text(ShapedRun {
                            source,
                            owners: span.map_or_else(Vec::new, |span| span.owners.clone()),
                            trailing_content_end,
                            line_y: line_top,
                            fragment: Fragment {
                                x: run.offset(),
                                y: line_fragment_y,
                                width: run.advance().max(0.0),
                                height: line_height.max(content_height).max(0.0),
                            },
                            line_fragment: Fragment {
                                x: run.offset(),
                                y: line_top,
                                width: run.advance().max(0.0),
                                height: line_height.max(0.0),
                            },
                            font_instance: self.intern_font(parley_run.font()),
                            font_size: parley_run.font_size(),
                            color: ColorF::new(red, green, blue, alpha),
                            glyphs,
                            clusters,
                        }));
                    },
                    PositionedLayoutItem::InlineBox(positioned) => {
                        let Some(inline_box) = usize::try_from(positioned.id)
                            .ok()
                            .and_then(|index| inline_boxes.get(index))
                        else {
                            continue;
                        };
                        // The margin box's top; an edge spans the line box.
                        let top = match position {
                            Some((_, (above, _))) => own_baseline - above,
                            None => line_top,
                        };
                        result.push(ShapedItem::InlineBox {
                            source: inline_box.source,
                            owners: inline_box.owners.clone(),
                            fragment: if inline_box.edge {
                                Fragment {
                                    x: positioned.x,
                                    y: top,
                                    width: positioned.width,
                                    height: line_height.max(0.0),
                                }
                            } else {
                                Fragment {
                                    x: positioned.x + inline_box.margin_left,
                                    y: top + inline_box.margin_top,
                                    width: inline_box.fragment.width,
                                    height: inline_box.fragment.height,
                                }
                            },
                            line_fragment: Fragment {
                                x: positioned.x,
                                y: line_top,
                                width: positioned.width,
                                height: line_height.max(0.0),
                            },
                            edge: inline_box.edge,
                            paint: inline_box.paint,
                            line_y: line_top,
                        });
                    },
                }
            }
            // A line ending in a forced break is no phantom even with nothing
            // else on it (CSS 2.1 9.4.2).
            if result.len() > emitted || line.break_reason() == BreakReason::Explicit {
                baselines =
                    Some(baselines.map_or((baseline, baseline), |(first, _)| (first, baseline)));
            }
        }
        append_positioned_inline_start_markers(&mut result, inline_boxes);
        Shaped {
            items: result,
            break_lines,
            baselines,
        }
    }

    fn intern_font(&mut self, font: &parley::FontData) -> FontInstanceKey {
        let identity = (font.data.id(), font.index);
        if let Some(key) = self.font_keys.get(&identity) {
            return *key;
        }
        let bytes = font.data.data();
        let key = content_key(bytes, font.index);
        self.fonts.entry(key).or_insert_with(|| FontResource {
            key,
            data: Arc::new(bytes.to_vec()),
            index: font.index,
        });
        self.font_keys.insert(identity, key);
        key
    }
}

/// Turn a supported webfont container into the SFNT bytes consumed by
/// Fontique. OTS owns both WOFF2 validation and decompression, so inputs it
/// rejects never enter the retained font collection. Ordinary SFNT bytes stay
/// byte-identical and keep their existing registration path.
fn normalized_font_bytes(bytes: Vec<u8>) -> Option<Vec<u8>> {
    if bytes.starts_with(b"wOF2") {
        #[cfg(not(target_arch = "wasm32"))]
        {
            fontsan::process(&bytes).ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    } else {
        Some(bytes)
    }
}

pub(crate) struct InlineRequest<'a> {
    pub(crate) roots: &'a [BoxId],
    pub(crate) parent_style: &'a ComputedValues,
    pub(crate) width: f32,
    pub(crate) intrinsic_kind: Option<IntrinsicSizeKind>,
    pub(crate) line_constraints: Option<&'a FloatLineConstraints>,
}

pub(crate) struct InlineLayout<Source> {
    width: f32,
    height: f32,
    baselines: Option<(f32, f32)>,
    items: Vec<ShapedItem<Source>>,
    text: String,
    text_sources: Vec<TextSource<Source>>,
}

pub(crate) struct InlinePlacement<Source> {
    pub(crate) fragments: HashMap<Source, Vec<Fragment>>,
    line_keys: HashMap<Source, Vec<f32>>,
}

impl<Source> Default for InlinePlacement<Source> {
    fn default() -> Self {
        Self {
            fragments: HashMap::new(),
            line_keys: HashMap::new(),
        }
    }
}

impl<Source> InlinePlacement<Source>
where
    Source: Copy + Eq + Hash,
{
    fn record(&mut self, source: Source, fragment: Fragment, line_y: f32) {
        let fragments = self.fragments.entry(source).or_default();
        let line_keys = self.line_keys.entry(source).or_default();
        if let Some(previous) = fragments.last_mut()
            && line_keys
                .last()
                .is_some_and(|previous_line| (previous_line - line_y).abs() <= 0.5)
            && fragment.x <= previous.x + previous.width + 0.5
        {
            let right = (previous.x + previous.width).max(fragment.x + fragment.width);
            let bottom = (previous.y + previous.height).max(fragment.y + fragment.height);
            previous.x = previous.x.min(fragment.x);
            previous.width = right - previous.x;
            previous.y = previous.y.min(fragment.y);
            previous.height = bottom - previous.y;
            return;
        }
        fragments.push(fragment);
        line_keys.push(line_y);
    }
}

impl<Source> InlineLayout<Source>
where
    Source: Copy + Eq + Hash,
{
    pub(crate) fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// The first and last line boxes' baselines, relative to this inline
    /// formatting context's block-start edge; a line of atoms alone has one
    /// too (CSS 2.1 10.8), as does a line holding only a forced break (9.4.2).
    /// None when no line holds anything, so the formatting context
    /// synthesizes its block-end fallback.
    pub(crate) fn baselines(&self) -> Option<(f32, f32)> {
        self.baselines
            .filter(|(first, last)| first.is_finite() && last.is_finite())
    }

    pub(crate) fn place<Id, Resolve>(
        &self,
        frame: &mut TextFrame<Id>,
        styles: &StylePlane<Id>,
        mut node_for: Resolve,
        origin: (f32, f32),
        percentage_basis: f32,
    ) -> InlinePlacement<Source>
    where
        Id: Copy + Eq + Hash,
        Resolve: FnMut(Source) -> Option<Id>,
    {
        let container = Fragment {
            x: origin.0,
            y: origin.1,
            width: percentage_basis,
            height: self.height,
        };
        let mut visual_commands = Vec::new();
        let mut prepared_sources = Vec::new();
        let mut placement = InlinePlacement::default();
        let text_sources = self
            .text_sources
            .iter()
            .filter_map(|source| {
                Some((
                    node_for(source.source)?,
                    self.text.get(source.range.clone())?.to_owned(),
                ))
            })
            .collect();
        frame.record_text_group(text_sources);

        for item in &self.items {
            match item {
                ShapedItem::Text(run) => {
                    let Some(source) = run.source else {
                        continue;
                    };
                    let mut fragment = run.fragment;
                    translate_fragment(&mut fragment, origin);
                    let line_y = run.line_y + origin.1;
                    let mut glyphs = run.glyphs.clone();
                    for glyph in &mut glyphs {
                        glyph.point.x += origin.0;
                        glyph.point.y += origin.1;
                    }
                    placement.record(source, fragment, line_y);
                    let Some(source_node) = node_for(source) else {
                        continue;
                    };
                    frame.record_inline_fragment(source_node, fragment, line_y);
                    for cluster in &run.clusters {
                        let Some(cluster_node) = node_for(cluster.source) else {
                            continue;
                        };
                        let mut cluster_fragment = cluster.fragment;
                        translate_fragment(&mut cluster_fragment, origin);
                        frame.record_text_cluster(
                            cluster_node,
                            cluster.range.clone(),
                            cluster_fragment,
                            cluster.rtl,
                        );
                    }
                    let mut command_owners = Vec::new();
                    for owner in &run.owners {
                        let Some(owner_node) = node_for(*owner) else {
                            continue;
                        };
                        let decorated = decorated_inline_fragment(
                            styles,
                            owner_node,
                            fragment,
                            percentage_basis,
                        );
                        placement.record(*owner, decorated, line_y);
                        frame.record_inline_fragment(owner_node, decorated, line_y);
                        command_owners.push(owner_node);
                    }
                    frame.used_fonts.insert(run.font_instance);
                    if frame.prepared_sources.insert(source_node) {
                        prepared_sources.push(source_node);
                    }
                    visual_commands.push(PreparedCommand {
                        source: source_node,
                        owners: command_owners,
                        command: PaintCmd::DrawText(TextRunItem {
                            placement: CommonPlacement::new(super::paint::bounds(&container)),
                            font_instance: run.font_instance,
                            font_size: run.font_size,
                            color: run.color,
                            glyphs,
                            options: TextOptions::default(),
                        }),
                    });
                },
                ShapedItem::InlineBox {
                    source,
                    owners,
                    fragment,
                    edge,
                    paint,
                    line_y,
                    ..
                } => {
                    let mut fragment = *fragment;
                    translate_fragment(&mut fragment, origin);
                    let line_y = *line_y + origin.1;
                    let source_node = node_for(*source);
                    if let Some(node) = source_node {
                        frame.prepared_sources.insert(node);
                    }
                    if *paint {
                        let source_fragment = if *edge {
                            source_node.map_or(fragment, |node| {
                                decorated_inline_fragment(styles, node, fragment, percentage_basis)
                            })
                        } else {
                            fragment
                        };
                        placement.record(*source, source_fragment, line_y);
                        if let Some(node) = source_node {
                            frame.record_inline_fragment(node, source_fragment, line_y);
                        }
                    }
                    for owner in owners {
                        let Some(owner_node) = node_for(*owner) else {
                            continue;
                        };
                        let decorated = decorated_inline_fragment(
                            styles,
                            owner_node,
                            fragment,
                            percentage_basis,
                        );
                        placement.record(*owner, decorated, line_y);
                        frame.record_inline_fragment(owner_node, decorated, line_y);
                    }
                },
            }
        }
        frame.record_prepared_group(prepared_sources, visual_commands);
        placement
    }
}

/// One element's recorded inline boxes and the lines they sit on.
pub(crate) struct InlineBoxes {
    fragments: Option<Vec<Fragment>>,
    line_keys: Option<Vec<f32>>,
}

fn restore<Id: Eq + Hash, V>(map: &mut HashMap<Id, V>, key: Id, value: Option<V>) {
    match value {
        Some(value) => map.insert(key, value),
        None => map.remove(&key),
    };
}

#[derive(Clone, Debug)]
pub(crate) struct TextFrame<Id> {
    prepared_groups: Vec<Vec<PreparedCommand<Id>>>,
    source_groups: HashMap<Id, usize>,
    prepared_sources: HashSet<Id>,
    inline_fragments: HashMap<Id, Vec<Fragment>>,
    inline_line_keys: HashMap<Id, Vec<f32>>,
    painted_decorations: HashSet<Id>,
    used_fonts: HashSet<FontInstanceKey>,
    text_order: Vec<Id>,
    text_values: HashMap<Id, String>,
    text_groups: HashMap<Id, usize>,
    text_clusters: Vec<RetainedTextCluster<Id>>,
    /// Every source that has ever contributed a retained cluster. It only
    /// grows, so it is a safe over-approximation: when it says a subtree owns
    /// no clusters, the subtree owns none.
    cluster_sources: HashSet<Id>,
    next_text_group: usize,
}

impl<Id> Default for TextFrame<Id> {
    fn default() -> Self {
        Self {
            prepared_groups: Vec::new(),
            source_groups: HashMap::new(),
            prepared_sources: HashSet::new(),
            inline_fragments: HashMap::new(),
            inline_line_keys: HashMap::new(),
            painted_decorations: HashSet::new(),
            used_fonts: HashSet::new(),
            text_order: Vec::new(),
            text_values: HashMap::new(),
            text_groups: HashMap::new(),
            text_clusters: Vec::new(),
            cluster_sources: HashSet::new(),
            next_text_group: 0,
        }
    }
}

impl<Id> TextFrame<Id>
where
    Id: Copy + Eq + Hash,
{
    pub(crate) fn drain(
        &mut self,
        source: Id,
        inline_owner: Option<Id>,
        excluded_roots: Option<&HashSet<Id>>,
        commands: &mut Vec<PaintCmd>,
    ) -> bool {
        let prepared = self.prepared_sources.contains(&source);
        if let Some(group) = self.source_groups.get(&source).copied() {
            let mut retained = Vec::new();
            for prepared in std::mem::take(&mut self.prepared_groups[group]) {
                let belongs_to_owner =
                    inline_owner.is_none_or(|owner| prepared.owners.contains(&owner));
                let belongs_to_child_context = excluded_roots
                    .is_some_and(|roots| prepared.owners.iter().any(|owner| roots.contains(owner)));
                if belongs_to_owner && !belongs_to_child_context {
                    commands.push(prepared.command);
                } else {
                    retained.push(prepared);
                }
            }
            self.prepared_groups[group] = retained;
        }
        prepared
    }

    fn record_prepared_group(&mut self, sources: Vec<Id>, commands: Vec<PreparedCommand<Id>>) {
        if sources.is_empty() {
            return;
        }
        let group = self.prepared_groups.len();
        self.prepared_groups.push(commands);
        for source in sources {
            self.source_groups.insert(source, group);
        }
    }

    /// Replace text data produced by one retained formatting subtree without
    /// disturbing shaped commands, clusters, and text order outside it. The
    /// caller supplies DOM text order because a selected root may gain its
    /// first text source, which has no old frame position to reuse.
    pub(crate) fn replace_subtree_from(
        &mut self,
        fresh: &Self,
        replaced: &HashSet<Id>,
        dom_text_order: &[Id],
    ) {
        let mut merged = Self::default();
        merged.append_prepared_from(self, |source| !replaced.contains(&source));
        merged.append_prepared_from(fresh, |source| replaced.contains(&source));

        merged.copy_geometry_from(self, |source| !replaced.contains(&source));
        merged.copy_geometry_from(fresh, |source| replaced.contains(&source));
        merged.rebuild_used_fonts();

        merged.copy_text_from(self, |source| !replaced.contains(&source), 0);
        let fresh_group_base = merged.next_text_group;
        merged.copy_text_from(fresh, |source| replaced.contains(&source), fresh_group_base);
        merged.next_text_group = fresh_group_base.saturating_add(fresh.next_text_group);
        merged.text_order = dom_text_order
            .iter()
            .copied()
            .filter(|source| merged.text_values.contains_key(source))
            .collect();

        *self = merged;
    }

    fn rebuild_used_fonts(&mut self) {
        self.used_fonts.clear();
        for command in self.prepared_groups.iter().flatten() {
            if let PaintCmd::DrawText(run) = &command.command {
                self.used_fonts.insert(run.font_instance);
            }
        }
    }

    fn append_prepared_from(&mut self, source_frame: &Self, mut includes: impl FnMut(Id) -> bool) {
        for (old_group, commands) in source_frame.prepared_groups.iter().enumerate() {
            let sources = source_frame
                .source_groups
                .iter()
                .filter_map(|(source, group)| {
                    (*group == old_group && includes(*source)).then_some(*source)
                })
                .collect::<Vec<_>>();
            if sources.is_empty() {
                continue;
            }
            let group = self.prepared_groups.len();
            self.prepared_groups.push(commands.clone());
            for source in sources {
                self.source_groups.insert(source, group);
                self.prepared_sources.insert(source);
            }
        }
    }

    fn copy_geometry_from(&mut self, source_frame: &Self, mut includes: impl FnMut(Id) -> bool) {
        for (source, fragments) in &source_frame.inline_fragments {
            if includes(*source) {
                self.inline_fragments.insert(*source, fragments.clone());
            }
        }
        for (source, lines) in &source_frame.inline_line_keys {
            if includes(*source) {
                self.inline_line_keys.insert(*source, lines.clone());
            }
        }
        let copied = source_frame
            .text_clusters
            .iter()
            .filter(|cluster| includes(cluster.source))
            .cloned()
            .collect::<Vec<_>>();
        self.cluster_sources
            .extend(copied.iter().map(|cluster| cluster.source));
        self.text_clusters.extend(copied);
    }

    fn copy_text_from(
        &mut self,
        source_frame: &Self,
        mut includes: impl FnMut(Id) -> bool,
        group_offset: usize,
    ) {
        for (source, value) in &source_frame.text_values {
            if includes(*source) {
                self.text_values.insert(*source, value.clone());
                let group = source_frame
                    .text_groups
                    .get(source)
                    .copied()
                    .unwrap_or_default();
                self.text_groups
                    .insert(*source, group.saturating_add(group_offset));
            }
        }
        self.next_text_group = self
            .next_text_group
            .max(source_frame.next_text_group.saturating_add(group_offset));
    }

    pub(crate) fn mark_decoration_painted(&mut self, source: Id) -> bool {
        self.painted_decorations.insert(source)
    }

    /// Move all shaped text belonging to a translated DOM subtree.
    ///
    /// Final relative and absolute positioning moves the fragment tree after
    /// inline formatting has already placed glyphs in document coordinates.
    /// Keep the retained text frame in lockstep with that fragment translation.
    pub(crate) fn translate_subtree<D>(&mut self, dom: &D, root: Id, offset: (f32, f32))
    where
        D: LayoutDom<NodeId = Id>,
    {
        if offset.0 == 0.0 && offset.1 == 0.0 {
            return;
        }
        let mut nodes = HashSet::new();
        collect_subtree_nodes(dom, root, &mut nodes);
        // T2: final positioning calls this once per positioned box. A document
        // whose positioned boxes carry no text — the L0b shape, and most
        // application chrome — must not pay a corpus scan per box, so the
        // subtree is asked first whether it owns any retained text at all.
        let owns_text = nodes.iter().any(|node| {
            self.source_groups.contains_key(node)
                || self.cluster_sources.contains(node)
                || self.inline_fragments.contains_key(node)
                || self.inline_line_keys.contains_key(node)
        });
        if !owns_text {
            return;
        }
        for group in &mut self.prepared_groups {
            for prepared in group {
                if nodes.contains(&prepared.source) {
                    translate_paint_command(&mut prepared.command, offset);
                }
            }
        }
        // The keyed planes are walked by subtree node, not by corpus entry.
        for node in &nodes {
            if let Some(fragments) = self.inline_fragments.get_mut(node) {
                for fragment in fragments {
                    translate_fragment(fragment, offset);
                }
            }
            if let Some(lines) = self.inline_line_keys.get_mut(node) {
                for line in lines {
                    *line += offset.1;
                }
            }
        }
        for cluster in &mut self.text_clusters {
            if nodes.contains(&cluster.source) {
                translate_fragment(&mut cluster.fragment, offset);
            }
        }
    }

    /// Whether a retained DOM subtree owns shaped text commands.
    ///
    /// A geometry-only leaf resize cannot reuse these commands because a new
    /// inline size may change wrapping and glyph placement. Pure translation
    /// remains safe and is handled by [`Self::translate_subtree`].
    pub(crate) fn subtree_has_prepared_text<D>(&self, dom: &D, root: Id) -> bool
    where
        D: LayoutDom<NodeId = Id>,
    {
        let mut nodes = HashSet::new();
        collect_subtree_nodes(dom, root, &mut nodes);
        if !nodes.iter().any(|node| self.source_groups.contains_key(node)) {
            return false;
        }
        self.prepared_groups
            .iter()
            .flatten()
            .any(|prepared| nodes.contains(&prepared.source))
    }

    pub(crate) fn inline_fragments(&self, source: Id) -> Option<&[Fragment]> {
        self.inline_fragments.get(&source).map(Vec::as_slice)
    }

    /// `source`'s recorded inline boxes, to put back with
    /// [`Self::restore_inline_boxes`].
    pub(crate) fn inline_boxes(&self, source: Id) -> InlineBoxes {
        InlineBoxes {
            fragments: self.inline_fragments.get(&source).cloned(),
            line_keys: self.inline_line_keys.get(&source).cloned(),
        }
    }

    /// Put back the inline boxes [`Self::inline_boxes`] read for `source`.
    pub(crate) fn restore_inline_boxes(&mut self, source: Id, boxes: InlineBoxes) {
        restore(&mut self.inline_fragments, source, boxes.fragments);
        restore(&mut self.inline_line_keys, source, boxes.line_keys);
    }

    pub(crate) fn first_inline_line(&self, source: Id) -> Option<f32> {
        self.inline_line_keys
            .get(&source)
            .and_then(|lines| lines.first().copied())
    }

    #[cfg(test)]
    pub(crate) fn text_order(&self) -> &[Id] {
        &self.text_order
    }

    fn record_inline_fragment(&mut self, source: Id, fragment: Fragment, line_y: f32) {
        let fragments = self.inline_fragments.entry(source).or_default();
        let line_keys = self.inline_line_keys.entry(source).or_default();
        if let Some(previous) = fragments.last_mut()
            && line_keys
                .last()
                .is_some_and(|previous_line| (previous_line - line_y).abs() <= 0.5)
            && fragment.x <= previous.x + previous.width + 0.5
        {
            let right = (previous.x + previous.width).max(fragment.x + fragment.width);
            let bottom = (previous.y + previous.height).max(fragment.y + fragment.height);
            previous.x = previous.x.min(fragment.x);
            previous.width = right - previous.x;
            previous.y = previous.y.min(fragment.y);
            previous.height = bottom - previous.y;
            return;
        }
        fragments.push(fragment);
        line_keys.push(line_y);
    }

    fn record_text_group(&mut self, sources: Vec<(Id, String)>) {
        if sources.is_empty() {
            return;
        }
        let group = self.next_text_group;
        self.next_text_group = self.next_text_group.saturating_add(1);
        for (source, text) in sources {
            if self.text_values.contains_key(&source) {
                continue;
            }
            self.text_order.push(source);
            self.text_values.insert(source, text);
            self.text_groups.insert(source, group);
        }
    }

    fn record_text_cluster(
        &mut self,
        source: Id,
        range: Range<usize>,
        fragment: Fragment,
        rtl: bool,
    ) {
        self.cluster_sources.insert(source);
        self.text_clusters.push(RetainedTextCluster {
            source,
            range,
            fragment,
            rtl,
        });
    }

    pub(crate) fn text_position_at_point<F>(
        &self,
        x: f32,
        y: f32,
        mut transform: F,
    ) -> Option<(Id, usize)>
    where
        F: FnMut(Id, Fragment) -> TextRect,
    {
        let mut closest = None::<(f32, &RetainedTextCluster<Id>, TextRect)>;
        for cluster in &self.text_clusters {
            let rect = transform(cluster.source, cluster.fragment);
            if rect.height <= 0.0 || y < rect.y || y >= rect.y + rect.height {
                continue;
            }
            let distance = if x < rect.x {
                rect.x - x
            } else if x > rect.x + rect.width {
                x - (rect.x + rect.width)
            } else {
                0.0
            };
            if closest.as_ref().is_none_or(|(best, _, _)| distance < *best) {
                closest = Some((distance, cluster, rect));
            }
        }
        let (_, cluster, rect) = closest?;
        let leading = x <= rect.x + rect.width * 0.5;
        let offset = match (cluster.rtl, leading) {
            (false, true) | (true, false) => cluster.range.start,
            (false, false) | (true, true) => cluster.range.end,
        };
        Some((cluster.source, offset))
    }

    pub(crate) fn find_text_range(&self, needle: &str) -> Option<TextRange<Id>> {
        let needle = needle.to_ascii_lowercase();
        if needle.is_empty() {
            return None;
        }
        self.text_order.iter().find_map(|source| {
            let text = self.text_values.get(source)?;
            let start = text.to_ascii_lowercase().find(&needle)?;
            Some(TextRange {
                anchor_node: *source,
                anchor_offset: start,
                focus_node: *source,
                focus_offset: start + needle.len(),
            })
        })
    }

    /// Resolve a parsed URL Text Directive against the retained logical text.
    ///
    /// Text sources in one retained inline group stay contiguous. A group
    /// boundary contributes a newline, which lets range ends and context cross
    /// blocks through whitespace without allowing one directive term to bridge
    /// non-whitespace content that Livery never shaped together.
    pub(crate) fn find_text_directive_range(
        &self,
        directive: &TextDirective,
    ) -> Option<TextRange<Id>> {
        let mut logical = String::new();
        let mut segments = Vec::new();
        let mut previous_group = None;
        for source in &self.text_order {
            let text = self.text_values.get(source)?;
            let group = self.text_groups.get(source).copied();
            if !logical.is_empty() && previous_group != group {
                logical.push('\n');
            }
            let start = logical.len();
            logical.push_str(text);
            let end = logical.len();
            if start != end {
                segments.push((start, end, *source));
            }
            previous_group = group;
        }
        if logical.is_empty() {
            return None;
        }

        let whitespace = |character: char| character.is_whitespace();
        let has_prefix = |start: usize| {
            directive.prefix.as_ref().is_none_or(|prefix| {
                logical[..start]
                    .trim_end_matches(whitespace)
                    .ends_with(prefix)
            })
        };
        let has_suffix = |end: usize| {
            directive.suffix.as_ref().is_none_or(|suffix| {
                logical[end..]
                    .trim_start_matches(whitespace)
                    .starts_with(suffix)
            })
        };

        let mut search_start = 0;
        while search_start < logical.len() {
            let relative = logical[search_start..].find(&directive.start)?;
            let range_start = search_start + relative;
            let after_start = range_start + directive.start.len();
            if has_prefix(range_start) {
                let range_end = match directive.end.as_deref() {
                    Some(end_term) => {
                        let mut end_search_start = after_start;
                        let mut found = None;
                        while end_search_start < logical.len() {
                            let Some(relative_end) = logical[end_search_start..].find(end_term)
                            else {
                                break;
                            };
                            let end_start = end_search_start + relative_end;
                            let end = end_start + end_term.len();
                            if has_suffix(end) {
                                found = Some(end);
                                break;
                            }
                            end_search_start = next_char_boundary(&logical, end_start);
                        }
                        found
                    },
                    None => has_suffix(after_start).then_some(after_start),
                };
                if let Some(range_end) = range_end
                    && let (Some(anchor), Some(focus)) = (
                        position_in_segment(&segments, range_start, false),
                        position_in_segment(&segments, range_end, true),
                    )
                {
                    return Some(TextRange {
                        anchor_node: anchor.0,
                        anchor_offset: anchor.1,
                        focus_node: focus.0,
                        focus_offset: focus.1,
                    });
                }
            }
            search_start = next_char_boundary(&logical, range_start);
        }
        None
    }

    pub(crate) fn caret_rect<F>(
        &self,
        source: Id,
        offset: usize,
        mut transform: F,
    ) -> Option<TextRect>
    where
        F: FnMut(Id, Fragment) -> TextRect,
    {
        let cluster = self
            .text_clusters
            .iter()
            .filter(|cluster| cluster.source == source)
            // Distance from the offset to the cluster, zero when inside it.
            .min_by_key(|cluster| {
                cluster
                    .range
                    .start
                    .saturating_sub(offset)
                    .max(offset.saturating_sub(cluster.range.end))
            })?;
        let rect = transform(source, cluster.fragment);
        let length = cluster.range.end.saturating_sub(cluster.range.start);
        let fraction = if length == 0 {
            0.0
        } else {
            (offset.clamp(cluster.range.start, cluster.range.end) - cluster.range.start) as f32
                / length as f32
        };
        let fraction = if cluster.rtl {
            1.0 - fraction
        } else {
            fraction
        };
        Some(TextRect {
            x: rect.x + rect.width * fraction,
            y: rect.y,
            width: 1.0,
            height: rect.height,
        })
    }

    pub(crate) fn text_selection<F>(
        &self,
        range: TextRange<Id>,
        mut transform: F,
    ) -> Option<TextSelection<Id>>
    where
        F: FnMut(Id, Fragment) -> TextRect,
    {
        let anchor_index = self
            .text_order
            .iter()
            .position(|source| *source == range.anchor_node)?;
        let focus_index = self
            .text_order
            .iter()
            .position(|source| *source == range.focus_node)?;
        let ((start_index, start_offset), (end_index, end_offset)) = if anchor_index < focus_index
            || (anchor_index == focus_index && range.anchor_offset <= range.focus_offset)
        {
            (
                (anchor_index, range.anchor_offset),
                (focus_index, range.focus_offset),
            )
        } else {
            (
                (focus_index, range.focus_offset),
                (anchor_index, range.anchor_offset),
            )
        };
        if start_index == end_index && start_offset == end_offset {
            return None;
        }

        let ordered = TextRange {
            anchor_node: self.text_order[start_index],
            anchor_offset: start_offset,
            focus_node: self.text_order[end_index],
            focus_offset: end_offset,
        };
        let mut selected = Vec::new();
        let mut text = String::new();
        let mut previous_group = None;
        for index in start_index..=end_index {
            let source = self.text_order[index];
            let value = self.text_values.get(&source)?;
            let start = if index == start_index {
                start_offset.min(value.len())
            } else {
                0
            };
            let end = if index == end_index {
                end_offset.min(value.len())
            } else {
                value.len()
            };
            if start >= end {
                continue;
            }
            if let Some(slice) = value.get(start..end) {
                let group = self.text_groups.get(&source).copied();
                if !text.is_empty() && previous_group != group {
                    text.push('\n');
                }
                text.push_str(slice);
                previous_group = group;
                selected.push((source, start, end));
            }
        }
        if text.is_empty() {
            return None;
        }

        let mut rects = Vec::new();
        for cluster in &self.text_clusters {
            let Some((_, start, end)) = selected
                .iter()
                .find(|(source, _, _)| *source == cluster.source)
            else {
                continue;
            };
            let start = (*start).max(cluster.range.start);
            let end = (*end).min(cluster.range.end);
            if start >= end {
                continue;
            }
            let mut rect = transform(cluster.source, cluster.fragment);
            let length = cluster.range.end.saturating_sub(cluster.range.start);
            if length > 0 {
                let start_fraction = (start - cluster.range.start) as f32 / length as f32;
                let end_fraction = (end - cluster.range.start) as f32 / length as f32;
                let (left, right) = if cluster.rtl {
                    (1.0 - end_fraction, 1.0 - start_fraction)
                } else {
                    (start_fraction, end_fraction)
                };
                rect.x += rect.width * left;
                rect.width *= right - left;
            }
            if rect.width > 0.0 && rect.height > 0.0 {
                rects.push(rect);
            }
        }
        rects.sort_by(|left, right| {
            left.y
                .total_cmp(&right.y)
                .then_with(|| left.x.total_cmp(&right.x))
        });
        let mut merged = Vec::<TextRect>::new();
        for rect in rects {
            if let Some(previous) = merged.last_mut()
                && (previous.y - rect.y).abs() <= 0.5
                && (previous.height - rect.height).abs() <= 0.5
                && rect.x <= previous.x + previous.width + 0.5
            {
                let right = (previous.x + previous.width).max(rect.x + rect.width);
                previous.x = previous.x.min(rect.x);
                previous.width = right - previous.x;
                continue;
            }
            merged.push(rect);
        }
        if merged.is_empty() {
            return None;
        }
        Some(TextSelection {
            range: ordered,
            source_nodes: selected.iter().map(|(source, _, _)| *source).collect(),
            rects: merged,
            text,
        })
    }
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    index
        .checked_add(text[index..].chars().next().map_or(1, char::len_utf8))
        .unwrap_or(text.len())
        .min(text.len())
}

fn position_in_segment<Id: Copy>(
    segments: &[(usize, usize, Id)],
    position: usize,
    is_end: bool,
) -> Option<(Id, usize)> {
    segments.iter().find_map(|(start, end, source)| {
        let contains = if is_end {
            *start < position && position <= *end
        } else {
            *start <= position && position < *end
        };
        contains.then_some((*source, position - *start))
    })
}

#[derive(Clone, Debug)]
struct PreparedCommand<Id> {
    source: Id,
    owners: Vec<Id>,
    command: PaintCmd,
}

#[derive(Clone, Debug)]
struct RetainedTextCluster<Id> {
    source: Id,
    range: Range<usize>,
    fragment: Fragment,
    rtl: bool,
}

struct SourceSpan<Id> {
    source: Option<Id>,
    owners: Vec<Id>,
    style: ComputedValues,
    range: Range<usize>,
}

fn append_generated_marker<Id>(
    text: &mut String,
    spans: &mut Vec<SourceSpan<Id>>,
    source: Id,
    owners: &[Id],
    style: &ComputedValues,
    marker: &str,
) where
    Id: Copy,
{
    let isolate = style.list_style_position == ListStylePosition::Inside
        && style.list_style_type == ListStyleType::Decimal
        && style.direction == Direction::Rtl
        && !style.writing_mode.is_vertical();
    if isolate {
        append_generated_marker_control(text, spans, owners, style, '\u{2067}');
    }

    let start = text.len();
    text.push_str(marker);
    if text.len() != start {
        spans.push(SourceSpan {
            source: Some(source),
            owners: owners.to_vec(),
            style: style.clone(),
            range: start..text.len(),
        });
    }

    if isolate {
        append_generated_marker_control(text, spans, owners, style, '\u{2069}');
    }
}

fn append_generated_marker_control<Id>(
    text: &mut String,
    spans: &mut Vec<SourceSpan<Id>>,
    owners: &[Id],
    style: &ComputedValues,
    control: char,
) where
    Id: Copy,
{
    let start = text.len();
    text.push(control);
    spans.push(SourceSpan {
        source: None,
        owners: owners.to_vec(),
        style: style.clone(),
        range: start..text.len(),
    });
}

#[derive(Clone)]
struct TextSource<Id> {
    source: Id,
    range: Range<usize>,
}

struct InlineAtom<Id> {
    source: Id,
    owners: Vec<Id>,
    index: usize,
    fragment: Fragment,
    line_width: f32,
    line_box_height: f32,
    /// First baseline from this atom's margin-box block-start. Non-table
    /// atomic boxes keep their prior block-end fallback here.
    baseline: f32,
    margin_left: f32,
    margin_top: f32,
    edge: bool,
    paint: bool,
    marker: bool,
    /// An empty inline element standing in for its own box on the line; its
    /// extent comes from its font, not from `line_box_height`.
    empty_line: bool,
    vertical_align: VerticalAlign,
    font_size: f32,
    line_height: f32,
}

/// What shaping one inline formatting context produced.
struct Shaped<Id> {
    items: Vec<ShapedItem<Id>>,
    /// The block extent of the line boxes that end in a preserved newline or
    /// forced break. Such a line box may hold no item, yet CSS 2.1 section
    /// 9.4.2 gives it height; an empty line after a trailing break has none.
    break_lines: Option<(f32, f32)>,
    /// The first and last baselines of the line boxes that hold anything,
    /// a line of atoms alone or of a forced break alone included.
    baselines: Option<(f32, f32)>,
}

enum ShapedItem<Id> {
    Text(ShapedRun<Id>),
    InlineBox {
        source: Id,
        owners: Vec<Id>,
        fragment: Fragment,
        line_fragment: Fragment,
        edge: bool,
        paint: bool,
        line_y: f32,
    },
}

struct ShapedRun<Id> {
    source: Option<Id>,
    owners: Vec<Id>,
    trailing_content_end: Option<f32>,
    line_y: f32,
    fragment: Fragment,
    line_fragment: Fragment,
    font_instance: FontInstanceKey,
    font_size: f32,
    color: ColorF,
    glyphs: Vec<GlyphInstance>,
    clusters: Vec<ShapedCluster<Id>>,
}

/// A positioned inline whose first in-flow content wraps to the next line
/// still owns an empty first fragment at the preceding line's content end.
/// Keep this as retained geometry after line breaking instead of an in-flow
/// Parley atom: an atom would make a preceding collapsible space significant
/// and shift the CSS fragment start.
fn append_positioned_inline_start_markers<Id>(
    items: &mut Vec<ShapedItem<Id>>,
    inline_boxes: &[InlineAtom<Id>],
) where
    Id: Copy + Eq,
{
    for marker in inline_boxes.iter().filter(|inline_box| inline_box.marker) {
        let Some((first_index, first_line)) =
            items
                .iter()
                .enumerate()
                .find_map(|(index, item)| match item {
                    ShapedItem::Text(run) if run.owners.contains(&marker.source) => {
                        Some((index, run.line_y))
                    },
                    ShapedItem::InlineBox {
                        source,
                        owners,
                        line_y,
                        ..
                    } if *source == marker.source || owners.contains(&marker.source) => {
                        Some((index, *line_y))
                    },
                    ShapedItem::Text(_) | ShapedItem::InlineBox { .. } => None,
                })
        else {
            continue;
        };
        let Some(previous_line) = items
            .iter()
            .map(shaped_item_line_y)
            .filter(|line_y| *line_y < first_line - 0.5)
            .max_by(|left, right| left.total_cmp(right))
        else {
            continue;
        };
        let Some(content_end) = items
            .iter()
            .filter(|item| (shaped_item_line_y(item) - previous_line).abs() <= 0.5)
            .map(shaped_item_inline_end)
            .max_by(|left, right| left.total_cmp(right))
        else {
            continue;
        };
        items.insert(
            first_index,
            ShapedItem::InlineBox {
                source: marker.source,
                owners: marker.owners.clone(),
                fragment: Fragment {
                    x: content_end,
                    y: previous_line,
                    ..Fragment::default()
                },
                line_fragment: Fragment {
                    x: content_end,
                    y: previous_line,
                    ..Fragment::default()
                },
                edge: false,
                paint: true,
                line_y: previous_line,
            },
        );
    }
}

fn shaped_item_line_y<Id>(item: &ShapedItem<Id>) -> f32 {
    match item {
        ShapedItem::Text(run) => run.line_y,
        ShapedItem::InlineBox { line_y, .. } => *line_y,
    }
}

fn shaped_item_inline_end<Id>(item: &ShapedItem<Id>) -> f32 {
    match item {
        ShapedItem::Text(run) => run.trailing_content_end.unwrap_or(run.line_fragment.x),
        ShapedItem::InlineBox { line_fragment, .. } => line_fragment.x + line_fragment.width,
    }
}

#[derive(Clone)]
struct ShapedCluster<Id> {
    source: Id,
    range: Range<usize>,
    fragment: Fragment,
    rtl: bool,
}

fn translate_fragment(fragment: &mut Fragment, origin: (f32, f32)) {
    fragment.x += origin.0;
    fragment.y += origin.1;
}

fn collect_subtree_nodes<D>(dom: &D, node: D::NodeId, nodes: &mut HashSet<D::NodeId>)
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    nodes.insert(node);
    for child in dom.flat_children(node) {
        collect_subtree_nodes(dom, child, nodes);
    }
}

fn translate_paint_command(command: &mut PaintCmd, offset: (f32, f32)) {
    let PaintCmd::DrawText(run) = command else {
        return;
    };
    run.placement.bounds.min.x += offset.0;
    run.placement.bounds.max.x += offset.0;
    run.placement.bounds.min.y += offset.1;
    run.placement.bounds.max.y += offset.1;
    for glyph in &mut run.glyphs {
        glyph.point.x += offset.0;
        glyph.point.y += offset.1;
    }
}

fn is_inline<D>(dom: &D, styles: &StylePlane<D::NodeId>, id: D::NodeId) -> bool
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    match dom.kind(id) {
        NodeKind::Text => true,
        NodeKind::Element => styles.get(id).is_some_and(|style| {
            matches!(style.display, Display::Inline | Display::InlineBlock)
                && !matches!(style.position, Position::Absolute | Position::Fixed)
                && !(style.display == Display::Inline
                    && dom.flat_children(id).any(|child| {
                        !is_inline(dom, styles, child)
                            && !styles
                                .get(child)
                                .is_some_and(|child_style| child_style.display == Display::None)
                    }))
        }),
        _ => false,
    }
}

fn is_replaced_element<D>(dom: &D, id: D::NodeId) -> bool
where
    D: LayoutDom,
    D::NodeId: Copy,
{
    dom.kind(id) == NodeKind::Element
        && dom.element_name(id).is_some_and(|name| {
            name.local.as_ref().eq_ignore_ascii_case("img")
                || name.local.as_ref().eq_ignore_ascii_case("canvas")
                || name.local.as_ref().eq_ignore_ascii_case("iframe")
        })
}

fn is_forced_line_break<D>(dom: &D, id: D::NodeId) -> bool
where
    D: LayoutDom,
{
    dom.kind(id) == NodeKind::Element
        && dom
            .element_name(id)
            .is_some_and(|name| name.local.as_ref().eq_ignore_ascii_case("br"))
}

struct BoxInlineCollector<'a, D, F>
where
    D: LayoutDom,
    F: FragmentLookup<D::NodeId>,
{
    dom: &'a D,
    styles: &'a StylePlane<D::NodeId>,
    boxes: &'a CssBoxTree<D::NodeId>,
    fragments: &'a F,
    owners: &'a mut Vec<BoxId>,
    text: &'a mut String,
    spans: &'a mut Vec<SourceSpan<BoxId>>,
    inline_boxes: &'a mut Vec<InlineAtom<BoxId>>,
    inline_styles: &'a mut HashMap<BoxId, ComputedValues>,
    percentage_basis: f32,
    intrinsic_kind: Option<IntrinsicSizeKind>,
}

impl<D, F> BoxInlineCollector<'_, D, F>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
    F: FragmentLookup<D::NodeId>,
{
    fn collect(&mut self, box_id: BoxId, inherited: &ComputedValues) {
        let css_box = &self.boxes[box_id];
        match css_box.origin {
            BoxOrigin::Text(node) => {
                let start = self.text.len();
                append_inline_text(self.text, self.dom.text(node).unwrap_or(""), inherited);
                if self.text.len() == start {
                    return;
                }
                self.spans.push(SourceSpan {
                    source: Some(box_id),
                    owners: self.owners.clone(),
                    style: inherited.clone(),
                    range: start..self.text.len(),
                });
            },
            BoxOrigin::Element(node) => {
                if matches!(
                    css_box.positioning,
                    buckram::PositioningScheme::Absolute | buckram::PositioningScheme::Fixed
                ) {
                    // The enclosing inline run supplies this box's static
                    // source, but an out-of-flow subtree never contributes
                    // text, line width, or line height to that run. Livery's
                    // layout bridge formats the root separately for K5d.
                    return;
                }
                let Some(style) = self.styles.get(node).cloned() else {
                    return;
                };
                if style.display == Display::None {
                    return;
                }
                if is_forced_line_break(self.dom, node) {
                    self.push_forced_line_break(box_id, &style);
                    return;
                }
                let atomic = css_box.replaced
                    || (css_box.display.outside == Some(DisplayOutside::Inline)
                        && css_box.display.inside == Some(DisplayInside::FlowRoot));
                if atomic {
                    self.push_atomic_box(box_id, &style);
                    return;
                }

                let ancestor_owners = self.owners.clone();
                let text_start = self.text.len();
                self.push_edge(box_id, &style, &ancestor_owners, true);
                self.push_positioned_start_marker(box_id, &style, &ancestor_owners);
                let content_start = self.inline_boxes.len();
                self.owners.push(box_id);
                for child in css_box.children() {
                    self.collect(*child, &style);
                }
                self.owners.pop();
                let has_inline_content = self.inline_boxes.len() > content_start;
                self.push_edge(box_id, &style, &ancestor_owners, false);
                if self.text.len() == text_start && !has_inline_content {
                    self.push_empty_line_box(box_id, &style, &ancestor_owners);
                }
                self.inline_styles.insert(box_id, style);
            },
            BoxOrigin::Anonymous { .. } => {
                // K4e4: an inline-table's wrapper is the atom that occupies
                // line space - the box that carries the element's margins
                // (CSS 2.1 section 17.4) and contains its captions. Without
                // this arm the wrapper would be walked as an ordinary inline
                // container and the grid's blocks shredded into the line.
                if css_box.display.internal_table == Some(InternalTableRole::Wrapper)
                    && css_box.display.outside == Some(DisplayOutside::Inline)
                    && let Some(owner) =
                        css_box.origin.node().and_then(|node| self.styles.get(node))
                {
                    if owner.display == Display::None {
                        return;
                    }
                    let mut style = crate::table_wrapper::wrapper_style(owner);
                    // Alignment in the line is the table element's own
                    // vertical-align; it is not on the wrapper's migrated
                    // property list, and for_child reset it.
                    style.vertical_align = owner.vertical_align;
                    self.push_atomic_box(box_id, &style);
                    return;
                }
                for child in css_box.children() {
                    self.collect(*child, inherited);
                }
            },
            BoxOrigin::Pseudo {
                owner,
                pseudo: buckram::PseudoElement::Marker,
            } => {
                let Some(style) = self.styles.get(owner) else {
                    return;
                };
                let Some(marker) = inside_marker_text(self.dom, self.styles, owner, style) else {
                    return;
                };
                // Marker pseudo-elements preserve an authored string verbatim.
                append_generated_marker(
                    self.text,
                    self.spans,
                    box_id,
                    self.owners,
                    style,
                    &marker,
                );
            },
            BoxOrigin::Pseudo { .. } => {},
        }
    }

    /// One atomic inline box: its whole subtree was laid out separately and
    /// its rectangle occupies line space as a unit.
    fn push_atomic_box(&mut self, box_id: BoxId, style: &ComputedValues) {
        let Some(mut fragment) = self.fragments.atomic_box_rect(box_id).copied() else {
            return;
        };
        if let Some(kind) = self.intrinsic_kind
            && let Some(intrinsic) = self.fragments.atomic_box_intrinsic_inline(box_id)
        {
            fragment.width = intrinsic.get(kind);
        }
        let font_size = super::paint::used_font_size(style);
        let (line_width, line_box_height, margin_left, margin_top) =
            inline_margin_box(style, fragment, font_size, self.percentage_basis);
        let exported_baseline = self
            .fragments
            .atomic_box_baseline(box_id)
            .filter(|baseline| baseline.is_finite() && *baseline >= 0.0);
        let baseline = exported_baseline.map_or(line_box_height, |baseline| margin_top + baseline);
        self.inline_boxes.push(InlineAtom {
            source: box_id,
            owners: self.owners.clone(),
            index: self.text.len(),
            fragment,
            line_width,
            line_box_height,
            baseline,
            margin_left,
            margin_top,
            edge: false,
            paint: true,
            marker: false,
            empty_line: false,
            vertical_align: style.vertical_align,
            font_size,
            line_height: super::layout::line_height_px(&style.line_height, font_size),
        });
    }

    fn push_edge(&mut self, source: BoxId, style: &ComputedValues, owners: &[BoxId], start: bool) {
        let edges = inline_decoration_edges(style, self.percentage_basis);
        let em = super::paint::used_font_size(style);
        let margin = inline_axis_margin(style, start, self.percentage_basis);
        let decoration = inline_axis_decoration(style, edges, start);
        let mut push = |width: f32, paint: bool| {
            if width.is_finite() && width.abs() > f32::EPSILON {
                self.inline_boxes.push(InlineAtom {
                    source,
                    owners: owners.to_vec(),
                    index: self.text.len(),
                    fragment: Fragment {
                        width,
                        ..Fragment::default()
                    },
                    line_width: width,
                    line_box_height: 0.0,
                    baseline: 0.0,
                    margin_left: 0.0,
                    margin_top: 0.0,
                    edge: true,
                    paint,
                    marker: false,
                    empty_line: false,
                    vertical_align: style.vertical_align,
                    font_size: em,
                    line_height: super::layout::line_height_px(&style.line_height, em),
                });
            }
        };
        if start {
            push(margin, false);
        }
        push(decoration, true);
        if !start {
            push(margin, false);
        }
    }

    /// Keep the first content edge of a positioned inline even when its first
    /// visible content wraps to a following line. This zero-width atom does
    /// not paint or consume line width; it gives the retained fragment tree
    /// the empty first box fragment CSS Positioned Layout uses for the
    /// containing block of an absolute descendant.
    fn push_positioned_start_marker(
        &mut self,
        source: BoxId,
        style: &ComputedValues,
        owners: &[BoxId],
    ) {
        if !matches!(
            self.boxes[source].positioning,
            buckram::PositioningScheme::Relative | buckram::PositioningScheme::Sticky
        ) {
            return;
        }
        let font_size = super::paint::used_font_size(style);
        let line_height = super::layout::line_height_px(&style.line_height, font_size);
        self.inline_boxes.push(InlineAtom {
            source,
            owners: owners.to_vec(),
            index: self.text.len(),
            fragment: Fragment::default(),
            line_width: 0.0,
            line_box_height: line_height,
            baseline: line_height,
            margin_left: 0.0,
            margin_top: 0.0,
            edge: false,
            paint: true,
            marker: true,
            empty_line: false,
            vertical_align: style.vertical_align,
            font_size,
            line_height,
        });
    }

    fn push_empty_line_box(&mut self, source: BoxId, style: &ComputedValues, owners: &[BoxId]) {
        if matches!(style.line_height, CssLineHeight::Normal) {
            return;
        }
        let font_size = super::paint::used_font_size(style);
        let height = super::layout::line_height_px(&style.line_height, font_size);
        if height <= 0.0 {
            return;
        }
        let retain_source_fragment = matches!(
            self.boxes[source].positioning,
            buckram::PositioningScheme::Relative | buckram::PositioningScheme::Sticky
        );
        self.inline_boxes.push(InlineAtom {
            source,
            owners: owners.to_vec(),
            index: self.text.len(),
            fragment: Fragment {
                width: 0.0,
                height,
                ..Fragment::default()
            },
            line_width: 0.0,
            line_box_height: height,
            baseline: height,
            margin_left: 0.0,
            margin_top: 0.0,
            edge: false,
            // An empty positioned inline still establishes the containing
            // block for an absolute descendant. `paint` also controls source
            // fragment retention for an atom; keep this zero-width line
            // rectangle without making it consume inline space.
            paint: retain_source_fragment,
            marker: false,
            empty_line: true,
            vertical_align: style.vertical_align,
            font_size,
            line_height: height,
        });
    }

    fn push_forced_line_break(&mut self, _source: BoxId, style: &ComputedValues) {
        let start = append_forced_line_break(self.text);
        push_forced_break_span(self.spans, self.owners, style, start..self.text.len());
    }
}

/// The admitted inside-marker slice follows HTML ordered-list ordinals. Outside
/// markers retain their separate formatting path.
pub(crate) fn inside_marker_text<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    owner: D::NodeId,
    style: &ComputedValues,
) -> Option<String>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    if style.list_style_position != ListStylePosition::Inside {
        return None;
    }
    match &style.list_style_type {
        ListStyleType::None => None,
        ListStyleType::Disc => Some("• ".to_owned()),
        ListStyleType::Decimal => decimal_inside_marker_text(dom, styles, owner),
        ListStyleType::String(value) => Some(value.clone()),
    }
}

fn decimal_inside_marker_text<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    item: D::NodeId,
) -> Option<String>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    if !is_html_element(dom, item, "li") {
        return None;
    }
    let list = dom.parent(item)?;
    if !is_html_element(dom, list, "ol") || html_attribute(dom, list, "reversed").is_some() {
        return None;
    }

    let mut ordinal = html_integer_attribute(dom, list, "start").unwrap_or(1);
    for sibling in dom.dom_children(list) {
        if !is_html_element(dom, sibling, "li")
            || styles
                .get(sibling)
                .is_none_or(|style| style.display != Display::ListItem)
        {
            continue;
        }
        if let Some(value) = html_integer_attribute(dom, sibling, "value") {
            ordinal = value;
        }
        if sibling == item {
            return Some(format!("{ordinal}. "));
        }
        ordinal = ordinal.checked_add(1)?;
    }
    None
}

fn is_html_element<D>(dom: &D, id: D::NodeId, local: &str) -> bool
where
    D: LayoutDom,
{
    dom.kind(id) == NodeKind::Element
        && dom.element_name(id).is_some_and(|name| {
            name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
                && name.local.as_ref().eq_ignore_ascii_case(local)
        })
}

fn html_attribute<'a, D>(dom: &'a D, id: D::NodeId, local: &str) -> Option<&'a str>
where
    D: LayoutDom,
{
    dom.attribute(id, &Namespace::from(""), &LocalName::from(local))
}

fn html_integer_attribute<D>(dom: &D, id: D::NodeId, local: &str) -> Option<i64>
where
    D: LayoutDom,
{
    html_attribute(dom, id, local).and_then(crate::presentational_hints::parse_integer)
}

struct InlineCollector<'a, D, F>
where
    D: LayoutDom,
    F: FragmentLookup<D::NodeId>,
{
    dom: &'a D,
    styles: &'a StylePlane<D::NodeId>,
    fragments: &'a F,
    already_prepared: &'a HashSet<D::NodeId>,
    owners: &'a mut Vec<D::NodeId>,
    text: &'a mut String,
    spans: &'a mut Vec<SourceSpan<D::NodeId>>,
    inline_boxes: &'a mut Vec<InlineAtom<D::NodeId>>,
    inline_styles: &'a mut HashMap<D::NodeId, ComputedValues>,
    percentage_basis: f32,
}

impl<D, F> InlineCollector<'_, D, F>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
    F: FragmentLookup<D::NodeId>,
{
    fn collect(&mut self, id: D::NodeId, inherited: &ComputedValues) {
        match self.dom.kind(id) {
            NodeKind::Text => {
                if self.already_prepared.contains(&id) {
                    return;
                }
                let start = self.text.len();
                append_inline_text(self.text, self.dom.text(id).unwrap_or(""), inherited);
                if self.text.len() == start {
                    return;
                }
                self.spans.push(SourceSpan {
                    source: Some(id),
                    owners: self.owners.clone(),
                    style: inherited.clone(),
                    range: start..self.text.len(),
                });
            },
            NodeKind::Element => {
                if self.already_prepared.contains(&id) {
                    return;
                }
                let Some(style) = self.styles.get(id).cloned() else {
                    return;
                };
                if style.display == Display::None {
                    return;
                }
                if is_forced_line_break(self.dom, id) {
                    self.push_forced_line_break(id, &style);
                    return;
                }
                if is_replaced_element(self.dom, id) && style.display != Display::InlineBlock {
                    if let Some(fragment) = self.fragments.atomic_rect(id).copied() {
                        let font_size = super::paint::used_font_size(&style);
                        let (line_width, line_box_height, margin_left, margin_top) =
                            inline_margin_box(&style, fragment, font_size, self.percentage_basis);
                        self.inline_boxes.push(InlineAtom {
                            source: id,
                            owners: self.owners.clone(),
                            index: self.text.len(),
                            fragment,
                            line_width,
                            line_box_height,
                            baseline: line_box_height,
                            margin_left,
                            margin_top,
                            edge: false,
                            paint: true,
                            marker: false,
                            empty_line: false,
                            vertical_align: style.vertical_align,
                            font_size,
                            line_height: super::layout::line_height_px(
                                &style.line_height,
                                font_size,
                            ),
                        });
                    }
                    return;
                }
                if style.display == Display::InlineBlock {
                    if let Some(fragment) = self.fragments.atomic_rect(id).copied() {
                        let font_size = super::paint::used_font_size(&style);
                        let (line_width, line_box_height, margin_left, margin_top) =
                            inline_margin_box(&style, fragment, font_size, self.percentage_basis);
                        self.inline_boxes.push(InlineAtom {
                            source: id,
                            owners: self.owners.clone(),
                            index: self.text.len(),
                            fragment,
                            line_width,
                            line_box_height,
                            baseline: line_box_height,
                            margin_left,
                            margin_top,
                            edge: false,
                            paint: true,
                            marker: false,
                            empty_line: false,
                            vertical_align: style.vertical_align,
                            font_size: super::paint::used_font_size(&style),
                            line_height: super::layout::line_height_px(
                                &style.line_height,
                                super::paint::used_font_size(&style),
                            ),
                        });
                    }
                    return;
                }
                let ancestor_owners = self.owners.clone();
                let text_start = self.text.len();
                self.push_edge(id, &style, &ancestor_owners, true);
                self.push_positioned_start_marker(id, &style, &ancestor_owners);
                let content_start = self.inline_boxes.len();
                self.owners.push(id);
                for child in self.dom.flat_children(id) {
                    if is_inline(self.dom, self.styles, child) {
                        self.collect(child, &style);
                    }
                }
                self.owners.pop();
                let has_inline_content = self.inline_boxes.len() > content_start;
                self.push_edge(id, &style, &ancestor_owners, false);
                if self.text.len() == text_start && !has_inline_content {
                    self.push_empty_line_box(id, &style, &ancestor_owners);
                }
                self.inline_styles.insert(id, style);
            },
            _ => {},
        }
    }
}

impl<D, F> InlineCollector<'_, D, F>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
    F: FragmentLookup<D::NodeId>,
{
    fn push_edge(
        &mut self,
        source: D::NodeId,
        style: &ComputedValues,
        owners: &[D::NodeId],
        start: bool,
    ) {
        let edges = inline_decoration_edges(style, self.percentage_basis);
        let em = super::paint::used_font_size(style);
        let margin = inline_axis_margin(style, start, self.percentage_basis);
        let decoration = inline_axis_decoration(style, edges, start);
        let mut push = |width: f32, paint: bool| {
            if width.is_finite() && width.abs() > f32::EPSILON {
                self.inline_boxes.push(InlineAtom {
                    source,
                    owners: owners.to_vec(),
                    index: self.text.len(),
                    fragment: Fragment {
                        width,
                        ..Fragment::default()
                    },
                    line_width: width,
                    line_box_height: 0.0,
                    baseline: 0.0,
                    margin_left: 0.0,
                    margin_top: 0.0,
                    edge: true,
                    paint,
                    marker: false,
                    empty_line: false,
                    vertical_align: style.vertical_align,
                    font_size: em,
                    line_height: super::layout::line_height_px(&style.line_height, em),
                });
            }
        };
        if start {
            push(margin, false);
        }
        push(decoration, true);
        if !start {
            push(margin, false);
        }
    }

    fn push_positioned_start_marker(
        &mut self,
        source: D::NodeId,
        style: &ComputedValues,
        owners: &[D::NodeId],
    ) {
        if !matches!(style.position, Position::Relative | Position::Sticky) {
            return;
        }
        let font_size = super::paint::used_font_size(style);
        let line_height = super::layout::line_height_px(&style.line_height, font_size);
        self.inline_boxes.push(InlineAtom {
            source,
            owners: owners.to_vec(),
            index: self.text.len(),
            fragment: Fragment::default(),
            line_width: 0.0,
            line_box_height: line_height,
            baseline: line_height,
            margin_left: 0.0,
            margin_top: 0.0,
            edge: false,
            paint: true,
            marker: true,
            empty_line: false,
            vertical_align: style.vertical_align,
            font_size,
            line_height,
        });
    }

    fn push_empty_line_box(
        &mut self,
        source: D::NodeId,
        style: &ComputedValues,
        owners: &[D::NodeId],
    ) {
        if matches!(style.line_height, CssLineHeight::Normal) {
            return;
        }
        let font_size = super::paint::used_font_size(style);
        let height = super::layout::line_height_px(&style.line_height, font_size);
        if height <= 0.0 {
            return;
        }
        self.inline_boxes.push(InlineAtom {
            source,
            owners: owners.to_vec(),
            index: self.text.len(),
            fragment: Fragment {
                width: 0.0,
                height,
                ..Fragment::default()
            },
            line_width: 0.0,
            line_box_height: height,
            baseline: height,
            margin_left: 0.0,
            margin_top: 0.0,
            edge: false,
            paint: false,
            marker: false,
            empty_line: true,
            vertical_align: style.vertical_align,
            font_size,
            line_height: height,
        });
    }

    fn push_forced_line_break(&mut self, _source: D::NodeId, style: &ComputedValues) {
        let start = append_forced_line_break(self.text);
        push_forced_break_span(self.spans, self.owners, style, start..self.text.len());
    }
}

#[derive(Clone, Copy)]
struct InlineDecorationEdges {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

/// Resolve the physical margin that occupies the inline start or end edge.
/// Parley's line coordinates follow the CSS inline axis, not always physical
/// left-to-right, so vertical writing modes use top and bottom margins.
fn inline_axis_margin(style: &ComputedValues, start: bool, percentage_basis: f32) -> f32 {
    let margin = if style.writing_mode.is_vertical() {
        if start {
            style.margin_top
        } else {
            style.margin_bottom
        }
    } else if start {
        style.margin_left
    } else {
        style.margin_right
    };
    inline_margin_px(
        margin,
        super::paint::used_font_size(style),
        percentage_basis,
    )
}

/// Resolve the physical decoration that occupies the inline start or end
/// edge. Left and right borders belong to the block axis in vertical writing
/// modes and therefore must not widen the inline line box.
fn inline_axis_decoration(
    style: &ComputedValues,
    edges: InlineDecorationEdges,
    start: bool,
) -> f32 {
    if style.writing_mode.is_vertical() {
        if start { edges.top } else { edges.bottom }
    } else if start {
        edges.left
    } else {
        edges.right
    }
}

fn inline_decoration_edges(style: &ComputedValues, percentage_basis: f32) -> InlineDecorationEdges {
    let em = super::paint::used_font_size(style);
    InlineDecorationEdges {
        left: super::layout::length_percentage_px(style.padding_left.0, em, percentage_basis)
            + super::layout::border_width_px(style.border_left_style, style.border_left_width, em),
        right: super::layout::length_percentage_px(style.padding_right.0, em, percentage_basis)
            + super::layout::border_width_px(
                style.border_right_style,
                style.border_right_width,
                em,
            ),
        top: super::layout::length_percentage_px(style.padding_top.0, em, percentage_basis)
            + super::layout::border_width_px(style.border_top_style, style.border_top_width, em),
        bottom: super::layout::length_percentage_px(style.padding_bottom.0, em, percentage_basis)
            + super::layout::border_width_px(
                style.border_bottom_style,
                style.border_bottom_width,
                em,
            ),
    }
}

fn inline_margin_px(value: Margin, em: f32, percentage_basis: f32) -> f32 {
    match value {
        Margin::Auto => 0.0,
        Margin::Value(value) => {
            super::layout::signed_length_percentage_px(value, em, percentage_basis)
        },
    }
}

fn inline_margin_box(
    style: &ComputedValues,
    fragment: Fragment,
    font_size: f32,
    percentage_basis: f32,
) -> (f32, f32, f32, f32) {
    let margin_left = inline_margin_px(style.margin_left, font_size, percentage_basis);
    let margin_right = inline_margin_px(style.margin_right, font_size, percentage_basis);
    let margin_top = inline_margin_px(style.margin_top, font_size, percentage_basis);
    let margin_bottom = inline_margin_px(style.margin_bottom, font_size, percentage_basis);
    (
        (fragment.width + margin_left + margin_right).max(0.0),
        (fragment.height + margin_top + margin_bottom).max(0.0),
        margin_left,
        margin_top,
    )
}

fn decorated_inline_fragment<Id>(
    styles: &StylePlane<Id>,
    source: Id,
    mut fragment: Fragment,
    percentage_basis: f32,
) -> Fragment
where
    Id: Copy + Eq + Hash,
{
    let Some(style) = styles.get(source) else {
        return fragment;
    };
    let edges = inline_decoration_edges(style, percentage_basis);
    fragment.y -= edges.top;
    fragment.height += edges.top + edges.bottom;
    fragment
}

fn is_opening_hanging_punctuation(character: char) -> bool {
    matches!(
        character,
        '\u{0022}'
            | '\u{0027}'
            | '\u{0028}'
            | '\u{005b}'
            | '\u{007b}'
            | '\u{00ab}'
            | '\u{2018}'
            | '\u{201b}'
            | '\u{201c}'
            | '\u{201f}'
            | '\u{2039}'
            | '\u{2045}'
            | '\u{207d}'
            | '\u{208d}'
            | '\u{2308}'
            | '\u{230a}'
            | '\u{2329}'
            | '\u{2768}'..='\u{2775}'
            | '\u{27c5}'
            | '\u{27e6}'..='\u{27ef}'
            | '\u{2983}'..='\u{2998}'
            | '\u{29d8}'..='\u{29db}'
            | '\u{29fc}'
            | '\u{2e02}'
            | '\u{2e04}'
            | '\u{2e09}'
            | '\u{2e0c}'
            | '\u{2e1c}'
            | '\u{2e20}'
            | '\u{2e22}'
            | '\u{2e24}'
            | '\u{2e26}'
            | '\u{2e28}'
            | '\u{2e42}'
            | '\u{3008}'
            | '\u{300a}'
            | '\u{300c}'
            | '\u{300e}'
            | '\u{3010}'
            | '\u{3014}'
            | '\u{3016}'
            | '\u{3018}'
            | '\u{301a}'
            | '\u{301d}'
            | '\u{fd3e}'
            | '\u{fe17}'
            | '\u{fe35}'
            | '\u{fe37}'
            | '\u{fe39}'
            | '\u{fe3b}'
            | '\u{fe3d}'
            | '\u{fe3f}'
            | '\u{fe41}'
            | '\u{fe43}'
            | '\u{fe47}'
            | '\u{fe59}'
            | '\u{fe5b}'
            | '\u{fe5d}'
            | '\u{ff08}'
            | '\u{ff3b}'
            | '\u{ff5b}'
            | '\u{ff5f}'
            | '\u{ff62}'
    )
}

fn normalized_text<'a>(source: &'a str, style: &ComputedValues) -> Cow<'a, str> {
    use livery::values::WhiteSpaceCollapse;

    let normalized = if matches!(
        style.white_space_collapse,
        WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
    ) {
        source.to_owned()
    } else if style.white_space_collapse == WhiteSpaceCollapse::PreserveBreaks {
        let mut output = String::new();
        append_preserving_breaks(&mut output, source);
        output
    } else {
        collapse_css_whitespace(source)
    };
    Cow::Owned(transform_text(&normalized, style))
}

fn append_inline_text(target: &mut String, source: &str, style: &ComputedValues) {
    use livery::values::WhiteSpaceCollapse;

    if matches!(
        style.white_space_collapse,
        WhiteSpaceCollapse::Preserve | WhiteSpaceCollapse::BreakSpaces
    ) {
        target.push_str(&transform_text(source, style));
        return;
    }
    if style.white_space_collapse == WhiteSpaceCollapse::PreserveBreaks {
        let start = target.len();
        append_preserving_breaks(target, source);
        let normalized = target[start..].to_owned();
        target.truncate(start);
        target.push_str(&transform_text(&normalized, style));
        return;
    }

    let leading = source.chars().next().is_some_and(is_css_whitespace);
    let trailing = source.chars().next_back().is_some_and(is_css_whitespace);
    let mut normalized = String::new();
    if leading && !target.is_empty() && !target.ends_with(char::is_whitespace) {
        normalized.push(' ');
    }
    let collapsed = collapse_css_whitespace(source);
    if !collapsed.is_empty() {
        normalized.push_str(&collapsed);
    }
    if trailing
        && (!target.is_empty() || !normalized.is_empty())
        && !normalized.ends_with(char::is_whitespace)
        && !(normalized.is_empty() && target.ends_with(char::is_whitespace))
    {
        normalized.push(' ');
    }
    target.push_str(&transform_text(&normalized, style));
}

fn transform_text(source: &str, style: &ComputedValues) -> String {
    let without_soft_hyphens = if style.hyphens == Hyphens::None {
        Cow::Owned(
            source
                .chars()
                .filter(|character| *character != '\u{00ad}')
                .collect(),
        )
    } else {
        Cow::Borrowed(source)
    };
    let mut transformed = match style.text_transform.case {
        TextTransformCase::None | TextTransformCase::MathAuto => without_soft_hyphens.into_owned(),
        TextTransformCase::Uppercase => without_soft_hyphens.to_uppercase(),
        TextTransformCase::Lowercase => without_soft_hyphens.to_lowercase(),
        TextTransformCase::Capitalize => capitalize_text(&without_soft_hyphens),
    };
    if style.text_transform.full_width {
        transformed = transformed.chars().map(full_width_character).collect();
    }
    if style.text_transform.full_size_kana {
        transformed = transformed.chars().map(full_size_kana_character).collect();
    }
    transformed
}

fn capitalize_text(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut at_word_start = true;
    for character in source.chars() {
        let enclosed_alphanumeric = matches!(character, '\u{2460}'..='\u{24ff}');
        if character.is_alphanumeric() && !enclosed_alphanumeric {
            if at_word_start {
                output.extend(character.to_uppercase());
            } else {
                output.push(character);
            }
            at_word_start = false;
        } else {
            output.push(character);
            if character != '\'' && character != '\u{2019}' {
                at_word_start = true;
            }
        }
    }
    output
}

fn full_width_character(character: char) -> char {
    match character {
        ' ' => '\u{3000}',
        '!'..='~' => char::from_u32(character as u32 + 0xfee0).unwrap_or(character),
        _ => character,
    }
}

fn full_size_kana_character(character: char) -> char {
    match character {
        '\u{3041}' => '\u{3042}',
        '\u{3043}' => '\u{3044}',
        '\u{3045}' => '\u{3046}',
        '\u{3047}' => '\u{3048}',
        '\u{3049}' => '\u{304a}',
        '\u{3063}' => '\u{3064}',
        '\u{3083}' => '\u{3084}',
        '\u{3085}' => '\u{3086}',
        '\u{3087}' => '\u{3088}',
        '\u{308e}' => '\u{308f}',
        '\u{30a1}' => '\u{30a2}',
        '\u{30a3}' => '\u{30a4}',
        '\u{30a5}' => '\u{30a6}',
        '\u{30a7}' => '\u{30a8}',
        '\u{30a9}' => '\u{30aa}',
        '\u{30c3}' => '\u{30c4}',
        '\u{30e3}' => '\u{30e4}',
        '\u{30e5}' => '\u{30e6}',
        '\u{30e7}' => '\u{30e8}',
        '\u{30ee}' => '\u{30ef}',
        '\u{31f0}'..='\u{31ff}' => character,
        _ => character,
    }
}

fn append_preserving_breaks(target: &mut String, source: &str) {
    let mut pending_space = false;
    let mut previous_was_cr = false;
    for character in source.chars() {
        if character == '\n' && previous_was_cr {
            previous_was_cr = false;
            continue;
        }
        previous_was_cr = false;
        match character {
            '\n' | '\r' | '\u{000c}' => {
                while target.ends_with(' ') {
                    target.pop();
                }
                target.push('\n');
                pending_space = false;
                previous_was_cr = character == '\r';
            },
            '\t' | ' ' => pending_space = true,
            _ => {
                if pending_space && !target.is_empty() && !target.ends_with('\n') {
                    target.push(' ');
                }
                pending_space = false;
                target.push(character);
            },
        }
    }
    if pending_space && !target.is_empty() && !target.ends_with('\n') {
        target.push(' ');
    }
}

/// A forced break sits in its parent inline box on the line, as text does, so
/// that box's extent counts there; with no source it is not text content.
fn push_forced_break_span<Id>(
    spans: &mut Vec<SourceSpan<Id>>,
    owners: &[Id],
    style: &ComputedValues,
    range: Range<usize>,
) where
    Id: Copy,
{
    spans.push(SourceSpan {
        source: None,
        owners: owners.to_vec(),
        style: style.clone(),
        range,
    });
}

fn append_forced_line_break(target: &mut String) -> usize {
    let index = target.len();
    target.push('\n');
    index
}

fn is_css_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | ' '
    )
}

fn collapse_css_whitespace(source: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for character in source.chars() {
        if is_css_whitespace(character) {
            pending_space = true;
            continue;
        }
        if pending_space && !output.is_empty() {
            output.push(' ');
        }
        pending_space = false;
        output.push(character);
    }
    if pending_space && !output.is_empty() {
        output.push(' ');
    }
    output
}

fn push_defaults(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &ComputedValues,
    features: Vec<FontFeature>,
    intrinsic_kind: Option<IntrinsicSizeKind>,
    tab_stop: f32,
) {
    let font_size = super::paint::used_font_size(style);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(font_family(style));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight(
        style,
    ))));
    builder.push_default(StyleProperty::FontStyle(font_style(style)));
    builder.push_default(StyleProperty::Brush(brush(style, 0)));
    builder.push_default(line_height(style));
    builder.push_default(StyleProperty::FontFeatures(FontFeatures::List(Cow::Owned(
        features,
    ))));
    builder.push_default(StyleProperty::WordBreak(word_break(style)));
    builder.push_default(StyleProperty::OverflowWrap(overflow_wrap(
        style,
        intrinsic_kind,
    )));
    builder.push_default(StyleProperty::TextWrapMode(text_wrap_mode(style)));
    builder.push_default(StyleProperty::TabSize(tab_stop));
    if let Some(letter_spacing) = spacing_px(style.letter_spacing, font_size) {
        builder.push_default(StyleProperty::LetterSpacing(letter_spacing));
    }
    if let Some(word_spacing) = spacing_px(style.word_spacing, font_size) {
        builder.push_default(StyleProperty::WordSpacing(word_spacing));
    }
}

fn push_span(
    builder: &mut parley::RangedBuilder<'_, Brush>,
    style: &ComputedValues,
    range: Range<usize>,
    source_index: usize,
    features: Vec<FontFeature>,
    intrinsic_kind: Option<IntrinsicSizeKind>,
    tab_stop: f32,
) {
    let font_size = super::paint::used_font_size(style);
    builder.push(StyleProperty::FontSize(font_size), range.clone());
    builder.push(font_family(style), range.clone());
    builder.push(
        StyleProperty::FontWeight(FontWeight::new(font_weight(style))),
        range.clone(),
    );
    builder.push(StyleProperty::FontStyle(font_style(style)), range.clone());
    builder.push(
        StyleProperty::Brush(brush(style, source_index)),
        range.clone(),
    );
    builder.push(line_height(style), range.clone());
    builder.push(
        StyleProperty::FontFeatures(FontFeatures::List(Cow::Owned(features))),
        range.clone(),
    );
    builder.push(StyleProperty::WordBreak(word_break(style)), range.clone());
    builder.push(
        StyleProperty::OverflowWrap(overflow_wrap(style, intrinsic_kind)),
        range.clone(),
    );
    builder.push(
        StyleProperty::TextWrapMode(text_wrap_mode(style)),
        range.clone(),
    );
    builder.push(StyleProperty::TabSize(tab_stop), range.clone());
    if let Some(letter_spacing) = spacing_px(style.letter_spacing, font_size) {
        builder.push(StyleProperty::LetterSpacing(letter_spacing), range.clone());
    }
    if let Some(word_spacing) = spacing_px(style.word_spacing, font_size) {
        builder.push(StyleProperty::WordSpacing(word_spacing), range);
    }
}

fn effective_font_features(
    face_features: &HashMap<String, Box<[FontFeatureSetting]>>,
    style: &ComputedValues,
) -> Vec<FontFeature> {
    let mut features = BTreeMap::<[u8; 4], u16>::new();
    let mut set = |tag: [u8; 4], value: u16| {
        features.insert(tag, value);
    };

    if let CssFontFamily::Named(family) = &style.font_family
        && let Some(defaults) = face_features.get(&family.to_ascii_lowercase())
    {
        for setting in defaults.iter() {
            set(setting.tag, setting.value);
        }
    }

    if let Some(value) = style.font_variant_ligatures.common() {
        set(*b"liga", u16::from(value));
        set(*b"clig", u16::from(value));
    }
    if let Some(value) = style.font_variant_ligatures.discretionary() {
        set(*b"dlig", u16::from(value));
    }
    if let Some(value) = style.font_variant_ligatures.historical() {
        set(*b"hlig", u16::from(value));
    }
    if let Some(value) = style.font_variant_ligatures.contextual() {
        set(*b"calt", u16::from(value));
    }

    if spacing_px(style.letter_spacing, super::paint::used_font_size(style))
        .is_some_and(|spacing| spacing.abs() > f32::EPSILON)
    {
        for tag in [*b"liga", *b"clig", *b"dlig", *b"hlig", *b"calt"] {
            set(tag, 0);
        }
    }

    for setting in style.font_feature_settings.settings() {
        set(setting.tag, setting.value);
    }

    features
        .into_iter()
        .map(|(tag, value)| FontFeature::new(parley::setting::Tag::from_bytes(tag), value))
        .collect()
}

fn text_alignment(style: TextAlign, direction: Direction, justify: TextJustify) -> Alignment {
    if style == TextAlign::Justify && justify == TextJustify::None {
        return directional_alignment(TextAlign::Start, direction);
    }
    match style {
        TextAlign::Start | TextAlign::End => directional_alignment(style, direction),
        TextAlign::Left => Alignment::Left,
        TextAlign::Right => Alignment::Right,
        TextAlign::Center => Alignment::Center,
        TextAlign::Justify => Alignment::Justify,
        TextAlign::JustifyAll => Alignment::Justify,
    }
}

fn directional_alignment(style: TextAlign, direction: Direction) -> Alignment {
    match (style, direction) {
        (TextAlign::Start, Direction::Ltr) | (TextAlign::End, Direction::Rtl) => Alignment::Left,
        (TextAlign::Start, Direction::Rtl) | (TextAlign::End, Direction::Ltr) => Alignment::Right,
        _ => Alignment::Start,
    }
}

fn last_line_alignment(style: &ComputedValues, primary: Alignment) -> Option<Alignment> {
    if style.text_align == TextAlign::JustifyAll && style.text_justify != TextJustify::None {
        return Some(Alignment::Justify);
    }
    match style.text_align_last {
        TextAlignLast::Auto => None,
        TextAlignLast::Start => Some(directional_alignment(TextAlign::Start, style.direction)),
        TextAlignLast::End => Some(directional_alignment(TextAlign::End, style.direction)),
        TextAlignLast::Left => Some(Alignment::Left),
        TextAlignLast::Right => Some(Alignment::Right),
        TextAlignLast::Center => Some(Alignment::Center),
        TextAlignLast::Justify if style.text_justify == TextJustify::None => Some(primary),
        TextAlignLast::Justify => Some(Alignment::Justify),
    }
}

fn word_break(style: &ComputedValues) -> ParleyWordBreak {
    if style.line_break == CssLineBreak::Anywhere {
        return ParleyWordBreak::BreakAll;
    }
    match style.word_break {
        CssWordBreak::Normal | CssWordBreak::BreakWord => ParleyWordBreak::Normal,
        CssWordBreak::KeepAll => ParleyWordBreak::KeepAll,
        CssWordBreak::BreakAll => ParleyWordBreak::BreakAll,
    }
}

fn overflow_wrap(
    style: &ComputedValues,
    intrinsic_kind: Option<IntrinsicSizeKind>,
) -> ParleyOverflowWrap {
    // The legacy `word-break: break-word` value has the min-content
    // behavior of `overflow-wrap: anywhere`, unlike
    // `overflow-wrap: break-word` itself.
    if style.word_break == CssWordBreak::BreakWord {
        return ParleyOverflowWrap::Anywhere;
    }
    if intrinsic_kind == Some(IntrinsicSizeKind::MinContent)
        && style.overflow_wrap == CssOverflowWrap::BreakWord
    {
        return ParleyOverflowWrap::Normal;
    }
    match (style.word_break, style.overflow_wrap) {
        (CssWordBreak::BreakWord, _) => unreachable!("handled above"),
        (_, CssOverflowWrap::Normal) => ParleyOverflowWrap::Normal,
        (_, CssOverflowWrap::BreakWord) => ParleyOverflowWrap::BreakWord,
        (_, CssOverflowWrap::Anywhere) => ParleyOverflowWrap::Anywhere,
    }
}

fn text_wrap_mode(style: &ComputedValues) -> ParleyTextWrapMode {
    match style.text_wrap_mode {
        TextWrapMode::Wrap => ParleyTextWrapMode::Wrap,
        TextWrapMode::Nowrap => ParleyTextWrapMode::NoWrap,
    }
}

/// The used `line-height` of a box in `style` whose font has `metrics`.
fn used_line_height(style: &ComputedValues, metrics: FontBoxMetrics) -> f32 {
    match style.line_height {
        CssLineHeight::Normal => {
            normal_line_height(metrics.ascent, metrics.descent, metrics.line_gap)
        },
        _ => super::layout::line_height_px(&style.line_height, super::paint::used_font_size(style)),
    }
}

/// `line-height: normal`: ascent, descent and line gap each rounded, as
/// Chromium's font metrics do, so a normal line box is a whole number of
/// pixels (18 for Arial at 16px, not 18.4).
fn normal_line_height(ascent: f32, descent: f32, line_gap: f32) -> f32 {
    ascent.round() + descent.round() + line_gap.round()
}

/// An inline box's space above and below its baseline (CSS 2.1 10.8.1): its
/// font's ascent and descent, rounded first, plus half the leading on each
/// side, the larger half below. Chromium's rule, which Parley's quantized
/// metrics follow for whole-pixel leadings.
fn leading_extent(ascent: f32, descent: f32, line_height: f32) -> (f32, f32) {
    let (ascent, descent) = (ascent.round(), descent.round());
    let leading = line_height - (ascent + descent);
    let above = (leading * 0.5).floor();
    (ascent + above, descent + leading - above)
}

/// How far `vertical-align` raises a box whose extent about its baseline is
/// `(above, below)`, against its parent's font; `sub` and `super` take
/// Chromium's offsets from the parent's size. None for `top` and `bottom`,
/// which align to the line box once the rest are placed.
fn vertical_align_raise(
    value: VerticalAlign,
    font_size: f32,
    line_height: f32,
    (above, below): (f32, f32),
    parent: FontBoxMetrics,
) -> Option<f32> {
    Some(match value {
        VerticalAlign::Baseline => 0.0,
        VerticalAlign::Sub => -(parent.font_size / 5.0 + 1.0),
        VerticalAlign::Super => parent.font_size / 3.0 + 1.0,
        VerticalAlign::Length(value) => {
            super::layout::signed_length_percentage_px(value, font_size, line_height)
        },
        VerticalAlign::TextTop => parent.ascent.round() - above,
        VerticalAlign::TextBottom => below - parent.descent.round(),
        VerticalAlign::Middle => (parent.x_height + below - above) * 0.5,
        VerticalAlign::MiddleWithBaseline => (below - above) * 0.5,
        VerticalAlign::Top | VerticalAlign::Bottom => return None,
    })
}

/// Whether the line height calculation quirk applies to lines whose root is
/// `root`: in quirks and limited-quirks mode, except in a list item, which
/// forces strict line height (whatwg/quirks#38, as Chromium does).
fn line_height_quirk(mode: QuirksMode, root: &ComputedValues) -> bool {
    mode != QuirksMode::NoQuirks && root.display != Display::ListItem
}

/// Where Parley started a line: its baseline less the quantized ascent and
/// upper half-leading it added.
fn parley_line_top(metrics: &parley::LineMetrics) -> f32 {
    let (ascent, descent) = (metrics.ascent.round(), metrics.descent.round());
    let leading = metrics.line_height - (ascent + descent);
    metrics.baseline - ascent - (leading * 0.5).floor()
}

/// One inline box: its `vertical-align`, its extent (above, below) about its
/// own baseline, and how far that baseline sits above its parent box's; None
/// for `top` and `bottom`.
#[derive(Clone, Copy)]
struct Placement {
    value: VerticalAlign,
    extent: (f32, f32),
    raise: Option<f32>,
}

/// Where a baseline sits on the line: in the root's aligned subtree (`group`
/// None) or in the one a `top` or `bottom` box heads, `raise` above that
/// subtree's baseline (CSS 2.1 10.8.1's aligned subtrees).
#[derive(Clone, Copy)]
struct Anchor<Id> {
    group: Option<(Id, VerticalAlign)>,
    raise: f32,
}

impl<Id> Anchor<Id> {
    fn root() -> Self {
        Self {
            group: None,
            raise: 0.0,
        }
    }

    /// A baseline `raise` above this one.
    fn raised(self, raise: f32) -> Self {
        Self {
            group: self.group,
            raise: self.raise + raise,
        }
    }
}

/// An inline box in `style` with font `metrics`: its extent about its baseline
/// and its `vertical-align` raise over its parent's (CSS 2.1 10.8.1).
fn inline_box_placement(
    style: &ComputedValues,
    metrics: FontBoxMetrics,
    parent: FontBoxMetrics,
) -> Placement {
    let line_height = used_line_height(style, metrics);
    let extent = leading_extent(metrics.ascent, metrics.descent, line_height);
    Placement {
        value: style.vertical_align,
        extent,
        raise: vertical_align_raise(
            style.vertical_align,
            super::paint::used_font_size(style),
            line_height,
            extent,
            parent,
        ),
    }
}

/// `id`'s anchor: its own aligned subtree when it is `top` or `bottom`, else
/// its parent's anchor raised by its own raise. Memoized in `anchors`.
fn anchor_of<Id>(
    id: Id,
    parents: &HashMap<Id, Id>,
    placements: &HashMap<Id, Placement>,
    anchors: &mut HashMap<Id, Anchor<Id>>,
) -> Anchor<Id>
where
    Id: Copy + Eq + Hash,
{
    if let Some(anchor) = anchors.get(&id) {
        return *anchor;
    }
    let placement = placements[&id];
    let anchor = match placement.raise {
        Some(raise) => parents
            .get(&id)
            .map_or(Anchor::root(), |parent| {
                anchor_of(*parent, parents, placements, anchors)
            })
            .raised(raise),
        None => Anchor {
            group: Some((id, placement.value)),
            raise: 0.0,
        },
    };
    anchors.insert(id, anchor);
    anchor
}

/// The space a line box needs about its baseline (CSS 2.1 10.8): the root's
/// aligned subtree, and each subtree a `top` or `bottom` box heads, held back
/// until the rest is placed and then growing the line if taller.
struct LineExtents<Id> {
    above: f32,
    below: f32,
    held: Vec<(Id, VerticalAlign, f32, f32)>,
}

impl<Id> LineExtents<Id>
where
    Id: Copy + Eq + Hash,
{
    fn new() -> Self {
        Self {
            above: f32::NEG_INFINITY,
            below: f32::NEG_INFINITY,
            held: Vec::new(),
        }
    }

    /// Whether nothing on the line has counted yet.
    fn is_empty(&self) -> bool {
        !self.above.is_finite() && self.held.is_empty()
    }

    /// Include a box whose baseline sits at `anchor`, with its extent about it.
    fn include(&mut self, anchor: Anchor<Id>, (above, below): (f32, f32)) {
        let (above, below) = (above + anchor.raise, below - anchor.raise);
        match anchor.group {
            None => {
                self.above = self.above.max(above);
                self.below = self.below.max(below);
            },
            Some((id, value)) => match self.held.iter_mut().find(|held| held.0 == id) {
                Some(held) => {
                    held.2 = held.2.max(above);
                    held.3 = held.3.max(below);
                },
                None => self.held.push((id, value, above, below)),
            },
        }
    }

    /// Include each of `boxes` not yet seen on this line by its own extent;
    /// in quirks mode, only those in `edged`.
    fn include_boxes(
        &mut self,
        seen: &mut Vec<Id>,
        boxes: &[Id],
        (placements, anchors): (&HashMap<Id, Placement>, &HashMap<Id, Anchor<Id>>),
        edged: Option<&HashSet<Id>>,
    ) {
        for id in boxes {
            if seen.contains(id) || edged.is_some_and(|edged| !edged.contains(id)) {
                continue;
            }
            seen.push(*id);
            if let (Some(placement), Some(anchor)) = (placements.get(id), anchors.get(id)) {
                self.include(*anchor, placement.extent);
            }
        }
    }

    /// The space above the root's baseline, and the line box's height.
    fn resolve(&self) -> (f32, f32) {
        // Only a quirks-mode line can hold nothing that counts.
        let (mut above, mut below) = if self.above.is_finite() {
            (self.above, self.below)
        } else {
            (0.0, 0.0)
        };
        for &(_, value, held_above, held_below) in &self.held {
            let height = held_above + held_below;
            if height > above + below {
                if matches!(value, VerticalAlign::Top) {
                    below = height - above;
                } else {
                    above = height - below;
                }
            }
        }
        (above, above + below)
    }

    /// The baseline at `anchor`, on a line box with the given root baseline,
    /// top and height.
    fn baseline_at(&self, anchor: Anchor<Id>, baseline: f32, line_top: f32, height: f32) -> f32 {
        let group = anchor.group.and_then(|(id, value)| {
            let &(_, _, above, below) = self.held.iter().find(|held| held.0 == id)?;
            Some(if matches!(value, VerticalAlign::Bottom) {
                line_top + height - below
            } else {
                line_top + above
            })
        });
        group.unwrap_or(baseline) - anchor.raise
    }
}

fn spacing_px(spacing: Spacing, font_size: f32) -> Option<f32> {
    match spacing {
        Spacing::Normal => None,
        Spacing::Length(length) => Some(length.to_px(font_size, 16.0, font_size)),
    }
}

fn brush(style: &ComputedValues, source_index: usize) -> Brush {
    let color = resolve_color(&style.color);
    Brush {
        color: [color.r, color.g, color.b, color.a],
        source_index,
    }
}

fn font_family(style: &ComputedValues) -> StyleProperty<'_, Brush> {
    let family = match &style.font_family {
        CssFontFamily::UserAgentDefault => FontFamily::from(GenericFamily::SansSerif),
        CssFontFamily::SystemUi => FontFamily::from(GenericFamily::SystemUi),
        CssFontFamily::Named(name) | CssFontFamily::List(name) => {
            FontFamily::Source(Cow::Borrowed(name))
        },
    };
    StyleProperty::FontFamily(family)
}

fn font_weight(style: &ComputedValues) -> f32 {
    match style.font_weight {
        CssFontWeight::Normal => 400.0,
        CssFontWeight::Bold | CssFontWeight::Bolder => 700.0,
        CssFontWeight::Lighter => 300.0,
        CssFontWeight::Number(value) => f32::from(value),
    }
}

fn font_style(style: &ComputedValues) -> FontStyle {
    match style.font_style {
        CssFontStyle::Normal => FontStyle::Normal,
        CssFontStyle::Italic | CssFontStyle::Oblique => FontStyle::Italic,
    }
}

fn line_height(style: &ComputedValues) -> StyleProperty<'static, Brush> {
    let value = match style.line_height {
        CssLineHeight::Normal => parley::LineHeight::MetricsRelative(1.0),
        CssLineHeight::Number(value) => parley::LineHeight::FontSizeRelative(value),
        CssLineHeight::Value(_) => parley::LineHeight::Absolute(super::layout::line_height_px(
            &style.line_height,
            super::paint::used_font_size(style),
        )),
    };
    StyleProperty::LineHeight(value)
}

fn content_key(bytes: &[u8], index: u32) -> FontInstanceKey {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in bytes.iter().copied().chain(index.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    FontInstanceKey::new(IdNamespace((hash >> 32) as u32), hash as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_woff2_is_rejected_before_font_registration() {
        assert!(normalized_font_bytes(b"wOF2not-a-font".to_vec()).is_none());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm_rejects_woff2_without_native_sanitizer() {
        assert!(normalized_font_bytes(b"wOF2opaque".to_vec()).is_none());
    }

    #[test]
    fn sfnt_font_bytes_keep_their_existing_identity() {
        let bytes = b"\0\x01\0\0existing-sfnt".to_vec();
        assert_eq!(normalized_font_bytes(bytes.clone()), Some(bytes));
    }

    #[test]
    fn adjacent_inline_text_nodes_do_not_create_a_collapsed_space() {
        let style = ComputedValues::default();
        let mut text = String::new();

        append_inline_text(&mut text, "12", &style);
        append_inline_text(&mut text, "34", &style);

        assert_eq!(text, "1234");
    }

    #[test]
    fn forced_line_break_is_not_collapsed_to_a_space() {
        let style = ComputedValues::default();
        let mut text = String::new();

        append_inline_text(&mut text, "12", &style);
        assert_eq!(append_forced_line_break(&mut text), 2);
        append_inline_text(&mut text, "34", &style);

        assert_eq!(text, "12\n34");
    }

    #[test]
    fn text_directive_matches_contextual_range_across_retained_sources() {
        let mut frame = TextFrame::<u8>::default();
        frame.text_order = vec![1, 2];
        frame.text_values.insert(1, "prefix start ".to_owned());
        frame.text_values.insert(2, "end suffix".to_owned());
        frame.text_groups.insert(1, 0);
        frame.text_groups.insert(2, 0);

        assert_eq!(
            frame.find_text_directive_range(&TextDirective {
                prefix: Some("prefix".to_owned()),
                start: "start".to_owned(),
                end: Some("end".to_owned()),
                suffix: Some("suffix".to_owned()),
            }),
            Some(TextRange {
                anchor_node: 1,
                anchor_offset: 7,
                focus_node: 2,
                focus_offset: 3,
            })
        );
    }

    #[test]
    fn text_directive_skips_occurrences_without_the_required_context() {
        let mut frame = TextFrame::<u8>::default();
        frame.text_order = vec![1];
        frame
            .text_values
            .insert(1, "start wrong prefix start right suffix".to_owned());
        frame.text_groups.insert(1, 0);

        assert_eq!(
            frame.find_text_directive_range(&TextDirective {
                prefix: Some("prefix".to_owned()),
                start: "start".to_owned(),
                end: None,
                suffix: Some("right".to_owned()),
            }),
            Some(TextRange {
                anchor_node: 1,
                anchor_offset: 19,
                focus_node: 1,
                focus_offset: 24,
            })
        );
    }
}
