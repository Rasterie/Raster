use raster_math::{Rect, Vec2};

/// Which way a container stacks its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    #[default]
    Vertical,
    Horizontal,
}

/// How much room a child asks for along the stacking axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// Une taille fixe, en pixels.
    Fixed(f32),
    /// Une part de ce qui reste, une fois les tailles fixes retirees.
    Grow(f32),
}

impl Size {
    #[must_use]
    pub fn fixed(&self) -> f32 {
        match self {
            Self::Fixed(v) => v.max(0.0),
            Self::Grow(_) => 0.0,
        }
    }

    #[must_use]
    pub fn weight(&self) -> f32 {
        match self {
            Self::Grow(w) => w.max(0.0),
            Self::Fixed(_) => 0.0,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::Grow(1.0)
    }
}

/// Splits a rectangle among children, along one axis.
///
/// Les tailles fixes sont servies d'abord, le reste se partage au prorata des
/// poids : c'est ce qui rend une barre d'outils stable quand la fenetre change.
#[must_use]
pub fn stack(area: Rect, axis: Axis, gap: f32, sizes: &[Size]) -> Vec<Rect> {
    if sizes.is_empty() {
        return Vec::new();
    }

    let total = match axis {
        Axis::Vertical => area.size.y,
        Axis::Horizontal => area.size.x,
    };

    let gaps = gap * (sizes.len() - 1) as f32;
    let fixed: f32 = sizes.iter().map(Size::fixed).sum();
    let weights: f32 = sizes.iter().map(Size::weight).sum();
    let free = (total - gaps - fixed).max(0.0);

    let mut out = Vec::with_capacity(sizes.len());
    let mut cursor = 0.0;

    for size in sizes {
        let length = match size {
            Size::Fixed(v) => v.max(0.0),
            // Sans poids, un `Grow` seul prend tout ce qui reste.
            Size::Grow(w) if weights > 0.0 => free * w.max(0.0) / weights,
            Size::Grow(_) => 0.0,
        };

        out.push(match axis {
            Axis::Vertical => Rect::new(
                area.position.x,
                area.position.y + cursor,
                area.size.x,
                length,
            ),
            Axis::Horizontal => Rect::new(
                area.position.x + cursor,
                area.position.y,
                length,
                area.size.y,
            ),
        });

        cursor += length + gap;
    }

    out
}

/// Shrinks a rectangle by the same amount on every side.
#[must_use]
pub fn inset(area: Rect, by: f32) -> Rect {
    let by = by.min(area.size.x / 2.0).min(area.size.y / 2.0).max(0.0);
    Rect::new(
        area.position.x + by,
        area.position.y + by,
        area.size.x - by * 2.0,
        area.size.y - by * 2.0,
    )
}

/// Where a HUD element sticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCentre,
    TopRight,
    CentreLeft,
    Centre,
    CentreRight,
    BottomLeft,
    BottomCentre,
    BottomRight,
}

impl Anchor {
    /// Le point de rattachement, en fractions de la zone.
    #[must_use]
    pub fn fractions(self) -> Vec2 {
        let x = match self {
            Self::TopLeft | Self::CentreLeft | Self::BottomLeft => 0.0,
            Self::TopCentre | Self::Centre | Self::BottomCentre => 0.5,
            _ => 1.0,
        };
        let y = match self {
            Self::TopLeft | Self::TopCentre | Self::TopRight => 0.0,
            Self::CentreLeft | Self::Centre | Self::CentreRight => 0.5,
            _ => 1.0,
        };
        Vec2::new(x, y)
    }

    /// Places a box of `size` in `area`, offset inwards by `margin`.
    ///
    /// La marge pousse toujours vers l'interieur : un element ancre en haut a
    /// droite s'ecarte du coin, quel que soit le coin.
    #[must_use]
    pub fn place(self, area: Rect, size: Vec2, margin: Vec2) -> Rect {
        let f = self.fractions();
        let x = area.position.x + (area.size.x - size.x) * f.x + margin.x * (1.0 - 2.0 * f.x);
        let y = area.position.y + (area.size.y - size.y) * f.y + margin.y * (1.0 - 2.0 * f.y);
        Rect::new(x, y, size.x, size.y)
    }
}

/// Centres a box inside an area.
#[must_use]
pub fn centre(area: Rect, size: Vec2) -> Rect {
    Anchor::Centre.place(area, size, Vec2::ZERO)
}
