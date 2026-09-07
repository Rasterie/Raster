use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A 2D vector of `f32`, used for positions, directions, sizes and velocities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);
    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);

    /// Screen space puts Y downwards, so `UP` is negative on that axis.
    pub const UP: Self = Self::new(0.0, -1.0);
    pub const DOWN: Self = Self::new(0.0, 1.0);
    pub const LEFT: Self = Self::new(-1.0, 0.0);
    pub const RIGHT: Self = Self::new(1.0, 0.0);

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// A vector with both components set to `v`.
    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self::new(v, v)
    }

    /// The unit vector at `radians`, measured clockwise from `X` because Y
    /// grows downwards.
    #[inline]
    #[must_use]
    pub fn from_angle(radians: f32) -> Self {
        Self::new(radians.cos(), radians.sin())
    }

    #[inline]
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// The z component of the 3D cross product, positive when `other` turns
    /// clockwise from `self` in screen space.
    #[inline]
    #[must_use]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Prefer this to [`Vec2::length`] when comparing distances: it skips the
    /// square root.
    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (other - self).length()
    }

    #[inline]
    #[must_use]
    pub fn distance_squared(self, other: Self) -> f32 {
        (other - self).length_squared()
    }

    /// The unit vector in the same direction, or [`Vec2::ZERO`] if this vector
    /// is too short to have a reliable direction.
    ///
    /// Returning zero rather than `NaN` means a degenerate case propagates as a
    /// visible standstill instead of poisoning every later computation.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > f32::EPSILON {
            self / len
        } else {
            Self::ZERO
        }
    }

    /// Whether this vector is within `f32::EPSILON` of unit length.
    #[inline]
    #[must_use]
    pub fn is_normalized(self) -> bool {
        (self.length_squared() - 1.0).abs() < f32::EPSILON
    }

    /// This vector shortened to `max` if it is longer, unchanged otherwise.
    #[inline]
    #[must_use]
    pub fn clamp_length(self, max: f32) -> Self {
        let len_sq = self.length_squared();
        if len_sq > max * max {
            self.normalized() * max
        } else {
            self
        }
    }

    /// Rotated clockwise by `radians`, matching the screen-space convention.
    #[inline]
    #[must_use]
    pub fn rotated(self, radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }

    /// Rotated a quarter turn clockwise. Exact, unlike `rotated(FRAC_PI_2)`.
    #[inline]
    #[must_use]
    pub fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// The angle from `X`, in `(-π, π]`.
    #[inline]
    #[must_use]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// The signed angle from `self` to `other`, in `(-π, π]`.
    #[inline]
    #[must_use]
    pub fn angle_to(self, other: Self) -> f32 {
        self.cross(other).atan2(self.dot(other))
    }

    #[inline]
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// This vector projected onto `other`, or zero if `other` is degenerate.
    #[inline]
    #[must_use]
    pub fn project_onto(self, other: Self) -> Self {
        let len_sq = other.length_squared();
        if len_sq > f32::EPSILON {
            other * (self.dot(other) / len_sq)
        } else {
            Self::ZERO
        }
    }

    /// This vector reflected across the surface whose normal is `normal`.
    ///
    /// `normal` is expected to be unit length; the result is meaningless
    /// otherwise.
    #[inline]
    #[must_use]
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    #[inline]
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline]
    #[must_use]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    #[inline]
    #[must_use]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    #[must_use]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    #[inline]
    #[must_use]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    /// Rounded half away from zero, unlike `f32::round_ties_even`.
    #[inline]
    #[must_use]
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    /// Snapped to whole pixels. The renderer applies this to sprite positions
    /// so pixel art lands on the grid; cameras deliberately skip it to keep
    /// scrolling smooth.
    #[inline]
    #[must_use]
    pub fn snap(self) -> Self {
        self.round()
    }

    #[inline]
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    #[must_use]
    pub fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    /// Whether both components are within `epsilon` of `other`'s.
    #[inline]
    #[must_use]
    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }

    #[inline]
    #[must_use]
    pub const fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// Component-wise, so a `Vec2` doubles as a non-uniform scale.
impl Mul for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

/// Lets `2.0 * v` read as naturally as `v * 2.0`.
impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        rhs * self
    }
}

impl Div for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
