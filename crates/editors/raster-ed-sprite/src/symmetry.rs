use raster_math::IVec2;

/// How a stroke is mirrored as it is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Symmetry {
    #[default]
    None,
    /// Miroir gauche-droite.
    Horizontal,
    /// Miroir haut-bas.
    Vertical,
    /// Les deux : quatre points par coup de pinceau.
    Both,
    /// Miroir sur la diagonale, pour une forme carree.
    Diagonal,
}

impl Symmetry {
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Horizontal,
        Self::Vertical,
        Self::Both,
        Self::Diagonal,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Aucune",
            Self::Horizontal => "Horizontale",
            Self::Vertical => "Verticale",
            Self::Both => "Les deux",
            Self::Diagonal => "Diagonale",
        }
    }
}

/// Every point a stroke touches, given the symmetry.
///
/// Sans doublon : au centre d'un axe, le miroir tombe sur le point lui-meme,
/// et le peindre deux fois doublerait l'effet d'un pinceau translucide.
#[must_use]
pub fn mirror(at: IVec2, size: IVec2, symmetry: Symmetry) -> Vec<IVec2> {
    // L'axe passe par le milieu : sur une largeur paire il tombe entre deux
    // pixels, ce qui est le comportement attendu.
    let fx = size.x - 1 - at.x;
    let fy = size.y - 1 - at.y;

    let mut points = match symmetry {
        Symmetry::None => vec![at],
        Symmetry::Horizontal => vec![at, IVec2::new(fx, at.y)],
        Symmetry::Vertical => vec![at, IVec2::new(at.x, fy)],
        Symmetry::Both => vec![
            at,
            IVec2::new(fx, at.y),
            IVec2::new(at.x, fy),
            IVec2::new(fx, fy),
        ],
        // La diagonale n'a de sens que sur un carre : ailleurs, elle sortirait
        // du cadre.
        Symmetry::Diagonal if size.x == size.y => vec![at, IVec2::new(at.y, at.x)],
        Symmetry::Diagonal => vec![at],
    };

    points.sort_by_key(|p| (p.x, p.y));
    points.dedup();
    points
}

/// Where the symmetry axes sit, in pixels, for drawing a guide.
///
/// `None` quand l'axe n'existe pas. La valeur est la position du milieu, qui
/// peut tomber entre deux pixels sur une dimension paire.
#[must_use]
pub fn axes(size: IVec2, symmetry: Symmetry) -> (Option<f32>, Option<f32>) {
    let cx = (size.x as f32 - 1.0) / 2.0 + 0.5;
    let cy = (size.y as f32 - 1.0) / 2.0 + 0.5;

    match symmetry {
        Symmetry::None | Symmetry::Diagonal => (None, None),
        Symmetry::Horizontal => (Some(cx), None),
        Symmetry::Vertical => (None, Some(cy)),
        Symmetry::Both => (Some(cx), Some(cy)),
    }
}
