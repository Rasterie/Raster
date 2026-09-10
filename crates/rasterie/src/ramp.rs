use crate::oklch::{self, Oklch, Rgb};

/// La courbe de luminance d'une rampe a cinq tons.
///
/// Relevee a la main plutot que calculee : une progression reguliere donne des
/// ombres trop claires et des lumieres trop sombres pour du pixel art.
const LUMINANCE_CURVE: [f64; 5] = [0.14, 0.35, 0.55, 0.74, 0.92];

/// How a ramp is built from one colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RampOptions {
    pub stops: usize,
    /// De combien la teinte vire vers le froid dans les ombres, en degres.
    pub hue_shift_dark: f64,
    /// Et vers le chaud dans les lumieres.
    pub hue_shift_light: f64,
    /// La chroma maximale atteinte au milieu de la rampe.
    pub chroma_peak: f64,
}

impl Default for RampOptions {
    fn default() -> Self {
        Self {
            stops: 5,
            hue_shift_dark: 25.0,
            hue_shift_light: 20.0,
            chroma_peak: 0.14,
        }
    }
}

/// The lightness of one stop.
fn luminance_at(index: usize, stops: usize) -> f64 {
    if stops == LUMINANCE_CURVE.len() {
        return LUMINANCE_CURVE[index];
    }

    let t = if stops == 1 {
        0.5
    } else {
        index as f64 / (stops - 1) as f64
    };
    0.15 + t.powf(1.15) * 0.75
}

/// La chroma culmine au milieu : les ombres et les lumieres extremes sont
/// moins saturees, comme dans la peinture.
fn chroma_scale_at(t: f64) -> f64 {
    0.45 + (t * std::f64::consts::PI).sin() * 0.55
}

/// Le decalage de teinte : les ombres virent d'un cote, les lumieres de
/// l'autre. C'est ce qui donne sa vie a une rampe de pixel art.
fn hue_shift_at(t: f64, dark: f64, light: f64) -> f64 {
    if t < 0.5 {
        -dark * (1.0 - t / 0.5)
    } else {
        light * ((t - 0.5) / 0.5)
    }
}

/// Builds a ramp from a base colour.
///
/// # Errors
///
/// If the colour cannot be read.
pub fn generate(base: &str, options: RampOptions) -> Result<Vec<String>, RampError> {
    let rgb = Rgb::from_hex(base).ok_or_else(|| RampError::Invalid(base.to_owned()))?;
    Ok(generate_from(oklch::rgb_to_oklch(rgb), options))
}

/// The same, from a colour already in OKLCH.
#[must_use]
pub fn generate_from(source: Oklch, options: RampOptions) -> Vec<String> {
    let stops = options.stops.max(1);

    // Un gris neutre le reste : le decalage de teinte n'a de sens que sur une
    // couleur qui en a une. Sinon 250 (bleu) teinte tout l'ecran en froid.
    let neutral = source.is_neutral();
    let base_hue = if neutral { 250.0 } else { source.h };
    let ceiling = if neutral {
        0.0
    } else {
        options.chroma_peak.min(source.c * 1.4)
    };

    (0..stops)
        .map(|index| {
            let t = if stops == 1 {
                0.5
            } else {
                index as f64 / (stops - 1) as f64
            };

            let colour = Oklch::new(
                luminance_at(index, stops),
                ceiling * chroma_scale_at(t),
                base_hue + hue_shift_at(t, options.hue_shift_dark, options.hue_shift_light),
            );

            oklch::oklch_to_rgb(colour).to_hex()
        })
        .collect()
}

/// Why a ramp could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RampError {
    Invalid(String),
}

impl std::fmt::Display for RampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(colour) => write!(f, "couleur invalide : {colour}"),
        }
    }
}

impl std::error::Error for RampError {}

/// The lightness of a colour, for checking a ramp reads.
#[must_use]
pub fn luminance_of(colour: &str) -> f64 {
    Rgb::from_hex(colour).map_or(0.0, |rgb| oklch::rgb_to_oklch(rgb).l)
}

/// Two neighbouring colours a ramp does not separate enough.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastIssue {
    pub from: String,
    pub to: String,
    pub delta: f64,
}

/// Finds neighbours too close in lightness to read apart.
///
/// En pixel art, deux tons voisins qui ne se distinguent pas font disparaitre
/// une forme entiere.
#[must_use]
pub fn validate(ramp: &[String], threshold: f64) -> Vec<ContrastIssue> {
    ramp.windows(2)
        .filter_map(|pair| {
            let delta = (luminance_of(&pair[1]) - luminance_of(&pair[0])).abs();
            (delta < threshold).then(|| ContrastIssue {
                from: pair[0].clone(),
                to: pair[1].clone(),
                delta,
            })
        })
        .collect()
}

/// Le seuil par defaut, celui de la version TypeScript.
pub const DEFAULT_THRESHOLD: f64 = 0.15;
