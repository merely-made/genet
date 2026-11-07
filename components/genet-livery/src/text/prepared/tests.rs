use super::*;

// Intrinsic-dependent properties must also be checked by can_share_eager_key.
#[cfg(test)]
fn legacy_properties(
    style: &ComputedValues,
    source_index: usize,
    features: Vec<FontFeature>,
    intrinsic: Option<IntrinsicSizeKind>,
    tab: f32,
) -> Vec<Property> {
    let size = super::super::super::paint::used_font_size(style);
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

fn fixture() -> ShapeInput {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    ShapeInput {
        text: "hello".into(),
        defaults: properties(&style, 0, &[], None, 8.0),
        spans: vec![Span {
            range: 0..5,
            properties: properties(&style, 0, &[], None, 8.0),
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

mod normalization;

#[test]
fn borrowed_properties_reject_each_changed_value() {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    style.letter_spacing = "0px".parse().unwrap();
    style.word_spacing = "0px".parse().unwrap();
    let features = vec![
        FontFeature::new(parley::setting::Tag::from_bytes(*b"kern"), 1),
        FontFeature::new(parley::setting::Tag::from_bytes(*b"liga"), 0),
    ];
    let reference = legacy_properties(&style, 0, features.clone(), None, 8.0);
    for index in 0..reference.len() {
        let mut changed = reference.clone();
        match &mut changed[index] {
            Property::Size(v)
            | Property::Weight(v)
            | Property::TabSize(v)
            | Property::LetterSpacing(v)
            | Property::WordSpacing(v) => *v ^= 1,
            Property::Family(v) => *v = Family::Source("changed".into()),
            Property::Italic(v) => *v = !*v,
            Property::Brush(color, _) => color[0] ^= 1,
            Property::LineHeight(_, v) => *v ^= 1,
            Property::Features(v) => v[0].1 ^= 1,
            Property::WordBreak(v) | Property::OverflowWrap(v) | Property::TextWrap(v) => *v ^= 1,
        }
        assert!(
            !properties_match(&changed, &style, 0, &features, None, 8.0),
            "property {index}"
        );
    }
    let mut changed = reference.clone();
    if let Property::Brush(_, source) = &mut changed[4] {
        *source += 1;
    }
    assert!(!properties_match(&changed, &style, 0, &features, None, 8.0));
    let mut changed = reference.clone();
    if let Property::Features(f) = &mut changed[6] {
        f[0].0 = *b"xxxx";
    }
    assert!(!properties_match(&changed, &style, 0, &features, None, 8.0));
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
    let used = properties(&style, 0, &[], None, 8.0);
    let min = properties(&style, 0, &[], Some(IntrinsicSizeKind::MinContent), 8.0);
    let max = properties(&style, 0, &[], Some(IntrinsicSizeKind::MaxContent), 8.0);
    assert_ne!(used, min);
    assert_eq!(used, max);
    style.overflow_wrap = CssOverflowWrap::Normal;
    assert_eq!(min, properties(&style, 0, &[], None, 8.0));
}

#[test]
fn intrinsic_policy_reuses_only_equal_builder_inputs() {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    let seed = std::collections::hash_map::RandomState::new();
    let make = |style: &ComputedValues, kind| {
        let mut input = fixture();
        input.defaults = properties(style, 0, &[], kind, 8.0);
        input.spans[0].properties = input.defaults.clone();
        super::super::cache::ShapeKey::new(0, input, &seed)
    };
    let mut worker = super::super::shaping::ShapeWorker::new();
    let mut cache = super::super::cache::ShapeCache::default();
    let used = make(&style, None);
    let value = std::sync::Arc::new(worker.shape(used.input()));
    cache.insert(used.clone(), value, 8, 1000, usize::MAX);
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
    let value = std::sync::Arc::new(worker.shape(changed.input()));
    cache.insert(changed, value, 8, 1000, usize::MAX);
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
