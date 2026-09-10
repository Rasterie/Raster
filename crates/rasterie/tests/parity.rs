//! Parite avec la version TypeScript.
//!
//! Les valeurs attendues viennent de `culori`, la bibliotheque que le moteur
//! TypeScript utilise, relevees en la faisant tourner. Les inventer aurait
//! verifie mon calcul contre lui-meme.

use rasterie::oklch::{self, Oklch, Rgb};

/// La tolerance : `culori` travaille en `f64` comme nous, et les references
/// sont arrondies a six decimales.
const EPSILON: f64 = 1e-5;

/// Les couleurs et ce que `culori` en dit, relevé le 2026-09-10.
const REFERENCES: &[(&str, f64, f64, f64)] = &[
    ("#000000", 0.000_000, 0.000_000, 0.0000),
    ("#ffffff", 1.000_000, 0.000_000, 0.0000),
    ("#ff0000", 0.627_955, 0.257_683, 29.2339),
    ("#00ff00", 0.866_440, 0.294_827, 142.4953),
    ("#0000ff", 0.452_014, 0.313_214, 264.0520),
    ("#808080", 0.599_871, 0.000_000, 0.0000),
    ("#7c3aed", 0.541_337, 0.246_586, 293.0090),
    ("#f59e0b", 0.768_590, 0.164_659, 70.0804),
    ("#1e3a8a", 0.379_059, 0.137_761, 265.5222),
    ("#84cc16", 0.768_141, 0.204_401, 130.8498),
    ("#123456", 0.319_168, 0.072_455, 251.1685),
    ("#abcdef", 0.834_979, 0.060_153, 248.5504),
    ("#0a0a0a", 0.144_788, 0.000_000, 0.0000),
    ("#fefefe", 0.997_025, 0.000_000, 0.0000),
];

#[test]
fn rgb_to_oklch_matches_culori() {
    for (hex, l, c, h) in REFERENCES {
        let rgb = Rgb::from_hex(hex).expect("hex valide");
        let got = oklch::rgb_to_oklch(rgb);

        assert!(
            (got.l - l).abs() < EPSILON,
            "{hex} : luminance {} au lieu de {l}",
            got.l
        );
        assert!(
            (got.c - c).abs() < EPSILON,
            "{hex} : chroma {} au lieu de {c}",
            got.c
        );

        // La teinte d'un gris n'a pas de sens : `culori` rend 0, nous aussi,
        // mais la comparer n'apprendrait rien.
        if *c > 0.001 {
            let ecart = (got.h - h).abs().min(360.0 - (got.h - h).abs());
            assert!(ecart < 1e-3, "{hex} : teinte {} au lieu de {h}", got.h);
        }
    }
}

#[test]
fn oklch_to_hex_matches_culori() {
    // Les memes cas, dans l'autre sens.
    for (l, c, h, expected) in [
        (0.5, 0.1, 250.0, "#32669a"),
        (0.14, 0.06, 30.0, "#1c0000"),
        (0.92, 0.02, 120.0, "#e3e7d8"),
        (0.7, 0.15, 340.0, "#da76bb"),
        (0.35, 0.12, 90.0, "#533400"),
    ] {
        let got = oklch::oklch_to_rgb(Oklch::new(l, c, h)).to_hex();
        assert_eq!(got, expected, "oklch({l}, {c}, {h})");
    }
}

#[test]
fn a_colour_survives_a_round_trip() {
    for (hex, _, _, _) in REFERENCES {
        let rgb = Rgb::from_hex(hex).unwrap();
        let back = oklch::oklch_to_rgb(oklch::rgb_to_oklch(rgb));

        assert_eq!(back.to_hex(), *hex, "{hex} n'est pas revenu identique");
    }
}

#[test]
fn every_representable_grey_survives() {
    // Les gris passent par une chroma nulle : c'est la que l'arrondi de la
    // teinte pourrait les faire virer.
    for level in 0..=255u8 {
        let c = f64::from(level) / 255.0;
        let rgb = Rgb::new(c, c, c);
        let back = oklch::oklch_to_rgb(oklch::rgb_to_oklch(rgb));

        assert_eq!(
            back.to_bytes(),
            [level, level, level],
            "le gris {level} a vire"
        );
    }
}

#[test]
fn a_grey_has_no_chroma() {
    for level in [0u8, 64, 128, 192, 255] {
        let c = f64::from(level) / 255.0;
        let got = oklch::rgb_to_oklch(Rgb::new(c, c, c));

        assert!(got.c < 1e-6, "le gris {level} a une chroma de {}", got.c);
        assert!(got.is_neutral());
    }
}

#[test]
fn hex_is_read_in_every_form() {
    let attendu = Rgb::from_hex("#aabbcc").unwrap();

    assert_eq!(Rgb::from_hex("aabbcc"), Some(attendu), "sans le diese");
    assert_eq!(Rgb::from_hex("#abc"), Some(attendu), "la forme courte");
    assert_eq!(
        Rgb::from_hex("  #aabbcc  "),
        Some(attendu),
        "avec des espaces"
    );
}

#[test]
fn something_that_is_not_a_colour_is_refused() {
    for mauvais in ["", "#", "#12", "#12345", "#gggggg", "rouge"] {
        assert_eq!(Rgb::from_hex(mauvais), None, "`{mauvais}` a ete accepte");
    }
}

#[test]
fn a_hue_beyond_the_circle_wraps() {
    let a = Oklch::new(0.5, 0.1, 400.0);
    let b = Oklch::new(0.5, 0.1, 40.0);

    assert!((a.h - b.h).abs() < 1e-9, "{} et {}", a.h, b.h);
    assert_eq!(
        oklch::oklch_to_rgb(a).to_hex(),
        oklch::oklch_to_rgb(b).to_hex()
    );
}

#[test]
fn a_negative_hue_wraps_too() {
    let a = Oklch::new(0.5, 0.1, -20.0);
    assert!((a.h - 340.0).abs() < 1e-9, "{}", a.h);
}

#[test]
fn a_negative_chroma_is_refused() {
    // Une chroma negative n'a pas de sens : elle inverserait la teinte.
    assert_eq!(Oklch::new(0.5, -0.1, 0.0).c, 0.0);
}

#[test]
fn colours_outside_srgb_are_recognised() {
    // Une chroma que l'ecran ne peut pas montrer.
    assert!(!oklch::in_gamut(Oklch::new(0.5, 0.5, 250.0)));
    assert!(oklch::in_gamut(Oklch::new(0.5, 0.05, 250.0)));
    assert!(oklch::in_gamut(Oklch::new(0.0, 0.0, 0.0)), "le noir");
    assert!(oklch::in_gamut(Oklch::new(1.0, 0.0, 0.0)), "le blanc");
}
