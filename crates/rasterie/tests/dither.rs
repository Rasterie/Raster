//! Parite du tramage avec la version TypeScript.
//!
//! Les valeurs viennent de `ditherOffset` et `noiseAt`, relevées en les
//! faisant tourner sur une grille 8x8.

use rasterie::dither::{self, Dither};

/// Les valeurs sur une grille, ligne par ligne.
fn grid(kind: Dither, size: i64) -> Vec<String> {
    (0..size)
        .flat_map(|y| (0..size).map(move |x| format!("{:.4}", dither::offset(kind, x, y))))
        .collect()
}

#[test]
fn no_dither_offsets_nothing() {
    assert!(grid(Dither::None, 8).iter().all(|v| v == "0.0000"));
}

#[test]
fn the_checker_matches_the_typescript_one() {
    let got = grid(Dither::Checker, 8).join(",");
    assert_eq!(
        got,
        "-0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,\
         -0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,\
         0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,\
         0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,\
         -0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,\
         -0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,\
         0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000,\
         0.5000,0.5000,-0.5000,-0.5000,0.5000,0.5000,-0.5000,-0.5000"
            .replace(['\n', ' '], "")
    );
}

#[test]
fn bayer_two_matches() {
    let got = grid(Dither::Bayer2, 8).join(",");
    assert_eq!(
        got,
        "-0.5000,-0.5000,0.0000,0.0000,-0.5000,-0.5000,0.0000,0.0000,\
         -0.5000,-0.5000,0.0000,0.0000,-0.5000,-0.5000,0.0000,0.0000,\
         0.2500,0.2500,-0.2500,-0.2500,0.2500,0.2500,-0.2500,-0.2500,\
         0.2500,0.2500,-0.2500,-0.2500,0.2500,0.2500,-0.2500,-0.2500,\
         -0.5000,-0.5000,0.0000,0.0000,-0.5000,-0.5000,0.0000,0.0000,\
         -0.5000,-0.5000,0.0000,0.0000,-0.5000,-0.5000,0.0000,0.0000,\
         0.2500,0.2500,-0.2500,-0.2500,0.2500,0.2500,-0.2500,-0.2500,\
         0.2500,0.2500,-0.2500,-0.2500,0.2500,0.2500,-0.2500,-0.2500"
            .replace(['\n', ' '], "")
    );
}

#[test]
fn bayer_four_matches() {
    let got = grid(Dither::Bayer4, 8).join(",");
    assert_eq!(
        got,
        "-0.5000,-0.5000,0.0000,0.0000,-0.3750,-0.3750,0.1250,0.1250,\
         -0.5000,-0.5000,0.0000,0.0000,-0.3750,-0.3750,0.1250,0.1250,\
         0.2500,0.2500,-0.2500,-0.2500,0.3750,0.3750,-0.1250,-0.1250,\
         0.2500,0.2500,-0.2500,-0.2500,0.3750,0.3750,-0.1250,-0.1250,\
         -0.3125,-0.3125,0.1875,0.1875,-0.4375,-0.4375,0.0625,0.0625,\
         -0.3125,-0.3125,0.1875,0.1875,-0.4375,-0.4375,0.0625,0.0625,\
         0.4375,0.4375,-0.0625,-0.0625,0.3125,0.3125,-0.1875,-0.1875,\
         0.4375,0.4375,-0.0625,-0.0625,0.3125,0.3125,-0.1875,-0.1875"
            .replace(['\n', ' '], "")
    );
}

#[test]
fn noise_matches() {
    let got: Vec<String> = (0..4)
        .flat_map(|y| (0..4).map(move |x| format!("{:.6}", dither::noise_at(x, y))))
        .collect();

    assert_eq!(
        got.join(","),
        "-0.500000,-0.500000,0.421690,0.421690,\
         -0.500000,-0.500000,0.421690,0.421690,\
         -0.317084,-0.317084,0.240085,0.240085,\
         -0.317084,-0.317084,0.240085,0.240085"
            .replace(['\n', ' '], "")
    );
}

#[test]
fn every_pattern_works_in_two_by_two_blocks() {
    // Au pixel, le tramage produirait exactement les pixels isoles que les
    // regles du pixel art proscrivent.
    for kind in [Dither::Checker, Dither::Bayer2, Dither::Bayer4] {
        for y in 0..8i64 {
            for x in (0..8i64).step_by(2) {
                assert_eq!(
                    dither::offset(kind, x, y),
                    dither::offset(kind, x + 1, y),
                    "{kind:?} change au milieu d'un bloc en ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn every_offset_stays_within_half() {
    // Un decalage au-dela deplacerait le niveau de plus d'un palier entier.
    for kind in Dither::ALL {
        for y in -20..20i64 {
            for x in -20..20i64 {
                let v = dither::offset(kind, x, y);
                assert!(
                    (-0.5..=0.5).contains(&v),
                    "{kind:?} sort des bornes en ({x},{y}) : {v}"
                );
            }
        }
    }
}

#[test]
fn negative_coordinates_never_panic() {
    // Le TypeScript plante ici : `%` rend un negatif en JavaScript, donc
    // `m[-1]` est `undefined`. `rem_euclid` fait ce qu'on attend.
    for kind in Dither::ALL {
        for (x, y) in [(-1, -1), (-2, 0), (0, -3), (-4, -4), (-100, -100)] {
            let v = dither::offset(kind, x, y);
            assert!(v.is_finite(), "{kind:?} en ({x},{y})");
        }
    }
}

#[test]
fn the_pattern_repeats_over_its_period() {
    // Bayer 2 sur des blocs de 2 : une periode de 4 pixels.
    for y in 0..8i64 {
        for x in 0..8i64 {
            assert_eq!(
                dither::offset(Dither::Bayer2, x, y),
                dither::offset(Dither::Bayer2, x + 4, y + 4)
            );
        }
    }

    // Bayer 4 : une periode de 8.
    for y in 0..8i64 {
        for x in 0..8i64 {
            assert_eq!(
                dither::offset(Dither::Bayer4, x, y),
                dither::offset(Dither::Bayer4, x + 8, y + 8)
            );
        }
    }
}

#[test]
fn noise_is_repeatable() {
    // Une meme position doit toujours donner la meme valeur, sinon l'image
    // changerait a chaque rendu.
    for _ in 0..3 {
        assert_eq!(dither::noise_at(7, 13), dither::noise_at(7, 13));
    }
}

#[test]
fn noise_stays_within_half() {
    for y in -50..50i64 {
        for x in -50..50i64 {
            let v = dither::noise_at(x, y);
            assert!(
                (-0.5..=0.5).contains(&v),
                "bruit hors bornes en ({x},{y}) : {v}"
            );
        }
    }
}
