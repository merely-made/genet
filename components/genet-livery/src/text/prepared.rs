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

// The same normalized values feed owned keys and borrowed validation.
enum PropertyRef<'a> {
    Value(Property),
    Family(&'a CssFontFamily),
    Features(&'a [FontFeature]),
}
impl PropertyRef<'_> {
    fn matches(&self, value: &Property) -> bool {
        match (self, value) {
            (Self::Value(a), b) => a == b,
            (Self::Family(a), Property::Family(b)) => match (a, b) {
                (CssFontFamily::UserAgentDefault, Family::SansSerif)
                | (CssFontFamily::SystemUi, Family::SystemUi) => true,
                (CssFontFamily::Named(a) | CssFontFamily::List(a), Family::Source(b)) => {
                    a.as_ref() == b
                },
                _ => false,
            },
            (Self::Features(a), Property::Features(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|(a, b)| (a.tag.to_bytes(), a.value) == *b)
            },
            _ => false,
        }
    }
    fn owned(self) -> Property {
        match self {
            Self::Value(value) => value,
            Self::Family(family) => Property::Family(match family {
                CssFontFamily::UserAgentDefault => Family::SansSerif,
                CssFontFamily::SystemUi => Family::SystemUi,
                CssFontFamily::Named(name) | CssFontFamily::List(name) => {
                    Family::Source(name.to_string())
                },
            }),
            Self::Features(features) => Property::Features(
                features
                    .iter()
                    .map(|f| (f.tag.to_bytes(), f.value))
                    .collect(),
            ),
        }
    }
}

fn visit_properties<'a>(
    style: &'a ComputedValues,
    source_index: usize,
    features: &'a [FontFeature],
    intrinsic: Option<IntrinsicSizeKind>,
    tab: f32,
    mut visit: impl FnMut(PropertyRef<'a>),
) {
    let size = super::super::paint::used_font_size(style);
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
    visit(PropertyRef::Value(Property::Size(size.to_bits())));
    visit(PropertyRef::Family(&style.font_family));
    visit(PropertyRef::Value(Property::Weight(
        font_weight(style).to_bits(),
    )));
    visit(PropertyRef::Value(Property::Italic(
        font_style(style) == FontStyle::Italic,
    )));
    visit(PropertyRef::Value(Property::Brush(
        b.color.map(f32::to_bits),
        b.source_index,
    )));
    visit(PropertyRef::Value(Property::LineHeight(
        kind,
        height.to_bits(),
    )));
    visit(PropertyRef::Features(features));
    visit(PropertyRef::Value(Property::WordBreak(
        match word_break(style) {
            ParleyWordBreak::Normal => 0,
            ParleyWordBreak::BreakAll => 1,
            ParleyWordBreak::KeepAll => 2,
        },
    )));
    visit(PropertyRef::Value(Property::OverflowWrap(
        match overflow_wrap(style, intrinsic) {
            ParleyOverflowWrap::Normal => 0,
            ParleyOverflowWrap::Anywhere => 1,
            ParleyOverflowWrap::BreakWord => 2,
        },
    )));
    visit(PropertyRef::Value(Property::TextWrap(
        match text_wrap_mode(style) {
            ParleyTextWrapMode::Wrap => 0,
            ParleyTextWrapMode::NoWrap => 1,
        },
    )));
    visit(PropertyRef::Value(Property::TabSize(tab.to_bits())));
    if let Some(v) = spacing_px(style.letter_spacing, size) {
        visit(PropertyRef::Value(Property::LetterSpacing(v.to_bits())));
    }
    if let Some(v) = spacing_px(style.word_spacing, size) {
        visit(PropertyRef::Value(Property::WordSpacing(v.to_bits())));
    }
}

pub(super) fn properties(
    style: &ComputedValues,
    source_index: usize,
    features: &[FontFeature],
    intrinsic: Option<IntrinsicSizeKind>,
    tab: f32,
) -> Vec<Property> {
    let mut result = Vec::with_capacity(11);
    visit_properties(style, source_index, features, intrinsic, tab, |p| {
        result.push(p.owned())
    });
    result
}

fn properties_match(
    expected: &[Property],
    style: &ComputedValues,
    source_index: usize,
    features: &[FontFeature],
    intrinsic: Option<IntrinsicSizeKind>,
    tab: f32,
) -> bool {
    let mut iter = expected.iter();
    let mut same = true;
    visit_properties(style, source_index, features, intrinsic, tab, |p| {
        same &= iter.next().is_some_and(|value| p.matches(value));
    });
    same && iter.next().is_none()
}

pub(super) fn slots<Id>(atoms: &[InlineAtom<Id>]) -> impl Iterator<Item = Slot> + '_ {
    atoms
        .iter()
        .enumerate()
        .filter(|(_, atom)| !atom.marker)
        .map(|(index, atom)| Slot {
            kind: SlotKind::InFlow,
            id: u64::try_from(index).unwrap_or(u64::MAX),
            index: atom.index,
            width: atom.line_width.to_bits(),
            height: (if atom.edge { 0.0 } else { atom.line_box_height }).to_bits(),
        })
}

impl ShapeInput {
    pub(super) fn matches_current<Id>(
        &self,
        text: &str,
        spans: &[SourceSpan<Id>],
        atoms: &[InlineAtom<Id>],
        root: &ComputedValues,
        intrinsic: Option<IntrinsicSizeKind>,
        resolved: &ResolvedShapeStyles,
    ) -> bool {
        let default = spans.first().map_or(root, |span| &span.style);
        self.text == text
            && self.base_level
                == match root.direction {
                    Direction::Ltr => 0,
                    Direction::Rtl => 1,
                }
            && self.scale == 1.0f32.to_bits()
            && self.quantize
            && properties_match(
                &self.defaults,
                default,
                0,
                &resolved.default_features,
                intrinsic,
                resolved.default_tab_stop,
            )
            && self.spans.len() == spans.len()
            && self
                .spans
                .iter()
                .zip(spans)
                .enumerate()
                .all(|(i, (old, span))| {
                    old.range == span.range
                        && properties_match(
                            &old.properties,
                            &span.style,
                            i,
                            &resolved.span_features[i],
                            intrinsic,
                            resolved.span_tab_stops[i],
                        )
                })
            && self.slots.iter().cloned().eq(slots(atoms))
    }
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
mod tests;

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
    // Keep in sync with every intrinsic-dependent input in properties. Empty
    // events make materialized geometry identical for the three requests.
    pub(crate) fn can_share_eager_key(&self, root: &ComputedValues) -> bool {
        if !self.events.is_empty() {
            return false;
        }
        let equivalent = |style: &ComputedValues| {
            let min = overflow_wrap(style, Some(IntrinsicSizeKind::MinContent));
            min == overflow_wrap(style, Some(IntrinsicSizeKind::MaxContent))
                && min == overflow_wrap(style, None)
        };
        equivalent(self.spans.first().map_or(root, |span| &span.style))
            && self.spans.iter().all(|span| equivalent(&span.style))
    }

    #[cfg(test)]
    pub(crate) fn test_empty_events(&self) -> bool {
        self.events.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn test_individual_event_rejections(
        &mut self,
        root: &ComputedValues,
    ) -> (usize, usize, usize) {
        let events = std::mem::take(&mut self.events);
        let mut restored = Vec::new();
        let mut counts = (0, 0, 0);
        for event in events {
            match &event {
                GeometryEvent::Begin { .. } => counts.0 += 1,
                GeometryEvent::End { .. } => counts.1 += 1,
                GeometryEvent::Atomic { .. } => counts.2 += 1,
            }
            self.events.push(event);
            assert!(!self.can_share_eager_key(root));
            restored.push(self.events.pop().unwrap());
        }
        self.events = restored;
        counts
    }

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
    pub(crate) fn missing_atom<Id, F: FragmentLookup<Id>>(&self, fragments: &F) -> Option<BoxId> {
        self.events.iter().find_map(|event| match event {
            GeometryEvent::Atomic { source, .. }
                if fragments.atomic_box_rect(*source).is_none() =>
            {
                Some(*source)
            },
            _ => None,
        })
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
