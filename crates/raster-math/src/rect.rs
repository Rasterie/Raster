use crate::Vec2;
use std::fmt;

/// An axis-aligned rectangle, stored as a position and a size.
///
/// This is the collision primitive of the engine: bodies are AABBs, and
/// rotation stays visual so that tile collision remains a grid lookup.
///
/// Y grows downwards, so `min` is the top-left corner and `max` the
/// bottom-right.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// The top-left corner.
    pub position: Vec2,
    /// Width and height. Negative components make the rectangle invalid; see
    /// [`Rect::normalized`].
    pub size: Vec2,
}

impl Rect {
    pub const ZERO: Self = Self {
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    /// From two opposite corners, in any order.
    #[inline]
    #[must_use]
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        let min = a.min(b);
        Self {
            position: min,
            size: (a.max(b)) - min,
        }
    }

    /// Centred on `center`, which is how colliders are usually specified.
    #[inline]
    #[must_use]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self {
            position: center - size * 0.5,
            size,
        }
    }

    #[inline]
    #[must_use]
    pub fn min(self) -> Vec2 {
        self.position
    }

    #[inline]
    #[must_use]
    pub fn max(self) -> Vec2 {
        self.position + self.size
    }

    #[inline]
    #[must_use]
    pub fn center(self) -> Vec2 {
        self.position + self.size * 0.5
    }

    #[inline]
    #[must_use]
    pub fn width(self) -> f32 {
        self.size.x
    }

    #[inline]
    #[must_use]
    pub fn height(self) -> f32 {
        self.size.y
    }

    #[inline]
    #[must_use]
    pub fn left(self) -> f32 {
        self.position.x
    }

    #[inline]
    #[must_use]
    pub fn right(self) -> f32 {
        self.position.x + self.size.x
    }

    #[inline]
    #[must_use]
    pub fn top(self) -> f32 {
        self.position.y
    }

    #[inline]
    #[must_use]
    pub fn bottom(self) -> f32 {
        self.position.y + self.size.y
    }

    #[inline]
    #[must_use]
    pub fn top_left(self) -> Vec2 {
        self.position
    }

    #[inline]
    #[must_use]
    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.right(), self.top())
    }

    #[inline]
    #[must_use]
    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.left(), self.bottom())
    }

    #[inline]
    #[must_use]
    pub fn bottom_right(self) -> Vec2 {
        self.max()
    }

    /// The four corners, clockwise from the top-left.
    #[inline]
    #[must_use]
    pub fn corners(self) -> [Vec2; 4] {
        [
            self.top_left(),
            self.top_right(),
            self.bottom_right(),
            self.bottom_left(),
        ]
    }

    #[inline]
    #[must_use]
    pub fn area(self) -> f32 {
        self.size.x * self.size.y
    }

    /// Whether either dimension is zero or negative. An empty rectangle
    /// intersects nothing and contains nothing.
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.size.x <= 0.0 || self.size.y <= 0.0
    }

    /// The same rectangle with a non-negative size, moving the position when a
    /// dimension was negative.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        Self::from_corners(self.position, self.position + self.size)
    }

    /// Whether `point` lies inside. The top and left edges are inclusive, the
    /// bottom and right exclusive, so tiling rectangles never both claim a
    /// point on their shared edge.
    #[inline]
    #[must_use]
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }

    /// Whether `other` lies entirely inside this rectangle. An empty rectangle
    /// contains nothing, including another empty rectangle.
    #[inline]
    #[must_use]
    pub fn contains_rect(self, other: Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        other.left() >= self.left()
            && other.right() <= self.right()
            && other.top() >= self.top()
            && other.bottom() <= self.bottom()
    }

    /// Whether the two rectangles overlap on both axes.
    ///
    /// Touching edges do not count as an overlap, so a body resting exactly on
    /// the ground is not reported as intersecting it every frame. An empty
    /// rectangle has no area and so intersects nothing, even when its position
    /// falls inside the other.
    #[inline]
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// The overlapping region, or `None` when they do not overlap.
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let min = self.min().max(other.min());
        let max = self.max().min(other.max());
        let rect = Self::from_position_size(min, max - min);
        if rect.is_empty() { None } else { Some(rect) }
    }

    /// The smallest rectangle containing both.
    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self::from_corners(self.min().min(other.min()), self.max().max(other.max()))
    }

    /// Grown by `amount` on every side. A negative amount shrinks it, possibly
    /// to an empty rectangle.
    #[inline]
    #[must_use]
    pub fn expanded(self, amount: f32) -> Self {
        self.expanded_by(Vec2::splat(amount))
    }

    /// Grown by `amount.x` horizontally and `amount.y` vertically, on each side.
    #[inline]
    #[must_use]
    pub fn expanded_by(self, amount: Vec2) -> Self {
        Self::from_position_size(self.position - amount, self.size + amount * 2.0)
    }

    /// Moved by `offset`, keeping its size.
    #[inline]
    #[must_use]
    pub fn translated(self, offset: Vec2) -> Self {
        Self::from_position_size(self.position + offset, self.size)
    }

    /// The point inside this rectangle closest to `point`.
    #[inline]
    #[must_use]
    pub fn closest_point(self, point: Vec2) -> Vec2 {
        point.clamp(self.min(), self.max())
    }

    /// The rectangle grown to whole pixels: the position floors and the far
    /// edge ceils, so nothing inside is ever cropped.
    #[inline]
    #[must_use]
    pub fn snap_outward(self) -> Self {
        Self::from_corners(self.min().floor(), self.max().ceil())
    }

    #[inline]
    #[must_use]
    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        self.position.approx_eq(other.position, epsilon) && self.size.approx_eq(other.size, epsilon)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}x{}]", self.position, self.size.x, self.size.y)
    }
}
