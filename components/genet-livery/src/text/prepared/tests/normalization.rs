use super::*;

#[test]
fn borrowed_normalization_matches_prechange_reference() {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    let features = vec![
        FontFeature::new(parley::setting::Tag::from_bytes(*b"kern"), 1),
        FontFeature::new(parley::setting::Tag::from_bytes(*b"liga"), 0),
    ];
    for family in [
        CssFontFamily::UserAgentDefault,
        CssFontFamily::SystemUi,
        CssFontFamily::Named(" a b ".into()),
        CssFontFamily::List("a, b".into()),
    ] {
        style.font_family = family;
        for policy in [
            CssOverflowWrap::Normal,
            CssOverflowWrap::BreakWord,
            CssOverflowWrap::Anywhere,
        ] {
            style.overflow_wrap = policy;
            for intrinsic in [
                None,
                Some(IntrinsicSizeKind::MinContent),
                Some(IntrinsicSizeKind::MaxContent),
            ] {
                for tab in [0.0, -0.0, 8.0] {
                    let old = legacy_properties(&style, 3, features.clone(), intrinsic, tab);
                    assert_eq!(properties(&style, 3, &features, intrinsic, tab), old);
                    assert!(properties_match(&old, &style, 3, &features, intrinsic, tab));
                    let mut extra = old.clone();
                    extra.push(Property::WordSpacing(0));
                    assert!(!properties_match(
                        &extra, &style, 3, &features, intrinsic, tab
                    ));
                    let mut changed = old.clone();
                    changed[0] = Property::Size(0);
                    assert!(!properties_match(
                        &changed, &style, 3, &features, intrinsic, tab
                    ));
                }
            }
        }
    }
}

#[test]
fn prechange_reference_covers_heights_spacing_and_feature_order() {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    let features = vec![
        FontFeature::new(parley::setting::Tag::from_bytes(*b"kern"), 1),
        FontFeature::new(parley::setting::Tag::from_bytes(*b"liga"), 0),
    ];
    for height in [
        CssLineHeight::Normal,
        CssLineHeight::Number(1.25),
        CssLineHeight::Value("20px".parse().unwrap()),
    ] {
        style.line_height = height;
        for spacing in ["normal", "0px", "2px"] {
            style.letter_spacing = spacing.parse().unwrap();
            style.word_spacing = spacing.parse().unwrap();
            let reference = legacy_properties(&style, 0, features.clone(), None, -0.0);
            assert_eq!(properties(&style, 0, &features, None, -0.0), reference);
            assert!(properties_match(
                &reference, &style, 0, &features, None, -0.0
            ));
            let mut reversed = features.clone();
            reversed.reverse();
            assert!(!properties_match(
                &reference, &style, 0, &reversed, None, -0.0
            ));
            assert!(!properties_match(
                &reference, &style, 0, &features, None, 0.0
            ));
            for index in 0..reference.len() {
                let mut changed = reference.clone();
                changed[index] = Property::TabSize(u32::MAX);
                assert!(!properties_match(
                    &changed, &style, 0, &features, None, -0.0
                ));
            }
        }
    }
}

#[test]
fn borrowed_input_checks_complete_owned_values() {
    let mut style = ComputedValues::default();
    style.color = "black".parse().unwrap();
    let resolved = ResolvedShapeStyles {
        default_features: vec![],
        span_features: vec![],
        default_tab_stop: 8.0,
        span_tab_stops: vec![],
    };
    let input = ShapeInput {
        text: "hello".into(),
        defaults: legacy_properties(&style, 0, vec![], None, 8.0),
        spans: vec![],
        slots: vec![],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    let matches = |value: &ShapeInput| {
        value.matches_current::<u32>("hello", &[], &[], &style, None, &resolved)
    };
    assert!(matches(&input));
    let mut changed = input.clone();
    changed.text.push('!');
    assert!(!matches(&changed));
    let mut changed = input.clone();
    changed.base_level = 1;
    assert!(!matches(&changed));
    let mut changed = input.clone();
    changed.scale = (-0.0f32).to_bits();
    assert!(!matches(&changed));
    let mut changed = input.clone();
    changed.quantize = false;
    assert!(!matches(&changed));
    let mut changed = input.clone();
    changed.spans.push(Span {
        range: 0..0,
        properties: vec![],
    });
    assert!(!matches(&changed));
    let mut changed = input.clone();
    changed.slots.push(Slot {
        kind: SlotKind::InFlow,
        id: 0,
        index: 0,
        width: 0,
        height: 0,
    });
    assert!(!matches(&changed));
    for index in 0..input.defaults.len() {
        let mut changed = input.clone();
        changed.defaults.remove(index);
        assert!(!matches(&changed));
    }
}

#[test]
fn borrowed_input_checks_nonempty_spans_and_current_slots() {
    let mut root = ComputedValues::default();
    root.color = "black".parse().unwrap();
    let mut first = root.clone();
    first.font_family = CssFontFamily::Named("first face".into());
    let spans = vec![SourceSpan {
        selectable: true,
        source: Some(1u8),
        owners: vec![1],
        style: first.clone(),
        range: 0..5,
    }];
    let resolved = ResolvedShapeStyles {
        default_features: vec![],
        span_features: vec![vec![]],
        default_tab_stop: 8.0,
        span_tab_stops: vec![8.0],
    };
    let atom = |index, marker, edge, width, height| InlineAtom {
        source: 1u8,
        owners: vec![],
        index,
        fragment: Fragment::default(),
        line_width: width,
        line_box_height: height,
        baseline: 0.0,
        margin_left: 0.0,
        margin_top: 0.0,
        edge,
        paint: true,
        marker,
        empty_line: false,
        vertical_align: VerticalAlign::Baseline,
        font_size: 16.0,
        line_height: 20.0,
    };
    let atoms = vec![
        atom(0, true, false, 9.0, 8.0),
        atom(2, false, true, 3.0, 7.0),
        atom(4, false, false, 5.0, 6.0),
    ];
    let input = ShapeInput {
        text: "hello".into(),
        defaults: legacy_properties(&first, 0, vec![], None, 8.0),
        spans: vec![Span {
            range: 0..5,
            properties: legacy_properties(&first, 0, vec![], None, 8.0),
        }],
        slots: vec![
            Slot {
                kind: SlotKind::InFlow,
                id: 1,
                index: 2,
                width: 3.0f32.to_bits(),
                height: 0.0f32.to_bits(),
            },
            Slot {
                kind: SlotKind::InFlow,
                id: 2,
                index: 4,
                width: 5.0f32.to_bits(),
                height: 6.0f32.to_bits(),
            },
        ],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    let matches = |value: &ShapeInput, atoms: &[InlineAtom<u8>]| {
        value.matches_current("hello", &spans, atoms, &root, None, &resolved)
    };
    assert!(matches(&input, &atoms));
    let mut changed = input.clone();
    changed.defaults = legacy_properties(&root, 0, vec![], None, 8.0);
    assert!(!matches(&changed, &atoms));
    let mut changed = input.clone();
    changed.spans[0].range = 1..5;
    assert!(!matches(&changed, &atoms));
    let mut changed = input.clone();
    changed.spans[0].properties[0] = Property::Size(0);
    assert!(!matches(&changed, &atoms));
    for field in 0..5 {
        let mut changed = input.clone();
        match field {
            0 => changed.slots[0].id += 1,
            1 => changed.slots[0].index += 1,
            2 => changed.slots[0].width ^= 1,
            3 => changed.slots[0].height ^= 1,
            _ => changed.slots.swap(0, 1),
        }
        assert!(!matches(&changed, &atoms));
    }
    let mut changed = input.clone();
    changed.slots.pop();
    assert!(!matches(&changed, &atoms));
    let no_marker = vec![
        atom(0, false, false, 9.0, 8.0),
        atom(2, false, true, 3.0, 7.0),
        atom(4, false, false, 5.0, 6.0),
    ];
    assert!(!matches(&input, &no_marker));
    let edge_changed = vec![
        atom(0, true, false, 9.0, 8.0),
        atom(2, false, false, 3.0, 7.0),
        atom(4, false, false, 5.0, 6.0),
    ];
    assert!(!matches(&input, &edge_changed));
}
