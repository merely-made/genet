/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bounded 2D transform matrices for parsing, resolved-value serialization,
//! paint, and mismatched-list interpolation.
//!
//! Harvested and reshaped from `style/values/animated/transform.rs` in the
//! mark-ik Stylo fork at `b157d925267fdd37b03f43e3387ab2f0909e57b0`.
//! Livery keeps only the CSS Transforms Level 1 2D decomposition path.

use super::format_number;
use super::property::{Transform, TransformFunction};
use super::{Length, LengthPercentage};

/// A CSS 2D affine matrix in `matrix(a, b, c, d, e, f)` order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix2D {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn is_finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .into_iter()
            .all(f32::is_finite)
    }

    /// Matrix multiplication in CSS's column-vector convention.
    pub fn multiply(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Compose one transform-list suffix into its equivalent 2D matrix.
    /// A list that needs a Z axis deliberately has none.
    pub fn from_functions(
        functions: &[TransformFunction],
        em: f32,
        reference_box: (f32, f32),
    ) -> Option<Self> {
        Matrix3D::from_functions(functions, em, reference_box)?.to_matrix_2d()
    }

    /// Compose a transform list only when it has no percentage dependency.
    /// Interpolation has no reference box, so it must reject rather than guess.
    pub(crate) fn from_absolute_functions(
        functions: &[TransformFunction],
        em: f32,
    ) -> Option<Self> {
        if has_percentage(functions) {
            return None;
        }
        Self::from_functions(functions, em, (0.0, 0.0))
    }

    /// Interpolate through the CSS 2D matrix decomposition algorithm.
    pub fn interpolate(self, other: Self, progress: f32) -> Option<Self> {
        if !self.is_finite() || !other.is_finite() {
            return None;
        }
        let from = Decomposed2D::from(self);
        let to = Decomposed2D::from(other);
        Some(from.interpolate(to, progress.clamp(0.0, 1.0)).into())
    }
}

impl std::fmt::Display for Matrix2D {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "matrix({}, {}, {}, {}, {}, {})",
            format_number(self.a),
            format_number(self.b),
            format_number(self.c),
            format_number(self.d),
            format_number(self.e),
            format_number(self.f)
        )
    }
}

impl Transform {
    /// Resolve this transform list to a 2D matrix. `none` deliberately has no
    /// matrix so callers can retain its computed serialization, and a list
    /// that needs a Z axis has none either.
    pub fn to_matrix(&self, em: f32, reference_box: (f32, f32)) -> Option<Matrix2D> {
        self.to_matrix_3d(em, reference_box)?.to_matrix_2d()
    }

    /// Resolve this transform list to its full 4x4. This is the lowering
    /// input: paint projects it, it is not projected here.
    pub fn to_matrix_3d(&self, em: f32, reference_box: (f32, f32)) -> Option<Matrix3D> {
        match self {
            Self::None => None,
            Self::Functions(functions) => Matrix3D::from_functions(functions, em, reference_box),
        }
    }

    /// CSSOM resolved-value serialization. `Matrix3D`'s own `Display` prints
    /// `matrix()` when the composition stayed in the plane.
    pub fn to_computed_css(&self, em: f32, reference_box: Option<(f32, f32)>) -> String {
        let matrix = match reference_box {
            Some(reference_box) => self.to_matrix_3d(em, reference_box),
            None => match self {
                Self::Functions(functions) if !has_percentage(functions) => {
                    Matrix3D::from_functions(functions, em, (0.0, 0.0))
                },
                _ => None,
            },
        };
        matrix.map_or_else(|| self.to_string(), |matrix| matrix.to_string())
    }

    /// Resolve relative translation lengths after font-size computation, so
    /// retained transition interpolation operates on computed px values.
    pub fn resolve_lengths(&mut self, em: f32, rem: f32) {
        let Self::Functions(functions) = self else {
            return;
        };
        for function in functions {
            match function {
                TransformFunction::Translate(x, y) => {
                    *x = x.resolve_font_relative(em, rem);
                    *y = y.resolve_font_relative(em, rem);
                },
                TransformFunction::Translate3D(x, y, z) => {
                    *x = x.resolve_font_relative(em, rem);
                    *y = y.resolve_font_relative(em, rem);
                    *z = resolve_font_relative_length(*z, em, rem);
                },
                TransformFunction::TranslateZ(z) => {
                    *z = resolve_font_relative_length(*z, em, rem);
                },
                TransformFunction::Perspective(Some(depth)) => {
                    *depth = resolve_font_relative_length(*depth, em, rem);
                },
                _ => {},
            }
        }
    }
}

pub(crate) fn has_percentage(functions: &[TransformFunction]) -> bool {
    functions.iter().any(|function| match function {
        TransformFunction::Translate(x, y) | TransformFunction::Translate3D(x, y, _) => {
            x.has_percentage() || y.has_percentage()
        },
        _ => false,
    })
}

fn resolve_font_relative_length(length: Length, em: f32, rem: f32) -> Length {
    match LengthPercentage::Length(length).resolve_font_relative(em, rem) {
        LengthPercentage::Length(resolved) => resolved,
        _ => length,
    }
}

impl Matrix3D {
    /// Compose a transform list in CSS order: the first function is outermost.
    pub fn from_functions(
        functions: &[TransformFunction],
        em: f32,
        reference_box: (f32, f32),
    ) -> Option<Self> {
        let mut matrix = Self::IDENTITY;
        for function in functions {
            matrix = matrix.multiply(&function_matrix(*function, em, reference_box)?);
        }
        matrix.is_finite().then_some(matrix)
    }
}

/// A CSS 4x4 transform matrix, stored in `matrix3d()` argument order: the
/// column-major cells of the column-vector matrix CSS composes with. That is
/// also `euclid::Transform3D`'s row-major row-vector order, so lowering to
/// `LayoutTransform` is a field-for-field copy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3D(pub [f32; 16]);

impl Matrix3D {
    pub const IDENTITY: Self = Self([
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Cell at `row`, `column` of the column-vector matrix.
    pub const fn at(&self, row: usize, column: usize) -> f32 {
        self.0[column * 4 + row]
    }

    fn set(&mut self, row: usize, column: usize, value: f32) {
        self.0[column * 4 + row] = value;
    }

    pub fn is_finite(&self) -> bool {
        self.0.iter().copied().all(f32::is_finite)
    }

    /// `self` composed over `other`: `other` applies to the point first.
    pub fn multiply(&self, other: &Self) -> Self {
        let mut out = Self([0.0; 16]);
        for column in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.at(row, k) * other.at(k, column);
                }
                out.set(row, column, sum);
            }
        }
        out
    }

    pub const fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[12] = x;
        matrix.0[13] = y;
        matrix.0[14] = z;
        matrix
    }

    pub const fn scaling(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[0] = x;
        matrix.0[5] = y;
        matrix.0[10] = z;
        matrix
    }

    /// Rodrigues rotation about a normalized axis. CSS's Y axis points down,
    /// which is why this matches `rotate()` for the Z axis without a sign flip.
    pub fn rotation(x: f32, y: f32, z: f32, angle: f32) -> Option<Self> {
        let length = (x * x + y * y + z * z).sqrt();
        if !length.is_finite() || length == 0.0 {
            return None;
        }
        let (x, y, z) = (x / length, y / length, z / length);
        let (sin, cos) = angle.sin_cos();
        // A cardinal axis is written out directly. Rodrigues computes the
        // unrotated diagonal cell as `(1 - cos) + cos`, which is not exactly
        // 1 in f32 and would leave every `rotateZ()` looking like a 3D matrix.
        if let Some(axis) = [
            (x, y, z) == (1.0, 0.0, 0.0),
            (x, y, z) == (0.0, 1.0, 0.0),
            (x, y, z) == (0.0, 0.0, 1.0),
        ]
        .into_iter()
        .position(|matched| matched)
        {
            let mut matrix = Self::IDENTITY;
            let (u, v) = [(1, 2), (2, 0), (0, 1)][axis];
            matrix.set(u, u, cos);
            matrix.set(v, v, cos);
            matrix.set(u, v, -sin);
            matrix.set(v, u, sin);
            return matrix.is_finite().then_some(matrix);
        }
        let t = 1.0 - cos;
        let mut matrix = Self::IDENTITY;
        matrix.set(0, 0, t * x * x + cos);
        matrix.set(0, 1, t * x * y - sin * z);
        matrix.set(0, 2, t * x * z + sin * y);
        matrix.set(1, 0, t * x * y + sin * z);
        matrix.set(1, 1, t * y * y + cos);
        matrix.set(1, 2, t * y * z - sin * x);
        matrix.set(2, 0, t * x * z - sin * y);
        matrix.set(2, 1, t * y * z + sin * x);
        matrix.set(2, 2, t * z * z + cos);
        matrix.is_finite().then_some(matrix)
    }

    /// The `perspective(d)` matrix. A non-positive or non-finite depth is the
    /// spec's identity-free case and is rejected by the caller's parser.
    pub fn perspective(depth: f32) -> Option<Self> {
        (depth.is_finite() && depth > 0.0).then(|| {
            let mut matrix = Self::IDENTITY;
            matrix.set(3, 2, -1.0 / depth);
            matrix
        })
    }

    /// Whether the fourth row is the affine `0 0 0 1`, so the projection has
    /// no perspective divide for netrender's affine rasterizers to lose.
    pub fn is_affine(&self) -> bool {
        self.at(3, 0) == 0.0 && self.at(3, 1) == 0.0 && self.at(3, 2) == 0.0 && self.at(3, 3) == 1.0
    }

    /// Whether every cell outside the 2D affine six is at its identity value.
    pub fn is_2d(&self) -> bool {
        const PLANAR: [usize; 8] = [2, 3, 6, 7, 8, 9, 11, 14];
        PLANAR.into_iter().all(|index| self.0[index] == 0.0)
            && self.0[10] == 1.0
            && self.0[15] == 1.0
    }

    pub fn to_matrix_2d(&self) -> Option<Matrix2D> {
        self.is_2d().then(|| {
            Matrix2D::new(
                self.0[0], self.0[1], self.0[4], self.0[5], self.0[12], self.0[13],
            )
        })
    }

    /// The orthographic projection netrender's rasterizers actually read: the
    /// planar six cells, with Z dropped rather than divided.
    pub fn to_affine_2d(&self) -> Matrix2D {
        Matrix2D::new(
            self.0[0], self.0[1], self.0[4], self.0[5], self.0[12], self.0[13],
        )
    }

    /// The depth of the local origin in the accumulated space, which is what a
    /// `preserve-3d` context sorts its participating boxes by.
    pub const fn origin_depth(&self) -> f32 {
        self.0[14]
    }

    /// Positive when the front face points at the viewer. This is the Z of the
    /// cross product of the mapped local X and Y axes, which reduces to the
    /// determinant of the projected affine part.
    pub const fn front_facing_z(&self) -> f32 {
        self.at(0, 0) * self.at(1, 1) - self.at(1, 0) * self.at(0, 1)
    }
}

impl From<Matrix2D> for Matrix3D {
    fn from(matrix: Matrix2D) -> Self {
        Self([
            matrix.a, matrix.b, 0.0, 0.0, //
            matrix.c, matrix.d, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            matrix.e, matrix.f, 0.0, 1.0,
        ])
    }
}

impl std::fmt::Display for Matrix3D {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(matrix) = self.to_matrix_2d() {
            return matrix.fmt(formatter);
        }
        formatter.write_str("matrix3d(")?;
        for (index, value) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            formatter.write_str(&format_number(*value))?;
        }
        formatter.write_str(")")
    }
}

fn function_matrix(
    function: TransformFunction,
    em: f32,
    reference_box: (f32, f32),
) -> Option<Matrix3D> {
    let px = |length: Length| length.unit.to_px(length.value, em, 16.0);
    let matrix = match function {
        TransformFunction::Translate(x, y) => Matrix3D::translation(
            x.to_px(em, 16.0, reference_box.0),
            y.to_px(em, 16.0, reference_box.1),
            0.0,
        ),
        TransformFunction::Translate3D(x, y, z) => Matrix3D::translation(
            x.to_px(em, 16.0, reference_box.0),
            y.to_px(em, 16.0, reference_box.1),
            px(z),
        ),
        TransformFunction::TranslateZ(z) => Matrix3D::translation(0.0, 0.0, px(z)),
        TransformFunction::Scale(x, y) => Matrix3D::scaling(x, y, 1.0),
        TransformFunction::Scale3D(x, y, z) => Matrix3D::scaling(x, y, z),
        TransformFunction::ScaleZ(z) => Matrix3D::scaling(1.0, 1.0, z),
        TransformFunction::Rotate(angle) | TransformFunction::RotateZ(angle) => {
            Matrix3D::rotation(0.0, 0.0, 1.0, angle)?
        },
        TransformFunction::RotateX(angle) => Matrix3D::rotation(1.0, 0.0, 0.0, angle)?,
        TransformFunction::RotateY(angle) => Matrix3D::rotation(0.0, 1.0, 0.0, angle)?,
        TransformFunction::Rotate3D(x, y, z, angle) => Matrix3D::rotation(x, y, z, angle)?,
        TransformFunction::Skew(x, y) => Matrix2D::new(1.0, y.tan(), x.tan(), 1.0, 0.0, 0.0).into(),
        TransformFunction::Perspective(None) => Matrix3D::IDENTITY,
        TransformFunction::Perspective(Some(depth)) => {
            // A zero depth has no defined projection, so it stays the
            // identity rather than dividing by zero.
            Matrix3D::perspective(px(depth)).unwrap_or(Matrix3D::IDENTITY)
        },
        TransformFunction::Matrix(matrix) => matrix.into(),
        TransformFunction::Matrix3D(matrix) => matrix,
    };
    matrix.is_finite().then_some(matrix)
}

#[derive(Clone, Copy, Debug)]
struct Decomposed2D {
    translate: (f32, f32),
    scale: (f32, f32),
    angle: f32,
    inner: (f32, f32, f32, f32),
}

impl Decomposed2D {
    fn interpolate(self, other: Self, progress: f32) -> Self {
        let scalar = |from: f32, to: f32| from + (to - from) * progress;
        let mut scale = self.scale;
        let mut angle = self.angle;
        let mut other_angle = other.angle;

        // If opposite axes are flipped, express one side as an unflipped
        // rotation before choosing the shortest rotation arc.
        if (scale.0 < 0.0 && other.scale.1 < 0.0) || (scale.1 < 0.0 && other.scale.0 < 0.0) {
            scale.0 = -scale.0;
            scale.1 = -scale.1;
            angle += if angle < 0.0 {
                std::f32::consts::PI
            } else {
                -std::f32::consts::PI
            };
        }

        if angle == 0.0 {
            angle = std::f32::consts::TAU;
        }
        if other_angle == 0.0 {
            other_angle = std::f32::consts::TAU;
        }
        if (angle - other_angle).abs() > std::f32::consts::PI {
            if angle > other_angle {
                angle -= std::f32::consts::TAU;
            } else {
                other_angle -= std::f32::consts::TAU;
            }
        }

        Self {
            translate: (
                scalar(self.translate.0, other.translate.0),
                scalar(self.translate.1, other.translate.1),
            ),
            scale: (
                scalar(scale.0, other.scale.0),
                scalar(scale.1, other.scale.1),
            ),
            angle: scalar(angle, other_angle),
            inner: (
                scalar(self.inner.0, other.inner.0),
                scalar(self.inner.1, other.inner.1),
                scalar(self.inner.2, other.inner.2),
                scalar(self.inner.3, other.inner.3),
            ),
        }
    }
}

impl From<Matrix2D> for Decomposed2D {
    fn from(matrix: Matrix2D) -> Self {
        let mut row0x = matrix.a;
        let mut row0y = matrix.b;
        let mut row1x = matrix.c;
        let mut row1y = matrix.d;
        let mut scale = (row0x.hypot(row0y), row1x.hypot(row1y));

        if row0x * row1y - row0y * row1x < 0.0 {
            if row0x < row1y {
                scale.0 = -scale.0;
            } else {
                scale.1 = -scale.1;
            }
        }
        if scale.0 != 0.0 {
            row0x /= scale.0;
            row0y /= scale.0;
        }
        if scale.1 != 0.0 {
            row1x /= scale.1;
            row1y /= scale.1;
        }

        let angle = row0y.atan2(row0x);
        if angle != 0.0 {
            let sin = -row0y;
            let cos = row0x;
            let m11 = row0x;
            let m12 = row0y;
            let m21 = row1x;
            let m22 = row1y;
            row0x = cos * m11 + sin * m21;
            row0y = cos * m12 + sin * m22;
            row1x = -sin * m11 + cos * m21;
            row1y = -sin * m12 + cos * m22;
        }

        Self {
            translate: (matrix.e, matrix.f),
            scale,
            angle,
            inner: (row0x, row0y, row1x, row1y),
        }
    }
}

impl From<Decomposed2D> for Matrix2D {
    fn from(decomposed: Decomposed2D) -> Self {
        let mut matrix = Matrix2D::new(
            decomposed.inner.0,
            decomposed.inner.1,
            decomposed.inner.2,
            decomposed.inner.3,
            decomposed.translate.0,
            decomposed.translate.1,
        );
        let (sin, cos) = decomposed.angle.sin_cos();
        matrix = matrix.multiply(Matrix2D::new(cos, sin, -sin, cos, 0.0, 0.0));
        matrix.a *= decomposed.scale.0;
        matrix.b *= decomposed.scale.0;
        matrix.c *= decomposed.scale.1;
        matrix.d *= decomposed.scale.1;
        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: Matrix2D, expected: Matrix2D) {
        for (actual, expected) in [
            (actual.a, expected.a),
            (actual.b, expected.b),
            (actual.c, expected.c),
            (actual.d, expected.d),
            (actual.e, expected.e),
            (actual.f, expected.f),
        ] {
            assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
        }
    }

    #[test]
    fn decomposition_round_trips_affine_matrices() {
        let matrix = Matrix2D::new(1.2, 0.5, -0.25, 0.8, 12.0, -4.0);
        close(Decomposed2D::from(matrix).into(), matrix);
    }

    #[test]
    fn decomposition_interpolates_translation_scale_and_rotation() {
        let from = Matrix2D::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let to = Matrix2D::new(0.0, 2.0, -2.0, 0.0, 20.0, 4.0);
        let middle = from.interpolate(to, 0.5).expect("decomposed matrix");
        let root = 0.5_f32.sqrt() * 1.5;
        close(middle, Matrix2D::new(root, root, -root, root, 10.0, 2.0));
    }
}
