use crate::Vec2;
use std::fmt;
use std::ops::Mul;

/// A 3x3 matrix representing a 2D affine transform.
///
/// Stored column-major to match what GPU APIs expect, so uploading one is a
/// straight copy. The last row is always `[0, 0, 1]` and is never stored.
///
/// Most code should reach for [`Transform2D`](crate::Transform2D), which keeps
/// translation, rotation and scale separate and readable. This type is the
/// composed form: what you get after multiplying transforms together, and what
/// the renderer hands to the GPU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    /// Column 0: the image of the X axis.
    pub x_axis: Vec2,
    /// Column 1: the image of the Y axis.
    pub y_axis: Vec2,
    /// Column 2: the translation.
    pub translation: Vec2,
}

impl Mat3 {
    pub const IDENTITY: Self = Self {
        x_axis: Vec2::X,
        y_axis: Vec2::Y,
        translation: Vec2::ZERO,
    };

    #[inline]
    #[must_use]
    pub const fn new(x_axis: Vec2, y_axis: Vec2, translation: Vec2) -> Self {
        Self {
            x_axis,
            y_axis,
            translation,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_translation(translation: Vec2) -> Self {
        Self {
            x_axis: Vec2::X,
            y_axis: Vec2::Y,
            translation,
        }
    }

    /// Clockwise by `radians`, matching the screen-space convention where Y
    /// grows downwards.
    #[inline]
    #[must_use]
    pub fn from_rotation(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            x_axis: Vec2::new(cos, sin),
            y_axis: Vec2::new(-sin, cos),
            translation: Vec2::ZERO,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_scale(scale: Vec2) -> Self {
        Self {
            x_axis: Vec2::new(scale.x, 0.0),
            y_axis: Vec2::new(0.0, scale.y),
            translation: Vec2::ZERO,
        }
    }

    /// Scale, then rotation, then translation — the order that behaves the way
    /// people expect when they set all three on an object.
    #[inline]
    #[must_use]
    pub fn from_scale_rotation_translation(scale: Vec2, radians: f32, translation: Vec2) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            x_axis: Vec2::new(cos * scale.x, sin * scale.x),
            y_axis: Vec2::new(-sin * scale.y, cos * scale.y),
            translation,
        }
    }

    /// Applies the full transform, translation included. Use this for points.
    #[inline]
    #[must_use]
    pub fn transform_point(self, point: Vec2) -> Vec2 {
        self.x_axis * point.x + self.y_axis * point.y + self.translation
    }

    /// Applies the linear part only, skipping translation. Use this for
    /// directions and velocities, which have no position to move.
    #[inline]
    #[must_use]
    pub fn transform_vector(self, vector: Vec2) -> Vec2 {
        self.x_axis * vector.x + self.y_axis * vector.y
    }

    /// The signed area scale factor. Negative when the transform flips
    /// handedness, zero when it collapses onto a line.
    #[inline]
    #[must_use]
    pub fn determinant(self) -> f32 {
        self.x_axis.x * self.y_axis.y - self.y_axis.x * self.x_axis.y
    }

    /// The inverse transform, or `None` when this one is degenerate — a zero
    /// scale on either axis collapses the plane and cannot be undone.
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let x_axis = Vec2::new(self.y_axis.y * inv_det, -self.x_axis.y * inv_det);
        let y_axis = Vec2::new(-self.y_axis.x * inv_det, self.x_axis.x * inv_det);

        Some(Self {
            x_axis,
            y_axis,
            translation: -(x_axis * self.translation.x + y_axis * self.translation.y),
        })
    }

    /// Column-major with the implied last row filled in, ready to upload as a
    /// `mat3x3<f32>`. Each column is padded to 4 floats because that is how GPU
    /// APIs align matrix columns.
    #[must_use]
    pub fn to_cols_array_padded(self) -> [f32; 12] {
        [
            self.x_axis.x,
            self.x_axis.y,
            0.0,
            0.0,
            self.y_axis.x,
            self.y_axis.y,
            0.0,
            0.0,
            self.translation.x,
            self.translation.y,
            1.0,
            0.0,
        ]
    }

    #[inline]
    #[must_use]
    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        self.x_axis.approx_eq(other.x_axis, epsilon)
            && self.y_axis.approx_eq(other.y_axis, epsilon)
            && self.translation.approx_eq(other.translation, epsilon)
    }
}

impl Default for Mat3 {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Composition. `a * b` applies `b` first, then `a` — the usual convention, so
/// `parent * child` places a child in its parent's space.
impl Mul for Mat3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            x_axis: self.transform_vector(rhs.x_axis),
            y_axis: self.transform_vector(rhs.y_axis),
            translation: self.transform_point(rhs.translation),
        }
    }
}

impl Mul<Vec2> for Mat3 {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        self.transform_point(rhs)
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {} {}]", self.x_axis, self.y_axis, self.translation)
    }
}
