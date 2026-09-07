use crate::Vec2;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Rem, Sub, SubAssign};

/// A 2D vector of `i32`, used for tile coordinates, pixel positions and grid
/// indices — anywhere a fractional value would be meaningless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub const ZERO: Self = Self::new(0, 0);
    pub const ONE: Self = Self::new(1, 1);
    pub const X: Self = Self::new(1, 0);
    pub const Y: Self = Self::new(0, 1);

    pub const UP: Self = Self::new(0, -1);
    pub const DOWN: Self = Self::new(0, 1);
    pub const LEFT: Self = Self::new(-1, 0);
    pub const RIGHT: Self = Self::new(1, 0);

    /// The four edge-sharing neighbours, clockwise from up.
    pub const CARDINAL: [Self; 4] = [Self::UP, Self::RIGHT, Self::DOWN, Self::LEFT];

    /// The eight surrounding cells, clockwise from up. The order matches the
    /// bit order used by autotiling, so index `i` is bit `i`.
    pub const NEIGHBOURS: [Self; 8] = [
        Self::new(0, -1),
        Self::new(1, -1),
        Self::new(1, 0),
        Self::new(1, 1),
        Self::new(0, 1),
        Self::new(-1, 1),
        Self::new(-1, 0),
        Self::new(-1, -1),
    ];

    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    #[must_use]
    pub const fn splat(v: i32) -> Self {
        Self::new(v, v)
    }

    /// Truncates towards zero. Use [`IVec2::from_vec2_floor`] for world-to-tile
    /// conversion, where -0.5 must land in tile -1 rather than tile 0.
    #[inline]
    #[must_use]
    pub fn from_vec2(v: Vec2) -> Self {
        Self::new(v.x as i32, v.y as i32)
    }

    /// Rounds towards negative infinity, which is what tile lookups need: the
    /// grid must not fold the two cells either side of the origin together.
    #[inline]
    #[must_use]
    pub fn from_vec2_floor(v: Vec2) -> Self {
        Self::new(v.x.floor() as i32, v.y.floor() as i32)
    }

    #[inline]
    #[must_use]
    pub fn as_vec2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// Manhattan distance — the number of cardinal steps between two cells.
    #[inline]
    #[must_use]
    pub fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Chebyshev distance — the number of steps when diagonals are allowed.
    #[inline]
    #[must_use]
    pub fn chebyshev_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    #[inline]
    #[must_use]
    pub fn length_squared(self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    /// Euclidean division, which floors instead of truncating. Chunk lookups
    /// need this: cell -1 with a chunk size of 16 belongs to chunk -1, not 0.
    #[inline]
    #[must_use]
    pub fn div_euclid(self, rhs: Self) -> Self {
        Self::new(self.x.div_euclid(rhs.x), self.y.div_euclid(rhs.y))
    }

    /// Euclidean remainder, always non-negative for a positive divisor. Pairs
    /// with [`IVec2::div_euclid`] to give a cell's offset within its chunk.
    #[inline]
    #[must_use]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        Self::new(self.x.rem_euclid(rhs.x), self.y.rem_euclid(rhs.y))
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

    /// The signum of each component: -1, 0 or 1.
    #[inline]
    #[must_use]
    pub fn signum(self) -> Self {
        Self::new(self.x.signum(), self.y.signum())
    }

    #[inline]
    #[must_use]
    pub const fn to_array(self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl Add for IVec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for IVec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul for IVec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<i32> for IVec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: i32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<i32> for IVec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: i32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Rem<i32> for IVec2 {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: i32) -> Self {
        Self::new(self.x % rhs, self.y % rhs)
    }
}

impl Neg for IVec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for IVec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for IVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<i32> for IVec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}

impl From<(i32, i32)> for IVec2 {
    #[inline]
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[i32; 2]> for IVec2 {
    #[inline]
    fn from([x, y]: [i32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl fmt::Display for IVec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
