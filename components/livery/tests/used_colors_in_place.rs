// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Used-color projection agrees with the former all-property algorithm while
//! preserving the source style's contextual expressions and unrelated fields.

use livery::media::SystemPalette;
use livery::values::{
    BackgroundImage, BoxShadow, Color, ColorScheme, ComputedColor, SystemColor, UsedColorContext,
};
use livery::{ComputedValues, PropertyId, PropertyValue, ValueType};

/// The former StylePlane lowering: each property is read as a tagged value,
/// its used colors are resolved, then it is assigned back to the style.
fn former_all_property_projection(
    source: &ComputedValues,
    context: UsedColorContext,
) -> ComputedValues {
    let mut result = source.clone();
    for &property in PropertyId::ALL {
        let value = match result.get(property) {
            PropertyValue::Color(color) => {
                PropertyValue::Color(ComputedColor::Absolute(color.resolve_used(context)))
            },
            PropertyValue::BackgroundImage(BackgroundImage::LinearGradient { from, to }) => {
                PropertyValue::BackgroundImage(BackgroundImage::LinearGradient {
                    from: ComputedColor::Absolute(from.resolve_used(context)),
                    to: ComputedColor::Absolute(to.resolve_used(context)),
                })
            },
            PropertyValue::BoxShadow(BoxShadow::Value(mut shadow)) => {
                shadow.color = ComputedColor::Absolute(shadow.color.resolve_used(context));
                PropertyValue::BoxShadow(BoxShadow::Value(shadow))
            },
            value => value,
        };
        result.set(property, value).unwrap();
    }
    result
}

fn populated_style(color: &str, image: &str, shadow: &str) -> ComputedValues {
    let mut values = ComputedValues::default();
    let mut families = [0, 0, 0];
    for &property in PropertyId::ALL {
        let metadata = property.metadata();
        let css = match metadata.value_type {
            ValueType::Color => {
                families[0] += 1;
                color
            },
            ValueType::BackgroundImage => {
                families[1] += 1;
                image
            },
            ValueType::BoxShadow => {
                families[2] += 1;
                shadow
            },
            // Also populate unrelated strings, lists and scalar fields so an
            // all-property equality check verifies that they stay unchanged.
            // Some catalog seeds describe function families ("perspective()")
            // rather than complete values; use a parsed seed or the initial.
            _ => {
                if let Some(value) = metadata
                    .seed_values
                    .iter()
                    .rev()
                    .find_map(|css| PropertyValue::parse(property, css).ok())
                {
                    values.set(property, value).unwrap();
                }
                continue;
            },
        };
        let value = PropertyValue::parse(property, css)
            .unwrap_or_else(|error| panic!("{}: {css}: {error:?}", metadata.name));
        values.set(property, value).unwrap();
    }
    assert!(
        families.into_iter().all(|count| count > 0),
        "exercise every supported color-bearing family"
    );
    values
}

fn palette() -> SystemPalette {
    let mut palette = SystemPalette::default();
    for (index, system) in SystemColor::ALL.into_iter().enumerate() {
        // Every system leaf has a distinct color under both schemes, so the
        // equivalence test cannot pass by always using a default palette.
        for (scheme, offset) in [(ColorScheme::Light, 16), (ColorScheme::Dark, 128)] {
            let channel = index + offset;
            let css = format!("#{channel:02x}4567");
            palette.set(scheme, system, css.parse::<Color>().unwrap());
        }
    }
    palette
}

#[test]
fn generated_used_colors_match_former_all_property_projection() {
    let mut colors = vec![
        "#2468ac".to_owned(),
        "currentcolor".to_owned(),
        "color-mix(in srgb, currentcolor 25%, Canvas 75%)".to_owned(),
        "rgb(from currentcolor r g b)".to_owned(),
        "alpha(from currentcolor / 0.5)".to_owned(),
        "color-layers(currentcolor)".to_owned(),
        "contrast-color(currentcolor)".to_owned(),
    ];
    colors.extend(
        SystemColor::ALL
            .into_iter()
            .map(|color| color.css_name().to_owned()),
    );
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let context = UsedColorContext::with_palette("#bf5218".parse().unwrap(), palette(), scheme);
        for color in &colors {
            let image = format!("linear-gradient({color}, Canvas)");
            let shadow = format!("inset 1px 2px 3px 4px {color}");
            let source = populated_style(color, &image, &shadow);
            let preserved = source.clone();
            let expected = former_all_property_projection(&source, context);
            let mut actual = source.clone();
            actual.resolve_used_colors_in_place(context);
            for &property in PropertyId::ALL {
                assert_eq!(
                    actual.get(property),
                    expected.get(property),
                    "{} with {color}, {scheme:?}",
                    property.metadata().name
                );
            }
            assert_eq!(
                source, preserved,
                "projection must not alter the retained source"
            );
            let resolved = actual.clone();
            actual.resolve_used_colors_in_place(context);
            assert_eq!(actual, resolved, "absolute used colors are idempotent");
        }
    }
}

#[test]
fn generated_used_colors_follow_supplied_foreground_and_palette() {
    let foreground: Color = "#b83535".parse().unwrap();
    let palette = palette();
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let context = UsedColorContext::with_palette(foreground, palette, scheme);
        let mut current = populated_style(
            "currentcolor",
            "linear-gradient(currentcolor, currentcolor)",
            "1px 2px currentcolor",
        );
        current.resolve_used_colors_in_place(context);
        for &property in PropertyId::ALL {
            match current.get(property) {
                PropertyValue::Color(color) => {
                    assert_eq!(color, ComputedColor::Absolute(foreground))
                },
                PropertyValue::BackgroundImage(BackgroundImage::LinearGradient { from, to }) => {
                    assert_eq!(from, ComputedColor::Absolute(foreground));
                    assert_eq!(to, ComputedColor::Absolute(foreground));
                },
                PropertyValue::BoxShadow(BoxShadow::Value(shadow)) => {
                    assert_eq!(shadow.color, ComputedColor::Absolute(foreground))
                },
                _ => {},
            }
        }
        for system in SystemColor::ALL {
            let css = system.css_name();
            let mut values = populated_style(
                css,
                &format!("linear-gradient({css}, {css})"),
                &format!("1px 2px {css}"),
            );
            values.resolve_used_colors_in_place(context);
            let expected = ComputedColor::Absolute(palette.get(scheme, system));
            assert_eq!(values.color, expected);
            assert_eq!(
                values.background_image,
                BackgroundImage::LinearGradient {
                    from: expected.clone(),
                    to: expected.clone()
                }
            );
            let BoxShadow::Value(shadow) = values.box_shadow else {
                panic!("shadow")
            };
            assert_eq!(shadow.color, expected);
        }
    }
}

#[test]
fn generated_used_colors_preserve_non_color_image_and_shadow_forms() {
    let context = UsedColorContext::legacy("#369abc".parse().unwrap());
    for image in ["none", "url(https://example.invalid/retained-image.png)"] {
        let source = populated_style("currentcolor", image, "none");
        let mut actual = source.clone();
        actual.resolve_used_colors_in_place(context);
        assert_eq!(actual, former_all_property_projection(&source, context));
        assert_eq!(actual.background_image, source.background_image);
        assert_eq!(actual.box_shadow, BoxShadow::None);
    }
}
