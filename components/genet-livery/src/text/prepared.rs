// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Family {
    SansSerif,
    SystemUi,
    Source(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Property {
    Size(u32),
    Family(Family),
    Weight(u32),
    Italic(bool),
    Brush([u32; 4], usize),
    LineHeight(u8, u32),
    Features(Vec<([u8; 4], u16)>),
    WordBreak(u8),
    OverflowWrap(u8),
    TextWrap(u8),
    TabSize(u32),
    LetterSpacing(u32),
    WordSpacing(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Span {
    pub range: Range<usize>,
    pub properties: Vec<Property>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum SlotKind {
    InFlow,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Slot {
    pub kind: SlotKind,
    pub id: u64,
    pub index: usize,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShapeInput {
    pub(super) text: String,
    pub(super) defaults: Vec<Property>,
    pub(super) spans: Vec<Span>,
    pub(super) slots: Vec<Slot>,
    pub(super) base_level: u8,
    pub(super) scale: u32,
    pub(super) quantize: bool,
}

pub(super) fn properties(
    style: &ComputedValues,
    source_index: usize,
    features: Vec<FontFeature>,
    intrinsic: Option<IntrinsicSizeKind>,
    tab: f32,
) -> Vec<Property> {
    let size = super::super::paint::used_font_size(style);
    let family = match &style.font_family {
        CssFontFamily::UserAgentDefault => Family::SansSerif,
        CssFontFamily::SystemUi => Family::SystemUi,
        CssFontFamily::Named(name) | CssFontFamily::List(name) => Family::Source(name.to_string()),
    };
    let b = brush(style, source_index);
    let height = match line_height(style) {
        StyleProperty::LineHeight(v) => v,
        _ => unreachable!(),
    };
    let (kind, height) = match height {
        parley::LineHeight::MetricsRelative(v) => (0, v),
        parley::LineHeight::FontSizeRelative(v) => (1, v),
        parley::LineHeight::Absolute(v) => (2, v),
    };
    let mut result = vec![
        Property::Size(size.to_bits()),
        Property::Family(family),
        Property::Weight(font_weight(style).to_bits()),
        Property::Italic(font_style(style) == FontStyle::Italic),
        Property::Brush(b.color.map(f32::to_bits), b.source_index),
        Property::LineHeight(kind, height.to_bits()),
        Property::Features(
            features
                .into_iter()
                .map(|f| (f.tag.to_bytes(), f.value))
                .collect(),
        ),
        Property::WordBreak(match word_break(style) {
            ParleyWordBreak::Normal => 0,
            ParleyWordBreak::BreakAll => 1,
            ParleyWordBreak::KeepAll => 2,
        }),
        Property::OverflowWrap(match overflow_wrap(style, intrinsic) {
            ParleyOverflowWrap::Normal => 0,
            ParleyOverflowWrap::Anywhere => 1,
            ParleyOverflowWrap::BreakWord => 2,
        }),
        Property::TextWrap(match text_wrap_mode(style) {
            ParleyTextWrapMode::Wrap => 0,
            ParleyTextWrapMode::NoWrap => 1,
        }),
        Property::TabSize(tab.to_bits()),
    ];
    if let Some(v) = spacing_px(style.letter_spacing, size) {
        result.push(Property::LetterSpacing(v.to_bits()));
    }
    if let Some(v) = spacing_px(style.word_spacing, size) {
        result.push(Property::WordSpacing(v.to_bits()));
    }
    result
}
impl Property {
    fn parley(&self) -> StyleProperty<'_, Brush> {
        match self {
            Self::Size(v) => StyleProperty::FontSize(f32::from_bits(*v)),
            Self::Family(v) => StyleProperty::FontFamily(match v {
                Family::SansSerif => FontFamily::from(GenericFamily::SansSerif),
                Family::SystemUi => FontFamily::from(GenericFamily::SystemUi),
                Family::Source(s) => FontFamily::Source(Cow::Borrowed(s)),
            }),
            Self::Weight(v) => StyleProperty::FontWeight(FontWeight::new(f32::from_bits(*v))),
            Self::Italic(v) => StyleProperty::FontStyle(if *v {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            }),
            Self::Brush(c, source_index) => StyleProperty::Brush(Brush {
                color: c.map(f32::from_bits),
                source_index: *source_index,
            }),
            Self::LineHeight(k, v) => StyleProperty::LineHeight(match k {
                0 => parley::LineHeight::MetricsRelative(f32::from_bits(*v)),
                1 => parley::LineHeight::FontSizeRelative(f32::from_bits(*v)),
                _ => parley::LineHeight::Absolute(f32::from_bits(*v)),
            }),
            Self::Features(v) => StyleProperty::FontFeatures(FontFeatures::List(Cow::Owned(
                v.iter()
                    .map(|(tag, value)| {
                        FontFeature::new(parley::setting::Tag::from_bytes(*tag), *value)
                    })
                    .collect(),
            ))),
            Self::WordBreak(v) => StyleProperty::WordBreak(match v {
                0 => ParleyWordBreak::Normal,
                1 => ParleyWordBreak::BreakAll,
                _ => ParleyWordBreak::KeepAll,
            }),
            Self::OverflowWrap(v) => StyleProperty::OverflowWrap(match v {
                0 => ParleyOverflowWrap::Normal,
                1 => ParleyOverflowWrap::Anywhere,
                _ => ParleyOverflowWrap::BreakWord,
            }),
            Self::TextWrap(v) => StyleProperty::TextWrapMode(if *v == 0 {
                ParleyTextWrapMode::Wrap
            } else {
                ParleyTextWrapMode::NoWrap
            }),
            Self::TabSize(v) => StyleProperty::TabSize(f32::from_bits(*v)),
            Self::LetterSpacing(v) => StyleProperty::LetterSpacing(f32::from_bits(*v)),
            Self::WordSpacing(v) => StyleProperty::WordSpacing(f32::from_bits(*v)),
        }
    }
}
impl ShapeInput {
    pub(super) fn apply(&self, builder: &mut parley::RangedBuilder<'_, Brush>) {
        builder.set_base_level(Some(self.base_level));
        for property in &self.defaults {
            builder.push_default(property.parley());
        }
        for span in &self.spans {
            for property in &span.properties {
                builder.push(property.parley(), span.range.clone());
            }
        }
        for slot in &self.slots {
            builder.push_inline_box(InlineBox {
                id: slot.id,
                kind: match slot.kind {
                    SlotKind::InFlow => InlineBoxKind::InFlow,
                },
                index: slot.index,
                width: f32::from_bits(slot.width),
                height: f32::from_bits(slot.height),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ShapeInput {
        let mut style = ComputedValues::default();
        style.color = "black".parse().unwrap();
        ShapeInput {
            text: "hello".into(),
            defaults: properties(&style, 0, Vec::new(), None, 8.0),
            spans: vec![Span {
                range: 0..5,
                properties: properties(&style, 0, Vec::new(), None, 8.0),
            }],
            slots: vec![Slot {
                kind: SlotKind::InFlow,
                id: 0,
                index: 1,
                width: 4.0f32.to_bits(),
                height: 5.0f32.to_bits(),
            }],
            base_level: 0,
            scale: 1.0f32.to_bits(),
            quantize: true,
        }
    }

    #[test]
    fn split_shape_input_exact_identity() {
        let input = fixture();
        assert_eq!(input, input.clone());
        use std::hash::{Hash, Hasher};
        let hash = |value: &ShapeInput| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(&input), hash(&input.clone()));
        let mut changed = input.clone();
        changed.text.push('!');
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.spans[0].range.end -= 1;
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.spans[0].properties.iter_mut().find_map(|p| {
            if let Property::Brush(_, source) = p {
                *source = 1;
                Some(())
            } else {
                None
            }
        });
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.defaults.iter_mut().find_map(|p| {
            if let Property::Features(features) = p {
                features.push((*b"liga", 0));
                Some(())
            } else {
                None
            }
        });
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.defaults[0] = Property::Size(17.0f32.to_bits());
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.defaults[1] = Property::Family(Family::Source("other".into()));
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.base_level = 1;
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.slots[0].width = 4.5f32.to_bits();
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.slots[0].height = (-0.0f32).to_bits();
        assert_ne!(input, changed);
        let mut changed = input.clone();
        changed.slots[0].width = f32::NAN.to_bits();
        assert_eq!(changed, changed.clone());
        assert_eq!(hash(&changed), hash(&changed.clone()));
        let mut changed = input.clone();
        changed.scale = (-0.0f32).to_bits();
        let mut zero = changed.clone();
        zero.scale = 0.0f32.to_bits();
        assert_ne!(zero, changed);
    }

    #[test]
    fn split_shape_min_content_overflow_wrap_is_effective_property() {
        let mut style = ComputedValues::default();
        style.color = "black".parse().unwrap();
        style.overflow_wrap = CssOverflowWrap::BreakWord;
        let used = properties(&style, 0, Vec::new(), None, 8.0);
        let min = properties(
            &style,
            0,
            Vec::new(),
            Some(IntrinsicSizeKind::MinContent),
            8.0,
        );
        let max = properties(
            &style,
            0,
            Vec::new(),
            Some(IntrinsicSizeKind::MaxContent),
            8.0,
        );
        assert_ne!(used, min);
        assert_eq!(used, max);
        style.overflow_wrap = CssOverflowWrap::Normal;
        assert_eq!(min, properties(&style, 0, Vec::new(), None, 8.0));
    }

    #[test]
    fn intrinsic_policy_reuses_only_equal_builder_inputs() {
        let mut style = ComputedValues::default();
        style.color = "black".parse().unwrap();
        let make = |style: &ComputedValues, kind| {
            let mut input = fixture();
            input.defaults = properties(style, 0, Vec::new(), kind, 8.0);
            input.spans[0].properties = input.defaults.clone();
            super::super::cache::ShapeKey {
                generation: 0,
                input,
            }
        };
        let mut worker = super::super::shaping::ShapeWorker::new();
        let mut cache = super::super::cache::ShapeCache::default();
        let used = make(&style, None);
        let value = std::sync::Arc::new(worker.shape(&used.input));
        cache.insert(used.clone(), value, 8, 1000);
        assert!(
            cache
                .get(&make(&style, Some(IntrinsicSizeKind::MinContent)))
                .is_some()
        );
        assert!(
            cache
                .get(&make(&style, Some(IntrinsicSizeKind::MaxContent)))
                .is_some()
        );
        style.overflow_wrap = CssOverflowWrap::BreakWord;
        let changed = make(&style, None);
        assert!(cache.get(&changed).is_none());
        assert!(
            cache
                .get(&make(&style, Some(IntrinsicSizeKind::MinContent)))
                .is_some()
        );
        let value = std::sync::Arc::new(worker.shape(&changed.input));
        cache.insert(changed, value, 8, 1000);
        assert!(
            cache
                .get(&make(&style, Some(IntrinsicSizeKind::MaxContent)))
                .is_some()
        );
    }

    #[test]
    fn split_shape_payloads_are_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ShapeInput>();
        assert_send::<super::super::shaping::UnbrokenText>();
    }
}

pub(crate) struct PreparedInlineGroup {
    pub(super) text: String,
    pub(super) spans: Vec<SourceSpan<BoxId>>,
    pub(super) inline_styles: HashMap<BoxId, ComputedValues>,
    events: Vec<GeometryEvent>,
    inline_owners: Vec<InlineOwner>,
}

pub(super) struct InlineOwner {
    pub source: BoxId,
    pub style: ComputedValues,
    pub owners: Vec<BoxId>,
    pub positioned: bool,
}
pub(super) enum GeometryEvent {
    Begin {
        slot: usize,
        index: usize,
    },
    End {
        slot: usize,
        index: usize,
        text_empty: bool,
    },
    Atomic {
        source: BoxId,
        style: ComputedValues,
        owners: Vec<BoxId>,
        index: usize,
    },
}

impl PreparedInlineGroup {
    pub(crate) fn collect<D: LayoutDom>(
        dom: &D,
        styles: &StylePlane<D::NodeId>,
        boxes: &CssBoxTree<D::NodeId>,
        roots: &[BoxId],
        parent_style: &ComputedValues,
    ) -> Self
    where
        D::NodeId: Copy + Eq + Hash,
    {
        let mut result = Self {
            text: String::new(),
            spans: Vec::new(),
            inline_styles: HashMap::new(),
            events: Vec::new(),
            inline_owners: Vec::new(),
        };
        let mut owners = Vec::new();
        let mut collector = TapeCollector {
            dom,
            styles,
            boxes,
            owners: &mut owners,
            text: &mut result.text,
            spans: &mut result.spans,
            inline_styles: &mut result.inline_styles,
            events: &mut result.events,
            inline_owners: &mut result.inline_owners,
        };
        for root in roots {
            collector.collect(*root, parent_style);
        }
        result
    }
    /// Only admit a speculative key when every inserted atom exists and all
    /// geometry contributing to it is independent of the eventual query width.
    pub(crate) fn ready<Id, F: FragmentLookup<Id>>(&self, fragments: &F) -> bool {
        fn independent(value: livery::values::LengthPercentage) -> bool {
            use livery::values::LengthPercentage as L;
            matches!(value, L::Zero | L::Length(_))
        }
        fn style_ready(style: &ComputedValues) -> bool {
            use livery::values::Margin;
            [
                style.padding_left.0,
                style.padding_right.0,
                style.padding_top.0,
                style.padding_bottom.0,
            ]
            .into_iter()
            .all(independent)
                && [
                    style.margin_left,
                    style.margin_right,
                    style.margin_top,
                    style.margin_bottom,
                ]
                .into_iter()
                .all(|m| match m {
                    Margin::Auto => true,
                    Margin::Value(v) => independent(v),
                })
        }
        self.events.iter().all(|event| match event {
            GeometryEvent::Begin { slot, .. } | GeometryEvent::End { slot, .. } => {
                style_ready(&self.inline_owners[*slot].style)
            },
            GeometryEvent::Atomic { source, style, .. } => {
                fragments.atomic_box_rect(*source).is_some() && style_ready(style)
            },
        })
    }
    pub(crate) fn materialize<Id, F: FragmentLookup<Id>>(
        &self,
        fragments: &F,
        width: f32,
        intrinsic_kind: Option<IntrinsicSizeKind>,
    ) -> Vec<InlineAtom<BoxId>> {
        let mut m = GeometryMaterializer {
            fragments,
            owners: Vec::new(),
            inline_boxes: Vec::new(),
            percentage_basis: width,
            intrinsic_kind,
            index: 0,
            marker: std::marker::PhantomData,
        };
        let mut stack = Vec::new();
        for event in &self.events {
            match event {
                GeometryEvent::Begin { slot, index } => {
                    let owner = &self.inline_owners[*slot];
                    m.index = *index;
                    m.push_edge(owner.source, &owner.style, &owner.owners, true);
                    m.push_positioned_start_marker(
                        owner.source,
                        &owner.style,
                        &owner.owners,
                        owner.positioned,
                    );
                    stack.push(m.inline_boxes.len());
                },
                GeometryEvent::End {
                    slot,
                    index,
                    text_empty,
                } => {
                    let owner = &self.inline_owners[*slot];
                    m.index = *index;
                    let has_content =
                        m.inline_boxes.len() > stack.pop().expect("balanced inline events");
                    m.push_edge(owner.source, &owner.style, &owner.owners, false);
                    if *text_empty && !has_content {
                        m.push_empty_line_box(
                            owner.source,
                            &owner.style,
                            &owner.owners,
                            owner.positioned,
                        );
                    }
                },
                GeometryEvent::Atomic {
                    source,
                    style,
                    owners,
                    index,
                } => {
                    m.index = *index;
                    m.owners.clone_from(owners);
                    m.push_atomic_box(*source, style);
                },
            }
        }
        debug_assert!(stack.is_empty());
        m.inline_boxes
    }
}
