// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Visual transform values: opacity, clip paths, rotate and scale, the
//! transform list and its functions, and box shadows.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(f32);

impl Opacity {
    pub const ONE: Self = Self(1.0);

    pub const fn from_value(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub const fn value(self) -> f32 {
        self.0
    }
}

impl FromStr for Opacity {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let value = if let Some(percentage) = input.strip_suffix('%') {
            percentage.trim().parse::<f32>().map(|value| value / 100.0)
        } else {
            input.parse::<f32>()
        }
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| ParseError::expected("a finite opacity number or percentage"))?;
        Ok(Self(value.clamp(0.0, 1.0)))
    }
}

impl fmt::Display for Opacity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&format_number(self.0))
    }
}

/// The bounded 2D individual `rotate` property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotate {
    None,
    Angle(f32),
    /// An angle expression whose bases are not all constant, retained until
    /// cascade supplies the element's tree context.
    Deferred(MathLengthPercentage),
    /// `rotate: <axis> <angle>`. An axis parallel to +Z normalizes to
    /// [`Rotate::Angle`] at parse time, which is also how CSS serializes it.
    Axis(f32, f32, f32, f32),
}

impl Rotate {
    pub const fn radians(self) -> Option<f32> {
        match self {
            Self::None | Self::Deferred(_) | Self::Axis(..) => None,
            Self::Angle(value) => Some(value),
        }
    }

    /// The individual property's own 4x4, or `None` for `none`.
    pub fn to_matrix_3d(self) -> Option<Matrix3D> {
        match self {
            Self::None | Self::Deferred(_) => None,
            Self::Angle(angle) => Matrix3D::rotation(0.0, 0.0, 1.0, angle),
            Self::Axis(x, y, z, angle) => Matrix3D::rotation(x, y, z, angle),
        }
    }

    pub(in crate::values) fn resolve_math(self, environment: RelativeLengthEnvironment) -> Self {
        match self {
            Self::Deferred(math) => {
                let resolved = math.resolve_relative(environment);
                resolved
                    .resolved_px()
                    .map_or(Self::Deferred(resolved), Self::Angle)
            },
            value => value,
        }
    }
}

impl FromStr for Rotate {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        if let Some(axis) = parse_rotate_axis(input) {
            return axis;
        }
        parse_angle(input)
            .or_else(|_| crate::values::calc::parse_angle(input))
            .map(Self::Angle)
            .or_else(|error| {
                crate::values::calc::parse_angle_math(input)
                    .map(Self::Deferred)
                    .map_err(|_| error)
            })
    }
}

impl fmt::Display for Rotate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Angle(value) => write!(formatter, "{}rad", format_number(*value)),
            Self::Deferred(math) => math.fmt(formatter),
            Self::Axis(x, y, z, angle) => write!(
                formatter,
                "{} {} {} {}rad",
                format_number(*x),
                format_number(*y),
                format_number(*z),
                format_number(*angle)
            ),
        }
    }
}

/// `[ x | y | z | <number>{3} ] && <angle>`, in either order.
fn parse_rotate_axis(input: &str) -> Option<Result<Rotate, ParseError>> {
    let parts = shadow_components(input);
    let (axis, angle) = match parts.as_slice() {
        [keyword, angle] | [angle, keyword] if axis_keyword(keyword).is_some() => {
            (axis_keyword(keyword)?, *angle)
        },
        [a, b, c, d] => match parse_axis_numbers(a, b, c) {
            Some(axis) => (axis, *d),
            None => (parse_axis_numbers(b, c, d)?, *a),
        },
        _ => return None,
    };
    let angle = match parse_angle(angle)
        .or_else(|error| crate::values::calc::parse_angle(angle).map_err(|_| error))
    {
        Ok(angle) => angle,
        Err(error) => return Some(Err(error)),
    };
    if axis == (0.0, 0.0, 0.0) {
        return Some(Err(ParseError::expected("a non-zero rotation axis")));
    }
    // An axis along +Z is plain 2D rotation, and CSS serializes it that way.
    Some(Ok(if axis.0 == 0.0 && axis.1 == 0.0 && axis.2 > 0.0 {
        Rotate::Angle(angle)
    } else {
        Rotate::Axis(axis.0, axis.1, axis.2, angle)
    }))
}

fn axis_keyword(value: &str) -> Option<(f32, f32, f32)> {
    match value.to_ascii_lowercase().as_str() {
        "x" => Some((1.0, 0.0, 0.0)),
        "y" => Some((0.0, 1.0, 0.0)),
        "z" => Some((0.0, 0.0, 1.0)),
        _ => None,
    }
}

fn parse_axis_numbers(x: &str, y: &str, z: &str) -> Option<(f32, f32, f32)> {
    let number = |value: &str| value.parse::<f32>().ok().filter(|value| value.is_finite());
    Some((number(x)?, number(y)?, number(z)?))
}

/// The bounded uniform individual `scale` property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    None,
    Uniform(f32),
    /// The [`Rotate::Deferred`] twin for number-valued expressions.
    Deferred(MathLengthPercentage),
    /// `scale: <x> <y> <z>`. A list equal to `x x 1` normalizes to
    /// [`Scale::Uniform`] at parse time, which is also how CSS serializes it.
    Values(f32, f32, f32),
}

impl Scale {
    pub const fn factor(self) -> Option<f32> {
        match self {
            Self::None | Self::Deferred(_) | Self::Values(..) => None,
            Self::Uniform(value) => Some(value),
        }
    }

    /// The individual property's own 4x4, or `None` for `none`.
    pub fn to_matrix_3d(self) -> Option<Matrix3D> {
        match self {
            Self::None | Self::Deferred(_) => None,
            Self::Uniform(value) => Some(Matrix3D::scaling(value, value, 1.0)),
            Self::Values(x, y, z) => Some(Matrix3D::scaling(x, y, z)),
        }
    }

    pub(in crate::values) fn resolve_math(self, environment: RelativeLengthEnvironment) -> Self {
        match self {
            Self::Deferred(math) => {
                let resolved = math.resolve_relative(environment);
                resolved
                    .resolved_px()
                    .map_or(Self::Deferred(resolved), Self::Uniform)
            },
            value => value,
        }
    }
}

impl FromStr for Scale {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        if let Some(values) = parse_scale_values(input) {
            return values;
        }
        let value = input
            .strip_suffix('%')
            .and_then(|value| value.trim().parse::<f32>().ok())
            .map(|value| value / 100.0)
            .or_else(|| input.parse::<f32>().ok())
            .filter(|value| value.is_finite())
            .map(Ok)
            .unwrap_or_else(|| crate::values::calc::parse_number(input));
        match value {
            Ok(value) => Ok(Self::Uniform(value)),
            Err(error) => crate::values::calc::parse_number_math(input)
                .map(Self::Deferred)
                .map_err(|_| error),
        }
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Uniform(value) => formatter.write_str(&format_number(*value)),
            Self::Deferred(math) => math.fmt(formatter),
            Self::Values(x, y, z) => write!(
                formatter,
                "{} {} {}",
                format_number(*x),
                format_number(*y),
                format_number(*z)
            ),
        }
    }
}

/// The two- and three-component forms of the individual `scale` property.
fn parse_scale_values(input: &str) -> Option<Result<Scale, ParseError>> {
    let parts = shadow_components(input);
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let mut values = [1.0_f32; 3];
    for (slot, part) in values.iter_mut().zip(&parts) {
        match scale_number(part) {
            Some(value) => *slot = value,
            None => return Some(Err(ParseError::expected("a scale number or percentage"))),
        }
    }
    Some(Ok(if values[0] == values[1] && values[2] == 1.0 {
        Scale::Uniform(values[0])
    } else {
        Scale::Values(values[0], values[1], values[2])
    }))
}

fn scale_number(value: &str) -> Option<f32> {
    value
        .strip_suffix('%')
        .and_then(|value| value.trim().parse::<f32>().ok())
        .map(|value| value / 100.0)
        .or_else(|| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
}

/// The individual `translate` property: `none | <length-percentage>{1,2} <length>?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Translate {
    None,
    Values(LengthPercentage, LengthPercentage, Length),
}

impl Translate {
    /// The used 4x4 against the transformed box, or `None` for `none`.
    pub fn to_matrix_3d(self, em: f32, reference_box: (f32, f32)) -> Option<Matrix3D> {
        let Self::Values(x, y, z) = self else {
            return None;
        };
        Some(Matrix3D::translation(
            x.to_px(em, 16.0, reference_box.0),
            y.to_px(em, 16.0, reference_box.1),
            z.unit.to_px(z.value, em, 16.0),
        ))
    }

    /// CSS Transforms 2 interpolates `none` as the identity translation, but
    /// only when the other endpoint is not `none` too.
    const IDENTITY: Self =
        Self::Values(LengthPercentage::ZERO, LengthPercentage::ZERO, Length::ZERO);

    pub fn interpolate(self, other: Self, progress: f32) -> Self {
        let (from, to) = match (self, other) {
            (Self::None, Self::None) => return Self::None,
            (Self::None, other) => (Self::IDENTITY, other),
            (value, Self::None) => (value, Self::IDENTITY),
            pair => pair,
        };
        match (from, to) {
            (Self::Values(fx, fy, fz), Self::Values(tx, ty, tz)) if fz.unit == tz.unit => {
                let progress = progress.clamp(0.0, 1.0);
                Self::Values(
                    fx.interpolate(tx, progress),
                    fy.interpolate(ty, progress),
                    Length {
                        value: fz.value + (tz.value - fz.value) * progress,
                        unit: fz.unit,
                    },
                )
            },
            _ if progress < 0.5 => from,
            _ => to,
        }
    }
}

impl FromStr for Translate {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        let parts = shadow_components(input);
        let (x, y, z) = match parts.as_slice() {
            [x] => (x.parse()?, LengthPercentage::ZERO, Length::ZERO),
            [x, y] => (x.parse()?, y.parse()?, Length::ZERO),
            [x, y, z] => (x.parse()?, y.parse()?, z.parse()?),
            _ => return Err(ParseError::expected("none or one to three translations")),
        };
        Ok(Self::Values(x, y, z))
    }
}

impl fmt::Display for Translate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Values(x, y, z) = self else {
            return formatter.write_str("none");
        };
        // A trailing zero component is dropped; one that is kept serializes
        // with its authored unit.
        formatter.write_str(&x.to_css_with_unit())?;
        if z.value != 0.0 {
            return write!(
                formatter,
                " {} {}",
                y.to_css_with_unit(),
                z.to_css_with_unit()
            );
        }
        let is_zero = match y {
            LengthPercentage::Zero => true,
            LengthPercentage::Length(length) => length.value == 0.0,
            LengthPercentage::Percentage(value) => *value == 0.0,
            _ => false,
        };
        if !is_zero {
            return write!(formatter, " {}", y.to_css_with_unit());
        }
        Ok(())
    }
}

/// The `perspective` property: the depth of the 3D rendering context this
/// element establishes for its children.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Perspective {
    None,
    Depth(Length),
}

impl Perspective {
    pub fn depth_px(self, em: f32) -> Option<f32> {
        match self {
            Self::None => None,
            Self::Depth(length) => {
                let value = length.unit.to_px(length.value, em, 16.0);
                (value.is_finite() && value > 0.0).then_some(value)
            },
        }
    }
}

impl FromStr for Perspective {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        let length = input.parse::<Length>()?;
        if length.value < 0.0 {
            return Err(ParseError::expected("a non-negative perspective depth"));
        }
        Ok(Self::Depth(length))
    }
}

impl fmt::Display for Perspective {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Depth(length) => length.fmt(formatter),
        }
    }
}

/// The `perspective-origin` property: the vanishing point of the perspective
/// this element establishes, relative to its own border box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerspectiveOrigin {
    pub x: LengthPercentage,
    pub y: LengthPercentage,
}

impl PerspectiveOrigin {
    pub const CENTER: Self = Self {
        x: LengthPercentage::Percentage(0.5),
        y: LengthPercentage::Percentage(0.5),
    };

    pub fn used(self, em: f32, width: f32, height: f32) -> (f32, f32) {
        (self.x.to_px(em, em, width), self.y.to_px(em, em, height))
    }

    pub fn interpolate(self, other: Self, progress: f32) -> Self {
        Self {
            x: self.x.interpolate(other.x, progress),
            y: self.y.interpolate(other.y, progress),
        }
    }
}

impl FromStr for PerspectiveOrigin {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let values = shadow_components(input.trim());
        let (x, y) = match values.as_slice() {
            [first] => parse_transform_origin_xy(first, None)?,
            [first, second] => parse_transform_origin_xy(first, Some(*second))?,
            _ => {
                return Err(ParseError::expected(
                    "one or two perspective-origin positions",
                ));
            },
        };
        Ok(Self { x, y })
    }
}

impl fmt::Display for PerspectiveOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.x, self.y)
    }
}

/// `transform-style`: whether this element flattens its children into its own
/// plane or extends its 3D rendering context to them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformStyle {
    Flat,
    Preserve3d,
}

impl FromStr for TransformStyle {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "flat" => Ok(Self::Flat),
            "preserve-3d" => Ok(Self::Preserve3d),
            _ => Err(ParseError::expected("flat or preserve-3d")),
        }
    }
}

impl fmt::Display for TransformStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Flat => "flat",
            Self::Preserve3d => "preserve-3d",
        })
    }
}

/// `backface-visibility`: whether a box whose transformed normal points away
/// from the viewer is painted at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackfaceVisibility {
    Visible,
    Hidden,
}

impl FromStr for BackfaceVisibility {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "visible" => Ok(Self::Visible),
            "hidden" => Ok(Self::Hidden),
            _ => Err(ParseError::expected("visible or hidden")),
        }
    }
}

impl fmt::Display for BackfaceVisibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
        })
    }
}

/// The CSS `transform-origin` point. The Z length is retained so valid 2D
/// declarations keep their CSS meaning; the current renderer has only a 2D
/// transform matrix, where that coordinate has no effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    pub x: LengthPercentage,
    pub y: LengthPercentage,
    pub z: Length,
}

impl TransformOrigin {
    pub const CENTER: Self = Self {
        x: LengthPercentage::Percentage(0.5),
        y: LengthPercentage::Percentage(0.5),
        z: Length::ZERO,
    };

    /// Resolve the two in-plane coordinates against the transformed element's
    /// border box, as CSS Transforms defines for the default view box.
    pub fn used_2d(self, em: f32, width: f32, height: f32) -> (f32, f32) {
        (self.x.to_px(em, em, width), self.y.to_px(em, em, height))
    }

    pub fn interpolate(self, other: Self, progress: f32) -> Self {
        let z = if self.z.unit == other.z.unit {
            Length {
                value: self.z.value + (other.z.value - self.z.value) * progress.clamp(0.0, 1.0),
                unit: self.z.unit,
            }
        } else if progress.clamp(0.0, 1.0) < 0.5 {
            self.z
        } else {
            other.z
        };
        Self {
            x: self.x.interpolate(other.x, progress),
            y: self.y.interpolate(other.y, progress),
            z,
        }
    }
}

impl FromStr for TransformOrigin {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let values = shadow_components(input.trim());
        let (xy, z) = match values.as_slice() {
            [first] => (parse_transform_origin_xy(first, None)?, Length::ZERO),
            [first, second] => (
                parse_transform_origin_xy(first, Some(*second))?,
                Length::ZERO,
            ),
            [first, second, z] => (
                parse_transform_origin_xy(first, Some(*second))?,
                z.parse::<Length>()?,
            ),
            _ => {
                return Err(ParseError::expected(
                    "one or two transform-origin positions and an optional Z length",
                ));
            },
        };
        Ok(Self {
            x: xy.0,
            y: xy.1,
            z,
        })
    }
}

fn parse_transform_origin_xy(
    first: &str,
    second: Option<&str>,
) -> Result<(LengthPercentage, LengthPercentage), ParseError> {
    let center = LengthPercentage::Percentage(0.5);
    let horizontal = |value: &str| match value.trim().to_ascii_lowercase().as_str() {
        "left" => Ok(LengthPercentage::ZERO),
        "right" => Ok(LengthPercentage::Percentage(1.0)),
        "center" => Ok(center),
        "top" | "bottom" => Err(ParseError::expected("a horizontal transform-origin value")),
        _ => value.parse(),
    };
    let vertical = |value: &str| match value.trim().to_ascii_lowercase().as_str() {
        "top" => Ok(LengthPercentage::ZERO),
        "bottom" => Ok(LengthPercentage::Percentage(1.0)),
        "center" => Ok(center),
        "left" | "right" => Err(ParseError::expected("a vertical transform-origin value")),
        _ => value.parse(),
    };
    let keyword = |value: &str| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "left" | "right" | "top" | "bottom" | "center"
        )
    };
    let first_is_vertical = matches!(first.trim().to_ascii_lowercase().as_str(), "top" | "bottom");
    let second_is_horizontal = second.is_some_and(|value| {
        matches!(value.trim().to_ascii_lowercase().as_str(), "left" | "right")
    });
    match second {
        None if first_is_vertical => Ok((center, vertical(first)?)),
        None => Ok((horizontal(first)?, center)),
        // CSS permits a reversed pair only when both components are
        // keywords. `top 10px` and `10px left` are therefore invalid rather
        // than a swapped two-axis position.
        Some(second)
            if keyword(first) && keyword(second) && (first_is_vertical || second_is_horizontal) =>
        {
            Ok((horizontal(second)?, vertical(first)?))
        },
        Some(second) => Ok((horizontal(first)?, vertical(second)?)),
    }
}

impl fmt::Display for TransformOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.x, self.y)?;
        if self.z != Length::ZERO {
            write!(formatter, " {}", self.z)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Transform {
    None,
    Functions(Vec<TransformFunction>),
}

/// The bounded `clip-path` subset used by Cambium's tile geometry.
///
/// A polygon coordinate is a regular Livery length-percentage. The cascade
/// resolves its environment-relative terms before paint supplies the element
/// box as the percentage basis.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipPath {
    None,
    Polygon(Vec<(LengthPercentage, LengthPercentage)>),
}

impl ClipPath {
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Resolve the polygon against its border-box dimensions for the shared
    /// paint and hit-testing paths. `None` deliberately has no geometry.
    pub fn polygon_points(&self, width: f32, height: f32) -> Option<Vec<(f32, f32)>> {
        let Self::Polygon(points) = self else {
            return None;
        };
        Some(
            points
                .iter()
                .map(|(x, y)| (x.to_px(16.0, 16.0, width), y.to_px(16.0, 16.0, height)))
                .collect(),
        )
    }
}

impl FromStr for ClipPath {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        let open = input
            .find('(')
            .ok_or_else(|| ParseError::expected("none or polygon()"))?;
        if !input[..open].trim().eq_ignore_ascii_case("polygon") || !input.ends_with(')') {
            return Err(ParseError::expected("none or polygon()"));
        }
        let arguments = &input[open + 1..input.len() - 1];
        let mut points = Vec::new();
        for point in polygon_arguments(arguments)? {
            let coordinates = shadow_components(point);
            let [x, y] = coordinates.as_slice() else {
                return Err(ParseError::expected("two polygon coordinates"));
            };
            points.push((x.parse()?, y.parse()?));
        }
        if points.len() < 3 {
            return Err(ParseError::expected("at least three polygon points"));
        }
        Ok(Self::Polygon(points))
    }
}

fn polygon_arguments(input: &str) -> Result<Vec<&str>, ParseError> {
    let mut arguments = Vec::new();
    let mut start = 0;
    let mut depth = 0_u32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            ')' => return Err(ParseError::expected("closed polygon coordinates")),
            ',' if depth == 0 => {
                let argument = input[start..index].trim();
                if argument.is_empty() {
                    return Err(ParseError::expected("a polygon coordinate pair"));
                }
                arguments.push(argument);
                start = index + 1;
            },
            _ => {},
        }
    }
    if depth != 0 {
        return Err(ParseError::expected("closed polygon coordinates"));
    }
    let argument = input[start..].trim();
    if argument.is_empty() {
        return Err(ParseError::expected("a polygon coordinate pair"));
    }
    arguments.push(argument);
    Ok(arguments)
}

impl fmt::Display for ClipPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Polygon(points) => {
                formatter.write_str("polygon(")?;
                for (index, (x, y)) in points.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{x} {y}")?;
                }
                formatter.write_str(")")
            },
        }
    }
}

/// A bounded single-layer CSS box shadow.
#[derive(Clone, Debug, PartialEq)]
pub enum BoxShadow {
    None,
    Value(BoxShadowValue),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadowValue {
    pub inset: bool,
    pub offset_x: Length,
    pub offset_y: Length,
    pub blur_radius: Length,
    pub spread_radius: Length,
    pub color: ComputedColor,
}

impl BoxShadow {
    /// Interpolate the bounded single-shadow form used by the retained paint
    /// lane. Matching length units and inset mode are required; `none`, mixed
    /// units, and mode changes stay discrete until the shadow-list ratchet.
    pub fn interpolate(&self, other: &Self, progress: f32) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        let value = match (self, other) {
            (Self::Value(from), Self::Value(to)) if from.inset == to.inset => {
                interpolate_box_shadow_value(from, to, progress).map(Self::Value)
            },
            _ => None,
        };
        value.unwrap_or_else(|| {
            if progress < 0.5 {
                self.clone()
            } else {
                other.clone()
            }
        })
    }

    /// Interpolate a matching shadow after resolving the two color endpoints
    /// under their respective used-value contexts.
    pub fn interpolate_used(
        &self,
        other: &Self,
        from_context: UsedColorContext,
        to_context: UsedColorContext,
        progress: f32,
    ) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        let value = match (self, other) {
            (Self::Value(from), Self::Value(to)) if from.inset == to.inset => {
                interpolate_box_shadow_value(from, to, progress).map(|mut value| {
                    value.color =
                        from.color
                            .interpolate_used(&to.color, from_context, to_context, progress);
                    Self::Value(value)
                })
            },
            _ => None,
        };
        value.unwrap_or_else(|| {
            if progress < 0.5 {
                self.clone()
            } else {
                other.clone()
            }
        })
    }
}

fn interpolate_box_shadow_value(
    from: &BoxShadowValue,
    to: &BoxShadowValue,
    progress: f32,
) -> Option<BoxShadowValue> {
    Some(BoxShadowValue {
        inset: from.inset,
        offset_x: interpolate_length(from.offset_x, to.offset_x, progress)?,
        offset_y: interpolate_length(from.offset_y, to.offset_y, progress)?,
        blur_radius: interpolate_length(from.blur_radius, to.blur_radius, progress)?,
        spread_radius: interpolate_length(from.spread_radius, to.spread_radius, progress)?,
        color: from.color.interpolate(&to.color, progress),
    })
}

impl FromStr for BoxShadow {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        let mut inset = false;
        let mut color = None;
        let mut lengths = Vec::new();
        for component in shadow_components(input) {
            if component.eq_ignore_ascii_case("inset") {
                if inset {
                    return Err(ParseError::expected("one inset box-shadow keyword"));
                }
                inset = true;
            } else if let Ok(value) = component.parse::<ComputedColor>() {
                if color.replace(value).is_some() {
                    return Err(ParseError::expected("one box-shadow color"));
                }
            } else if let Ok(value) = component.parse::<Length>() {
                lengths.push(value);
            } else {
                return Err(ParseError::expected("a bounded box-shadow component"));
            }
        }
        if !(2..=4).contains(&lengths.len()) {
            return Err(ParseError::expected("two through four box-shadow lengths"));
        }
        Ok(Self::Value(BoxShadowValue {
            inset,
            offset_x: lengths[0],
            offset_y: lengths[1],
            blur_radius: lengths.get(2).copied().unwrap_or(Length::ZERO),
            spread_radius: lengths.get(3).copied().unwrap_or(Length::ZERO),
            color: color.unwrap_or(ComputedColor::CURRENT_COLOR),
        }))
    }
}

impl fmt::Display for BoxShadow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Value(value) => {
                write!(
                    formatter,
                    "{} {} {} {} {}",
                    value.offset_x,
                    value.offset_y,
                    value.blur_radius,
                    value.spread_radius,
                    value.color
                )?;
                if value.inset {
                    formatter.write_str(" inset")?;
                }
                Ok(())
            },
        }
    }
}

pub(crate) fn shadow_components(input: &str) -> Vec<&str> {
    let mut components = Vec::new();
    let mut start = None;
    let mut depth = 0_u32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => {
                start.get_or_insert(index);
                depth += 1;
            },
            ')' => depth = depth.saturating_sub(1),
            _ if ch.is_ascii_whitespace() && depth == 0 => {
                if let Some(offset) = start.take() {
                    components.push(&input[offset..index]);
                }
            },
            _ => {
                start.get_or_insert(index);
            },
        }
    }
    if let Some(offset) = start {
        components.push(&input[offset..]);
    }
    components
}

impl Transform {
    pub fn functions(&self) -> Option<&[TransformFunction]> {
        match self {
            Self::None => None,
            Self::Functions(functions) => Some(functions),
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Interpolate matching transform primitives directly, then normalize any
    /// mismatched suffix (including `none`) through a decomposed 2D matrix.
    pub fn interpolate(&self, other: &Self, progress: f32) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        if matches!((self, other), (Self::None, Self::None)) {
            return Self::None;
        }

        let from = self.functions().unwrap_or(&[]);
        let to = other.functions().unwrap_or(&[]);
        let mut functions = Vec::new();
        let mut prefix = 0;
        while let (Some(from), Some(to)) = (from.get(prefix), to.get(prefix)) {
            let Some(value) = interpolate_transform_function(*from, *to, progress) else {
                break;
            };
            functions.push(value);
            prefix += 1;
        }
        if prefix == from.len() && prefix == to.len() {
            return Self::Functions(functions);
        }

        let from_matrix = Matrix2D::from_absolute_functions(&from[prefix..], 16.0);
        let to_matrix = Matrix2D::from_absolute_functions(&to[prefix..], 16.0);
        match from_matrix
            .zip(to_matrix)
            .and_then(|(from, to)| from.interpolate(to, progress))
        {
            Some(matrix) => {
                functions.push(TransformFunction::Matrix(matrix));
                Self::Functions(functions)
            },
            None if progress < 0.5 => self.clone(),
            None => other.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransformFunction {
    Translate(LengthPercentage, LengthPercentage),
    /// `translate3d()`. `translateZ()` is this with zero in-plane terms; the
    /// two keep distinct variants because their computed serializations differ.
    Translate3D(LengthPercentage, LengthPercentage, Length),
    TranslateZ(Length),
    Scale(f32, f32),
    Scale3D(f32, f32, f32),
    ScaleZ(f32),
    Rotate(f32),
    RotateX(f32),
    RotateY(f32),
    RotateZ(f32),
    /// `rotate3d(x, y, z, angle)`. The axis is kept as authored so the
    /// computed value serializes the authored numbers, not a normalized axis.
    Rotate3D(f32, f32, f32, f32),
    Skew(f32, f32),
    /// `perspective(<length> | none)`. `none` is the identity.
    Perspective(Option<Length>),
    Matrix(Matrix2D),
    Matrix3D(Matrix3D),
}

impl TransformFunction {
    /// Whether this function needs a Z axis the affine lowering cannot carry.
    pub const fn is_3d(&self) -> bool {
        !matches!(
            self,
            Self::Translate(..) | Self::Scale(..) | Self::Rotate(_) | Self::Skew(..)
        ) && !matches!(self, Self::Matrix(_))
    }
}

fn interpolate_transform_function(
    from: TransformFunction,
    to: TransformFunction,
    progress: f32,
) -> Option<TransformFunction> {
    let scalar = |from: f32, to: f32| from + (to - from) * progress;
    Some(match (from, to) {
        (
            TransformFunction::Translate(from_x, from_y),
            TransformFunction::Translate(to_x, to_y),
        ) => TransformFunction::Translate(
            from_x.interpolate(to_x, progress),
            from_y.interpolate(to_y, progress),
        ),
        (TransformFunction::Scale(from_x, from_y), TransformFunction::Scale(to_x, to_y)) => {
            TransformFunction::Scale(scalar(from_x, to_x), scalar(from_y, to_y))
        },
        (TransformFunction::Rotate(from), TransformFunction::Rotate(to)) => {
            TransformFunction::Rotate(scalar(from, to))
        },
        (TransformFunction::Skew(from_x, from_y), TransformFunction::Skew(to_x, to_y)) => {
            TransformFunction::Skew(scalar(from_x, to_x), scalar(from_y, to_y))
        },
        (TransformFunction::Matrix(from), TransformFunction::Matrix(to)) => {
            TransformFunction::Matrix(from.interpolate(to, progress)?)
        },
        (
            TransformFunction::Translate3D(from_x, from_y, from_z),
            TransformFunction::Translate3D(to_x, to_y, to_z),
        ) => TransformFunction::Translate3D(
            from_x.interpolate(to_x, progress),
            from_y.interpolate(to_y, progress),
            interpolate_length(from_z, to_z, progress)?,
        ),
        (TransformFunction::TranslateZ(from), TransformFunction::TranslateZ(to)) => {
            TransformFunction::TranslateZ(interpolate_length(from, to, progress)?)
        },
        (
            TransformFunction::Scale3D(from_x, from_y, from_z),
            TransformFunction::Scale3D(to_x, to_y, to_z),
        ) => TransformFunction::Scale3D(
            scalar(from_x, to_x),
            scalar(from_y, to_y),
            scalar(from_z, to_z),
        ),
        (TransformFunction::ScaleZ(from), TransformFunction::ScaleZ(to)) => {
            TransformFunction::ScaleZ(scalar(from, to))
        },
        (TransformFunction::RotateX(from), TransformFunction::RotateX(to)) => {
            TransformFunction::RotateX(scalar(from, to))
        },
        (TransformFunction::RotateY(from), TransformFunction::RotateY(to)) => {
            TransformFunction::RotateY(scalar(from, to))
        },
        (TransformFunction::RotateZ(from), TransformFunction::RotateZ(to)) => {
            TransformFunction::RotateZ(scalar(from, to))
        },
        // Same-axis rotate3d interpolates its angle; a differing axis has no
        // primitive interpolation and falls to the list's matrix path.
        (
            TransformFunction::Rotate3D(fx, fy, fz, from),
            TransformFunction::Rotate3D(tx, ty, tz, to),
        ) if (fx, fy, fz) == (tx, ty, tz) => {
            TransformFunction::Rotate3D(fx, fy, fz, scalar(from, to))
        },
        (TransformFunction::Perspective(from), TransformFunction::Perspective(to)) => {
            // `none` is an infinite depth, which has no finite midpoint.
            let (Some(from), Some(to)) = (from, to) else {
                return None;
            };
            TransformFunction::Perspective(Some(interpolate_length(from, to, progress)?))
        },
        _ => return None,
    })
}

fn interpolate_length(from: Length, to: Length, progress: f32) -> Option<Length> {
    (from.unit == to.unit).then_some(Length {
        value: from.value + (to.value - from.value) * progress,
        unit: from.unit,
    })
}

impl FromStr for Transform {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut input = input.trim();
        if input.eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }

        let mut functions = Vec::new();
        while !input.is_empty() {
            let open = input
                .find('(')
                .ok_or_else(|| ParseError::expected("a supported 2D transform function"))?;
            let name = input[..open].trim().to_ascii_lowercase();
            if name.is_empty() || name.split_ascii_whitespace().count() != 1 {
                return Err(ParseError::expected("a supported 2D transform function"));
            }
            let tail = &input[open + 1..];
            let close = tail
                .find(')')
                .ok_or_else(|| ParseError::expected("a closed 2D transform function"))?;
            let arguments = tail[..close]
                .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>();
            functions.push(parse_transform_function(&name, &arguments)?);
            input = tail[close + 1..].trim_start();
        }
        if functions.is_empty() {
            Err(ParseError::expected("none or a 2D transform list"))
        } else {
            Ok(Self::Functions(functions))
        }
    }
}

fn parse_transform_function(
    name: &str,
    arguments: &[&str],
) -> Result<TransformFunction, ParseError> {
    let length_percentage = |value: &str| value.parse::<LengthPercentage>();
    let length = |value: &str| value.parse::<Length>();
    let number = |value: &str| {
        value
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| ParseError::expected("a finite transform number"))
    };
    match (name, arguments) {
        ("translate", [x]) => Ok(TransformFunction::Translate(
            length_percentage(x)?,
            LengthPercentage::ZERO,
        )),
        ("translate", [x, y]) => Ok(TransformFunction::Translate(
            length_percentage(x)?,
            length_percentage(y)?,
        )),
        ("translatex", [x]) => Ok(TransformFunction::Translate(
            length_percentage(x)?,
            LengthPercentage::ZERO,
        )),
        ("translatey", [y]) => Ok(TransformFunction::Translate(
            LengthPercentage::ZERO,
            length_percentage(y)?,
        )),
        ("scale", [both]) => {
            let both = number(both)?;
            Ok(TransformFunction::Scale(both, both))
        },
        ("scale", [x, y]) => Ok(TransformFunction::Scale(number(x)?, number(y)?)),
        ("scalex", [x]) => Ok(TransformFunction::Scale(number(x)?, 1.0)),
        ("scaley", [y]) => Ok(TransformFunction::Scale(1.0, number(y)?)),
        ("rotate", [angle]) => Ok(TransformFunction::Rotate(parse_angle(angle)?)),
        ("skew", [x]) => Ok(TransformFunction::Skew(parse_angle(x)?, 0.0)),
        ("skew", [x, y]) => Ok(TransformFunction::Skew(parse_angle(x)?, parse_angle(y)?)),
        ("skewx", [x]) => Ok(TransformFunction::Skew(parse_angle(x)?, 0.0)),
        ("skewy", [y]) => Ok(TransformFunction::Skew(0.0, parse_angle(y)?)),
        ("matrix", [a, b, c, d, e, f]) => Ok(TransformFunction::Matrix(Matrix2D::new(
            number(a)?,
            number(b)?,
            number(c)?,
            number(d)?,
            number(e)?,
            number(f)?,
        ))),
        ("translate3d", [x, y, z]) => Ok(TransformFunction::Translate3D(
            length_percentage(x)?,
            length_percentage(y)?,
            length(z)?,
        )),
        ("translatez", [z]) => Ok(TransformFunction::TranslateZ(length(z)?)),
        ("scale3d", [x, y, z]) => Ok(TransformFunction::Scale3D(
            number(x)?,
            number(y)?,
            number(z)?,
        )),
        ("scalez", [z]) => Ok(TransformFunction::ScaleZ(number(z)?)),
        ("rotatex", [angle]) => Ok(TransformFunction::RotateX(angle_value(angle)?)),
        ("rotatey", [angle]) => Ok(TransformFunction::RotateY(angle_value(angle)?)),
        ("rotatez", [angle]) => Ok(TransformFunction::RotateZ(angle_value(angle)?)),
        ("rotate3d", [x, y, z, angle]) => {
            let (x, y, z) = (number(x)?, number(y)?, number(z)?);
            if x == 0.0 && y == 0.0 && z == 0.0 {
                // A zero axis leaves the rotation undefined, which makes the
                // whole declaration invalid rather than an identity.
                return Err(ParseError::expected("a non-zero rotate3d axis"));
            }
            Ok(TransformFunction::Rotate3D(x, y, z, angle_value(angle)?))
        },
        ("perspective", [depth]) => {
            if depth.trim().eq_ignore_ascii_case("none") {
                return Ok(TransformFunction::Perspective(None));
            }
            let depth = length(depth)?;
            if depth.value < 0.0 {
                return Err(ParseError::expected("a non-negative perspective depth"));
            }
            Ok(TransformFunction::Perspective(Some(depth)))
        },
        ("matrix3d", values) if values.len() == 16 => {
            let mut cells = [0.0_f32; 16];
            for (cell, value) in cells.iter_mut().zip(values) {
                *cell = number(value)?;
            }
            Ok(TransformFunction::Matrix3D(Matrix3D(cells)))
        },
        _ => Err(ParseError::expected(
            "a CSS Transforms Level 2 transform function",
        )),
    }
}

/// `parse_angle` plus the `calc()` fallback the individual properties use.
fn angle_value(input: &str) -> Result<f32, ParseError> {
    parse_angle(input).or_else(|error| crate::values::calc::parse_angle(input).map_err(|_| error))
}

fn parse_angle(input: &str) -> Result<f32, ParseError> {
    let lower = input.trim().to_ascii_lowercase();
    let (number, factor) = if let Some(value) = lower.strip_suffix("deg") {
        (value, std::f32::consts::PI / 180.0)
    } else if let Some(value) = lower.strip_suffix("grad") {
        (value, std::f32::consts::PI / 200.0)
    } else if let Some(value) = lower.strip_suffix("rad") {
        (value, 1.0)
    } else if let Some(value) = lower.strip_suffix("turn") {
        (value, std::f32::consts::TAU)
    } else if lower == "0" || lower == "+0" || lower == "-0" {
        ("0", 1.0)
    } else {
        return Err(ParseError::expected("a deg, rad, or turn angle"));
    };
    number
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value * factor)
        .ok_or_else(|| ParseError::expected("a finite angle"))
}

impl fmt::Display for Transform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Functions(functions) => {
                for (index, function) in functions.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(" ")?;
                    }
                    function.fmt(formatter)?;
                }
                Ok(())
            },
        }
    }
}

impl fmt::Display for TransformFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Translate(x, y) => write!(formatter, "translate({x}, {y})"),
            Self::Scale(x, y) => write!(
                formatter,
                "scale({}, {})",
                format_number(*x),
                format_number(*y)
            ),
            Self::Rotate(radians) => write!(formatter, "rotate({}rad)", format_number(*radians)),
            Self::Skew(x, y) => write!(
                formatter,
                "skew({}rad, {}rad)",
                format_number(*x),
                format_number(*y)
            ),
            Self::Matrix(matrix) => matrix.fmt(formatter),
            Self::Translate3D(x, y, z) => write!(formatter, "translate3d({x}, {y}, {z})"),
            Self::TranslateZ(z) => write!(formatter, "translateZ({z})"),
            Self::Scale3D(x, y, z) => write!(
                formatter,
                "scale3d({}, {}, {})",
                format_number(*x),
                format_number(*y),
                format_number(*z)
            ),
            Self::ScaleZ(z) => write!(formatter, "scaleZ({})", format_number(*z)),
            Self::RotateX(radians) => write!(formatter, "rotateX({}rad)", format_number(*radians)),
            Self::RotateY(radians) => write!(formatter, "rotateY({}rad)", format_number(*radians)),
            Self::RotateZ(radians) => write!(formatter, "rotateZ({}rad)", format_number(*radians)),
            Self::Rotate3D(x, y, z, radians) => write!(
                formatter,
                "rotate3d({}, {}, {}, {}rad)",
                format_number(*x),
                format_number(*y),
                format_number(*z),
                format_number(*radians)
            ),
            Self::Perspective(None) => formatter.write_str("perspective(none)"),
            Self::Perspective(Some(depth)) => write!(formatter, "perspective({depth})"),
            Self::Matrix3D(matrix) => matrix.fmt(formatter),
        }
    }
}
