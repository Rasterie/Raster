use raster_ed_visual::Frame;
use raster_math::IVec2;
use raster_render::Colour;
use rasterie::dither::{self, Dither};
use rasterie::grammar::{self, Angles, Recipe};
use rasterie::oklch::Rgb;
use rasterie::ramp::{self, RampOptions};

/// Where the light comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Light {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
}

impl Light {
    pub const ALL: [Self; 5] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Left,
        Self::Right,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Haut-gauche",
            Self::Top => "Haut",
            Self::TopRight => "Haut-droite",
            Self::Left => "Gauche",
            Self::Right => "Droite",
        }
    }

    /// La direction d'ou vient la lumiere, normalisee.
    #[must_use]
    pub fn direction(self) -> (f64, f64) {
        // Y vers le bas, comme partout dans le moteur.
        match self {
            Self::TopLeft => (-0.707, -0.707),
            Self::Top => (0.0, -1.0),
            Self::TopRight => (0.707, -0.707),
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
        }
    }
}

/// Everything that decides what a generated sprite looks like.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub recipe: Recipe,
    pub angles: Angles,
    /// La couleur de base, en hexadecimal.
    pub base: String,
    pub ramp: RampOptions,
    pub light: Light,
    pub dither: Dither,
    /// Un contour, pose sur le bord de la forme.
    pub outline: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            recipe: Recipe::default(),
            angles: Angles::Free,
            base: "#7c3aed".to_owned(),
            ramp: RampOptions::default(),
            light: Light::TopLeft,
            dither: Dither::None,
            outline: true,
        }
    }
}

/// Why generation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateError {
    /// La couleur de base ne se lit pas.
    Colour(String),
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Colour(c) => write!(f, "couleur invalide : {c}"),
        }
    }
}

impl std::error::Error for GenerateError {}

/// Builds a sprite from parameters.
///
/// # Errors
///
/// If the base colour cannot be read.
pub fn generate(params: &Params, width: i32, height: i32) -> Result<Frame, GenerateError> {
    let shades = ramp::generate(&params.base, params.ramp)
        .map_err(|_| GenerateError::Colour(params.base.clone()))?;

    let mask = grammar::build(
        params.recipe,
        width.max(3) as usize,
        height.max(3) as usize,
        params.angles,
    );

    let mut frame = Frame::new(mask.width() as i32, mask.height() as i32);
    let (lx, ly) = params.light.direction();
    let steps = shades.len();

    for y in 0..mask.height() as i64 {
        for x in 0..mask.width() as i64 {
            if !mask.is_inside(x, y) {
                continue;
            }

            let at = IVec2::new(x as i32, y as i32);

            // Le contour prend le ton le plus sombre : c'est ce qui detache la
            // forme de son fond sans l'ecraser.
            if params.outline && on_edge(&mask, x, y) {
                if let Some(colour) = parse(&shades[0]) {
                    frame.set(at, colour);
                }
                continue;
            }

            let level =
                lighting(&mask, x, y, lx, ly) + dither::offset(params.dither, x, y) / steps as f64;

            let index =
                ((level * (steps - 1) as f64).round() as i64).clamp(0, steps as i64 - 1) as usize;

            if let Some(colour) = parse(&shades[index]) {
                frame.set(at, colour);
            }
        }
    }

    Ok(frame)
}

/// Whether a filled cell touches the outside.
fn on_edge(mask: &rasterie::Mask, x: i64, y: i64) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(dx, dy)| !mask.is_inside(x + dx, y + dy))
}

/// How lit a cell is, from 0 (shadow) to 1 (highlight).
///
/// La distance au bord dans la direction de la lumiere : ce qui est du cote
/// eclaire et loin du bord recoit le plus.
fn lighting(mask: &rasterie::Mask, x: i64, y: i64, lx: f64, ly: f64) -> f64 {
    // Sonde a quelle distance le bord se trouve, du cote de la lumiere et du
    // cote oppose. Le rapport donne la position dans la rampe.
    let reach = 6;
    let towards = distance_to_edge(mask, x, y, lx, ly, reach);
    let away = distance_to_edge(mask, x, y, -lx, -ly, reach);

    let total = towards + away;
    if total <= 0.0 {
        return 0.5;
    }

    // Loin du bord eclaire = sombre ; pres = clair.
    (away / total).clamp(0.0, 1.0)
}

fn distance_to_edge(mask: &rasterie::Mask, x: i64, y: i64, dx: f64, dy: f64, reach: i64) -> f64 {
    for step in 1..=reach {
        let px = x + (dx * step as f64).round() as i64;
        let py = y + (dy * step as f64).round() as i64;

        if !mask.is_inside(px, py) {
            return step as f64;
        }
    }
    reach as f64 + 1.0
}

fn parse(hex: &str) -> Option<Colour> {
    let rgb = Rgb::from_hex(hex)?;
    Some(Colour::rgb(rgb.r as f32, rgb.g as f32, rgb.b as f32))
}
