//! Tramage et bruit.
//!
//! Les deux travaillent par blocs de 2x2 : au pixel, ils produiraient
//! exactement les pixels isoles que les regles du pixel art proscrivent.

/// How two neighbouring shades are blended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dither {
    #[default]
    None,
    Bayer2,
    Bayer4,
    Checker,
}

impl Dither {
    pub const ALL: [Self; 4] = [Self::None, Self::Bayer2, Self::Bayer4, Self::Checker];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Aucun",
            Self::Bayer2 => "Bayer 2",
            Self::Bayer4 => "Bayer 4",
            Self::Checker => "Damier",
        }
    }
}

const BAYER_2: [[f64; 2]; 2] = [[0.0, 2.0], [3.0, 1.0]];

const BAYER_4: [[f64; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

/// The dithering threshold at a point, between -0.5 and 0.5.
///
/// Il decale le niveau d'eclairage avant quantification : deux paliers voisins
/// s'entremelent en motif regulier au lieu de se separer net.
#[must_use]
pub fn offset(kind: Dither, x: i64, y: i64) -> f64 {
    if kind == Dither::None {
        return 0.0;
    }

    // Le motif travaille par blocs de 2x2 : un damier au pixel produit des
    // points isoles, que la regle des grappes proscrit.
    let bx = x.div_euclid(2);
    let by = y.div_euclid(2);

    match kind {
        Dither::Checker => {
            if (bx + by).rem_euclid(2) == 0 {
                -0.5
            } else {
                0.5
            }
        }
        Dither::Bayer2 => {
            let n = 2;
            BAYER_2[by.rem_euclid(n) as usize][bx.rem_euclid(n) as usize] / 4.0 - 0.5
        }
        Dither::Bayer4 => {
            let n = 4;
            BAYER_4[by.rem_euclid(n) as usize][bx.rem_euclid(n) as usize] / 16.0 - 0.5
        }
        Dither::None => 0.0,
    }
}

/// Repeatable noise: the same position always gives the same value.
///
/// Par blocs de 2x2 comme le tramage, et pour la meme raison.
#[must_use]
pub fn noise_at(x: i64, y: i64) -> f64 {
    let bx = x.div_euclid(2) as f64;
    let by = y.div_euclid(2) as f64;

    // Le hachage flottant classique : sa valeur exacte importe peu, sa
    // reproductibilite si.
    let v = (bx * 12.9898 + by * 78.233).sin() * 43758.5453;
    v - v.floor() - 0.5
}
