use raster_math::{Rect, Vec2};

/// A moving box that collides with tiles.
#[derive(Debug, Clone, Copy)]
pub struct Body {
    /// The collision box, in world space.
    pub bounds: Rect,
    pub velocity: Vec2,
    /// Whether the body passes through one-way platforms from above.
    pub drop_through: bool,
}

impl Body {
    #[must_use]
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            velocity: Vec2::ZERO,
            drop_through: false,
        }
    }

    #[must_use]
    pub fn position(&self) -> Vec2 {
        self.bounds.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.bounds.position = position;
    }
}

/// What a body ran into while moving.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Contacts {
    pub left: bool,
    pub right: bool,
    pub above: bool,
    pub below: bool,
}

impl Contacts {
    /// Sur le sol : ce que le coyote time et le saut interrogent.
    #[must_use]
    pub fn grounded(self) -> bool {
        self.below
    }

    #[must_use]
    pub fn any(self) -> bool {
        self.left || self.right || self.above || self.below
    }

    /// Contre un mur, sans etre au sol.
    #[must_use]
    pub fn on_wall(self) -> bool {
        (self.left || self.right) && !self.below
    }
}
