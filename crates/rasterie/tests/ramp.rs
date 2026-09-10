//! Parite des rampes avec la version TypeScript.
//!
//! Les rampes attendues viennent de `generateRamp`, relevees en la faisant
//! tourner. Une rampe qui differe d'un seul ton produirait des sprites
//! differents entre les deux versions.

use rasterie::ramp::{self, RampOptions};

/// Ce que la version TypeScript rend, relevé le 2026-09-10.
fn expect(base: &str, options: RampOptions, expected: &str) {
    let got = ramp::generate(base, options)
        .expect("couleur valide")
        .join(",");
    assert_eq!(got, expected, "rampe de {base} avec {options:?}");
}

#[test]
fn a_default_ramp_matches_the_typescript_one() {
    expect(
        "#7c3aed",
        RampOptions::default(),
        "#020523,#322e75,#765ebb,#ba98e8,#f4d9ff",
    );
    expect(
        "#f59e0b",
        RampOptions::default(),
        "#1c0000,#652400,#a45f00,#d1a24c,#f5e4b5",
    );
    expect(
        "#1e3a8a",
        RampOptions::default(),
        "#000a20,#003a75,#4a6cc3,#96a4f5,#e0dfff",
    );
}

#[test]
fn a_grey_stays_grey() {
    // Sans ce cas, la teinte par defaut (250, un bleu) teinterait tout l'ecran
    // en froid.
    expect(
        "#808080",
        RampOptions::default(),
        "#090909,#3a3a3a,#717171,#ababab,#e4e4e4",
    );
}

#[test]
fn the_number_of_stops_changes_the_curve() {
    expect(
        "#ff0000",
        RampOptions {
            stops: 3,
            ..RampOptions::default()
        },
        "#1e0008,#a0382d,#ffd3ba",
    );
    expect(
        "#00ff00",
        RampOptions {
            stops: 7,
            ..RampOptions::default()
        },
        "#0a0e00,#152700,#1b4b00,#237220,#449b59,#79c495,#b9ecd2",
    );
}

#[test]
fn a_single_stop_sits_in_the_middle() {
    expect(
        "#7c3aed",
        RampOptions {
            stops: 1,
            ..RampOptions::default()
        },
        "#654ca7",
    );
}

#[test]
fn turning_off_the_hue_shift_keeps_one_hue() {
    expect(
        "#abcdef",
        RampOptions {
            hue_shift_dark: 0.0,
            hue_shift_light: 0.0,
            ..RampOptions::default()
        },
        "#000a18,#193d5d,#4875a1,#88afd6,#d2e7fd",
    );
}

#[test]
fn the_chroma_peak_caps_the_saturation() {
    expect(
        "#123456",
        RampOptions {
            chroma_peak: 0.05,
            ..RampOptions::default()
        },
        "#010b10,#253e4e,#5c748e,#9cacc6,#dfe4f4",
    );
}

#[test]
fn neutrality_matches_the_typescript_definition() {
    use rasterie::oklch::{self, Rgb};

    for (colour, expected) in [
        ("#808080", true),
        ("#000000", true),
        ("#ffffff", true),
        ("#7c3aed", false),
        ("#abcdef", false),
    ] {
        let got = oklch::rgb_to_oklch(Rgb::from_hex(colour).unwrap()).is_neutral();
        assert_eq!(got, expected, "{colour}");
    }
}

#[test]
fn a_default_ramp_has_enough_contrast() {
    let generated = ramp::generate("#7c3aed", RampOptions::default()).unwrap();
    let issues = ramp::validate(&generated, ramp::DEFAULT_THRESHOLD);

    assert!(
        issues.is_empty(),
        "la rampe par defaut doit se lire : {issues:?}"
    );
}

#[test]
fn colours_too_close_together_are_reported() {
    // Deux tons voisins indistinguables font disparaitre une forme entiere.
    let plate = vec![
        "#808080".to_owned(),
        "#828282".to_owned(),
        "#ffffff".to_owned(),
    ];
    let issues = ramp::validate(&plate, ramp::DEFAULT_THRESHOLD);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].from, "#808080");
    assert!(issues[0].delta < ramp::DEFAULT_THRESHOLD);
}

#[test]
fn a_ramp_always_climbs_in_lightness() {
    for base in ["#7c3aed", "#f59e0b", "#808080", "#123456", "#00ff00"] {
        let generated = ramp::generate(base, RampOptions::default()).unwrap();
        let lightness: Vec<f64> = generated.iter().map(|c| ramp::luminance_of(c)).collect();

        for pair in lightness.windows(2) {
            assert!(
                pair[1] > pair[0],
                "la rampe de {base} redescend : {lightness:?}"
            );
        }
    }
}

#[test]
fn an_invalid_colour_is_refused() {
    assert!(ramp::generate("pas une couleur", RampOptions::default()).is_err());
    assert!(ramp::generate("", RampOptions::default()).is_err());
}

#[test]
fn zero_stops_still_gives_a_colour() {
    // Une rampe vide ferait planter tout ce qui l'utilise.
    let generated = ramp::generate(
        "#7c3aed",
        RampOptions {
            stops: 0,
            ..RampOptions::default()
        },
    )
    .unwrap();

    assert_eq!(generated.len(), 1);
}

#[test]
fn a_nearly_neutral_colour_still_gives_a_grey_ramp() {
    // La condition de neutralite ne compte pas pour un gris parfait — sa
    // chroma nulle annule deja le plafond — mais pour une couleur juste sous
    // le seuil, qui serait sinon teintee.
    expect(
        "#8b7c7a",
        RampOptions::default(),
        "#090909,#3a3a3a,#717171,#ababab,#e4e4e4",
    );
}

#[test]
fn just_past_the_threshold_the_hue_comes_back() {
    // Et juste au-dessus, la teinte reapparait : le seuil bascule bien.
    expect(
        "#8f7b77",
        RampOptions::default(),
        "#0f0708,#493434,#856a65,#bca59c,#eee2db",
    );
}
