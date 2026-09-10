//! Grammaire de formes.
//!
//! Au lieu d'un catalogue ferme, une forme est une recette faite de trois
//! decisions independantes. Chaque decision est une donnee, donc les
//! combinaisons se multiplient sans ajouter de code.

use crate::mask::Mask;

/// The overall outline of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Silhouette {
    #[default]
    Rect,
    Diamond,
    Shield,
    Hexagon,
    Capsule,
    Banner,
    Cross,
}

/// What happens at the corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Corner {
    #[default]
    Square,
    Cut,
    Round,
    Spike,
    Notch,
}

/// How the inside is filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fill {
    #[default]
    Solid,
    Hollow,
    Double,
    Half,
}

/// How sharp the oblique edges are allowed to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Angles {
    /// Les aretes obliques sont redressees.
    Orthogonal,
    /// Le bord s'aligne sur une diagonale 1:1, la seule que l'oeil suit sans
    /// accroc en pixel art.
    Diagonal,
    #[default]
    Free,
}

impl Silhouette {
    pub const ALL: [Self; 7] = [
        Self::Rect,
        Self::Diamond,
        Self::Shield,
        Self::Hexagon,
        Self::Capsule,
        Self::Banner,
        Self::Cross,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Rect => "Rectangle",
            Self::Diamond => "Losange",
            Self::Shield => "Blason",
            Self::Hexagon => "Hexagone",
            Self::Capsule => "Capsule",
            Self::Banner => "Bandeau",
            Self::Cross => "Croix",
        }
    }
}

impl Corner {
    pub const ALL: [Self; 5] = [
        Self::Square,
        Self::Cut,
        Self::Round,
        Self::Spike,
        Self::Notch,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Square => "Droit",
            Self::Cut => "Coupé",
            Self::Round => "Arrondi",
            Self::Spike => "Épine",
            Self::Notch => "Encoche",
        }
    }
}

impl Fill {
    pub const ALL: [Self; 4] = [Self::Solid, Self::Hollow, Self::Double, Self::Half];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Plein",
            Self::Hollow => "Évidé",
            Self::Double => "Double",
            Self::Half => "Moitié",
        }
    }
}

/// A shape, as three independent decisions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recipe {
    pub silhouette: Silhouette,
    pub corner: Corner,
    pub corner_size: i64,
    pub fill: Fill,
    /// L'epaisseur du cadre quand le remplissage est evide.
    pub thickness: i64,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            silhouette: Silhouette::Rect,
            corner: Corner::Square,
            corner_size: 2,
            fill: Fill::Solid,
            thickness: 3,
        }
    }
}

/// Combien de formes la grammaire couvre.
pub const COMBINATIONS: usize = Silhouette::ALL.len() * Corner::ALL.len() * Fill::ALL.len();

/// Builds a mask from a recipe.
#[must_use]
pub fn build(recipe: Recipe, width: usize, height: usize, angles: Angles) -> Mask {
    let mut mask = Mask::new(width.max(3), height.max(3));

    draw_silhouette(&mut mask, recipe.silhouette, angles);
    apply_corners(&mut mask, recipe.corner, recipe.corner_size);
    apply_fill(&mut mask, recipe.fill, recipe.thickness);

    mask
}

fn draw_silhouette(mask: &mut Mask, silhouette: Silhouette, angles: Angles) {
    let (w, h) = (mask.width(), mask.height());
    let half_w = (w as f64 - 1.0) / 2.0;
    let half_h = (h as f64 - 1.0) / 2.0;

    for y in 0..h {
        for x in 0..w {
            // Normalise en -1..1 : la forme ne depend alors plus de la taille.
            let nx = (x as f64 - half_w) / if half_w == 0.0 { 1.0 } else { half_w };
            let ny = (y as f64 - half_h) / if half_h == 0.0 { 1.0 } else { half_h };

            let mut inside = match silhouette {
                Silhouette::Rect => true,
                Silhouette::Diamond => nx.abs() + ny.abs() <= 1.0,
                Silhouette::Hexagon => {
                    let flat = 0.5;
                    ny.abs() <= 1.0 && nx.abs() <= 1.0 - (ny.abs() - flat).max(0.0) / (1.0 - flat)
                }
                Silhouette::Capsule => {
                    let r = half_w.min(half_h);
                    let dx = ((x as f64 - half_w).abs() - (half_w - r)).max(0.0);
                    let dy = ((y as f64 - half_h).abs() - (half_h - r)).max(0.0);
                    dx * dx + dy * dy <= r * r
                }
                Silhouette::Shield => {
                    let shoulder = 0.1;
                    ny <= shoulder
                        || nx.abs() <= 1.0 - ((ny - shoulder) / (1.0 - shoulder)).powf(1.6)
                }
                Silhouette::Banner => {
                    let notch = 0.22;
                    nx.abs() <= 1.0 - notch * (1.0 - ny.abs())
                }
                Silhouette::Cross => {
                    let arm = 0.38;
                    nx.abs() <= arm || ny.abs() <= arm
                }
            };

            // Le jeu d'angles decide de la nettete des aretes obliques.
            if inside && silhouette != Silhouette::Rect {
                match angles {
                    Angles::Orthogonal => {
                        inside = nx.abs() <= 0.92 && ny.abs() <= 0.92;
                    }
                    Angles::Diagonal => {
                        if nx.abs() + ny.abs() > 1.34 {
                            inside = false;
                        }
                    }
                    Angles::Free => {}
                }
            }

            mask.put(x as i64, y as i64, inside);
        }
    }
}

/// Applique le traitement des angles sur une silhouette deja tracee.
///
/// On ne peut pas viser les coins du cadre : seul un rectangle en possede. On
/// travaille donc sur les pixels de bord, en mesurant a quel point chacun
/// occupe un angle de la forme — un pixel dont les voisins manquent dans deux
/// directions perpendiculaires est un coin, ou qu'il se trouve.
fn apply_corners(mask: &mut Mask, corner: Corner, size: i64) {
    if corner == Corner::Square || size <= 0 {
        return;
    }

    let original = mask.clone();
    let (w, h) = (mask.width() as i64, mask.height() as i64);

    for y in 0..h {
        for x in 0..w {
            if !original.is_inside(x, y) {
                continue;
            }

            let open = openness(&original, x, y);
            // Quatre voisins absents : le pixel est sur une pointe ou un angle.
            // En dessous il est sur un bord droit, qu'il ne faut pas entamer.
            if open < 4 {
                continue;
            }

            // La profondeur suit le degre d'angle : une pointe se creuse plus
            // qu'un coin franc.
            let reach = (((size * (open - 3)) as f64 / 4.0).round() as i64).max(1);

            for dy in -reach..=reach {
                for dx in -reach..=reach {
                    let d = dx.abs() + dy.abs();
                    let (px, py) = (x + dx, y + dy);

                    if !original.is_inside(px, py) {
                        continue;
                    }

                    let cut = match corner {
                        Corner::Cut => d < reach,
                        Corner::Round => dx * dx + dy * dy < reach * reach,
                        Corner::Notch => (d as f64) < reach as f64 - 0.5,
                        // L'epine alterne plein et vide : l'angle devient dentele.
                        Corner::Spike => d < reach && (px + py).rem_euclid(2) == 0,
                        Corner::Square => false,
                    };

                    if cut {
                        mask.put(px, py, false);
                    }
                }
            }
        }
    }
}

/// Combien de voisins manquent, dans les huit directions.
fn openness(mask: &Mask, x: i64, y: i64) -> i64 {
    let mut n = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if !mask.is_inside(x + dx, y + dy) {
                n += 1;
            }
        }
    }
    n
}

fn apply_fill(mask: &mut Mask, fill: Fill, thickness: i64) {
    if fill == Fill::Solid {
        return;
    }

    let (w, h) = (mask.width() as i64, mask.height() as i64);

    if fill == Fill::Half {
        for y in h / 2..h {
            for x in 0..w {
                mask.put(x, y, false);
            }
        }
        return;
    }

    let depth = thickness.max(1);
    let mut inner = Mask::new(mask.width(), mask.height());

    // Un pixel est interieur s'il est a plus de `depth` du bord de la forme.
    for y in 0..h {
        for x in 0..w {
            if !mask.is_inside(x, y) {
                continue;
            }

            let mut far = true;
            'search: for dy in -depth..=depth {
                for dx in -depth..=depth {
                    if dx.abs() + dy.abs() > depth {
                        continue;
                    }
                    if !mask.is_inside(x + dx, y + dy) {
                        far = false;
                        break 'search;
                    }
                }
            }

            if far {
                inner.put(x, y, true);
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            if inner.is_inside(x, y) {
                mask.put(x, y, false);
            }
        }
    }

    // Le mode double repose un lisere au centre de la zone evidee.
    if fill == Fill::Double {
        let gap = depth + 2;
        let mut ring = Mask::new(mask.width(), mask.height());

        for y in 0..h {
            for x in 0..w {
                if !inner.is_inside(x, y) {
                    continue;
                }
                let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| !inner.is_inside(x + dx * gap, y + dy * gap));

                if edge {
                    ring.put(x, y, true);
                }
            }
        }

        for y in 0..h {
            for x in 0..w {
                if ring.is_inside(x, y) {
                    mask.put(x, y, true);
                }
            }
        }
    }
}
