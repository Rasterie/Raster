//! OKLCH et ses conversions.
//!
//! Porte de `culori`, que la version TypeScript utilise. Les constantes
//! viennent de la definition d'Oklab par Bjorn Ottosson : les changer, meme
//! d'une decimale, ferait diverger les deux versions.

/// A colour in OKLCH: lightness, chroma, hue.
///
/// Perceptuellement uniforme : deux couleurs distantes de la meme valeur
/// paraissent aussi differentes, ce qu'aucun espace RGB ne donne.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    /// De 0 (noir) a 1 (blanc).
    pub l: f64,
    /// La saturation perceptuelle. Zero est un gris.
    pub c: f64,
    /// La teinte en degres, de 0 a 360.
    pub h: f64,
}

impl Oklch {
    #[must_use]
    pub fn new(l: f64, c: f64, h: f64) -> Self {
        Self {
            l,
            c: c.max(0.0),
            h: h.rem_euclid(360.0),
        }
    }

    /// Whether the colour has no usable hue.
    ///
    /// Le seuil porte sur la chroma OKLCH, perceptuellement uniforme : un ecart
    /// RGB varie trop avec la luminance pour delimiter le meme ensemble.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.c < Self::NEUTRAL_CHROMA
    }

    /// En deca, une teinte ne peut ni virer ni se saturer davantage.
    pub const NEUTRAL_CHROMA: f64 = 0.02;
}

/// Un canal RGB entre 0 et 1, en espace sRGB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// The eight-bit form, clamped.
    #[must_use]
    pub fn to_bytes(self) -> [u8; 3] {
        [byte(self.r), byte(self.g), byte(self.b)]
    }

    /// `#rrggbb`, as the TypeScript version writes it.
    #[must_use]
    pub fn to_hex(self) -> String {
        let [r, g, b] = self.to_bytes();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// Reads `#rgb`, `#rrggbb`, or the same without the hash.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        let hex = text.trim().trim_start_matches('#');

        let (r, g, b) = match hex.len() {
            // La forme courte double chaque chiffre : `#abc` vaut `#aabbcc`.
            3 => {
                let d = |i: usize| u8::from_str_radix(&hex[i..=i].repeat(2), 16).ok();
                (d(0)?, d(1)?, d(2)?)
            }
            6 => {
                let d = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
                (d(0)?, d(2)?, d(4)?)
            }
            _ => return None,
        };

        Some(Self::new(
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        ))
    }
}

fn byte(channel: f64) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// sRGB vers lineaire : la courbe que tout ecran applique, et sans laquelle
/// les melanges de couleurs sortent trop sombres.
fn to_linear(channel: f64) -> f64 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(channel: f64) -> f64 {
    if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts sRGB to OKLCH.
#[must_use]
pub fn rgb_to_oklch(rgb: Rgb) -> Oklch {
    let (r, g, b) = (to_linear(rgb.r), to_linear(rgb.g), to_linear(rgb.b));

    // Vers l'espace LMS, puis sa racine cubique : c'est ce qui rend Oklab
    // perceptuellement uniforme.
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;

    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());

    let lightness = 0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s;
    let a = 1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s;
    let b_axis = 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s;

    let chroma = (a * a + b_axis * b_axis).sqrt();
    // Une couleur sans chroma n'a pas de teinte : atan2(0,0) vaut zero, ce qui
    // ferait passer un gris pour du rouge.
    let hue = if chroma < 1e-9 {
        0.0
    } else {
        b_axis.atan2(a).to_degrees().rem_euclid(360.0)
    };

    Oklch {
        l: lightness,
        c: chroma,
        h: hue,
    }
}

/// Converts OKLCH back to sRGB.
///
/// Le resultat peut sortir de l'espace sRGB : c'est a l'appelant de decider
/// s'il borne ou s'il rapproche. `to_bytes` borne.
#[must_use]
pub fn oklch_to_rgb(colour: Oklch) -> Rgb {
    let radians = colour.h.to_radians();
    let a = colour.c * radians.cos();
    let b_axis = colour.c * radians.sin();

    let l = colour.l + 0.396_337_777_4 * a + 0.215_803_757_3 * b_axis;
    let m = colour.l - 0.105_561_345_8 * a - 0.063_854_172_8 * b_axis;
    let s = colour.l - 0.089_484_177_5 * a - 1.291_485_548_0 * b_axis;

    let (l, m, s) = (l * l * l, m * m * m, s * s * s);

    let r = 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s;
    let g = -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s;
    let b = -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s;

    Rgb::new(from_linear(r), from_linear(g), from_linear(b))
}

/// Whether a colour fits inside sRGB.
///
/// OKLCH decrit des couleurs qu'aucun ecran ne montre : une rampe qui en
/// produit sortirait bornee, donc aplatie.
#[must_use]
pub fn in_gamut(colour: Oklch) -> bool {
    let rgb = oklch_to_rgb(colour);
    let inside = |c: f64| (-0.000_5..=1.000_5).contains(&c);
    inside(rgb.r) && inside(rgb.g) && inside(rgb.b)
}
