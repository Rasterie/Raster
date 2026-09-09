use crate::IVec2;
use std::fmt;

/// An integer rectangle, for tile regions, texture sub-images and pixel areas.
///
/// Like [`Rect`](crate::Rect), the edges are half-open: `min` is included and
/// `max` excluded, so a rectangle of size `(2, 2)` covers exactly four cells
/// and two adjacent rectangles never share one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IRect {
    pub position: IVec2,
    pub size: IVec2,
}

impl IRect {
    pub const ZERO: Self = Self {
        position: IVec2::ZERO,
        size: IVec2::ZERO,
    };

    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            position: IVec2::new(x, y),
            size: IVec2::new(width, height),
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_position_size(position: IVec2, size: IVec2) -> Self {
        Self { position, size }
    }

    /// From two opposite corners, in any order. The upper corner is exclusive.
    #[inline]
    #[must_use]
    pub fn from_corners(a: IVec2, b: IVec2) -> Self {
        let min = a.min(b);
        Self {
            position: min,
            size: a.max(b) - min,
        }
    }

    #[inline]
    #[must_use]
    pub fn min(self) -> IVec2 {
        self.position
    }

    /// The exclusive upper corner: one past the last cell on each axis.
    #[inline]
    #[must_use]
    pub fn max(self) -> IVec2 {
        self.position + self.size
    }

    #[inline]
    #[must_use]
    pub fn left(self) -> i32 {
        self.position.x
    }

    #[inline]
    #[must_use]
    pub fn right(self) -> i32 {
        self.position.x + self.size.x
    }

    #[inline]
    #[must_use]
    pub fn top(self) -> i32 {
        self.position.y
    }

    #[inline]
    #[must_use]
    pub fn bottom(self) -> i32 {
        self.position.y + self.size.y
    }

    /// The number of cells covered.
    #[inline]
    #[must_use]
    pub fn area(self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.size.x * self.size.y
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.size.x <= 0 || self.size.y <= 0
    }

    #[inline]
    #[must_use]
    pub fn contains(self, point: IVec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }

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

    /// The smallest integer rectangle containing a float one.
    ///
    /// Englobe plutot que tronque : une decoupe qui rogne un demi-pixel
    /// mangerait le bord de ce qu'elle devait laisser passer.
    #[inline]
    #[must_use]
    pub fn from_rect(rect: crate::Rect) -> Self {
        let left = rect.position.x.floor();
        let top = rect.position.y.floor();
        let right = (rect.position.x + rect.size.x).ceil();
        let bottom = (rect.position.y + rect.size.y).ceil();

        Self {
            position: IVec2::new(left as i32, top as i32),
            size: IVec2::new(
                (right - left).max(0.0) as i32,
                (bottom - top).max(0.0) as i32,
            ),
        }
    }

    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let min = self.min().max(other.min());
        let max = self.max().min(other.max());
        let rect = Self::from_position_size(min, max - min);
        if rect.is_empty() { None } else { Some(rect) }
    }

    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self::from_corners(self.min().min(other.min()), self.max().max(other.max()))
    }

    #[inline]
    #[must_use]
    pub fn translated(self, offset: IVec2) -> Self {
        Self::from_position_size(self.position + offset, self.size)
    }

    #[inline]
    #[must_use]
    pub fn expanded(self, amount: i32) -> Self {
        Self::from_position_size(
            self.position - IVec2::splat(amount),
            self.size + IVec2::splat(amount * 2),
        )
    }

    /// Every cell in the rectangle, row by row. Empty when the rectangle is.
    ///
    /// This is the iteration order chunk and tilemap code uses, so that reads
    /// walk memory forwards rather than jumping between rows.
    pub fn cells(self) -> impl Iterator<Item = IVec2> {
        let (left, top) = (self.left(), self.top());
        let (width, height) = (self.size.x.max(0), self.size.y.max(0));

        (0..height).flat_map(move |y| (0..width).map(move |x| IVec2::new(left + x, top + y)))
    }
}

impl fmt::Display for IRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}x{}]", self.position, self.size.x, self.size.y)
    }
}
