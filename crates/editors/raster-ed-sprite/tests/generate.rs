use raster_ed_sprite::generate::{self, Light, Params};
use raster_math::IVec2;
use rasterie::dither::Dither;
use rasterie::grammar::{Recipe, Silhouette};

fn at(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

fn params(silhouette: Silhouette) -> Params {
    Params {
        recipe: Recipe {
            silhouette,
            ..Recipe::default()
        },
        ..Params::default()
    }
}

/// La luminance perçue d'un pixel, ou `None` s'il est vide.
fn light_at(frame: &raster_ed_visual::Frame, x: i32, y: i32) -> Option<f32> {
    let c = frame.get(at(x, y))?;
    let [r, g, b, a] = c.to_array();
    (a > 0.0).then_some(0.299 * r + 0.587 * g + 0.114 * b)
}

#[test]
fn a_sprite_comes_out_the_size_asked_for() {
    let frame = generate::generate(&Params::default(), 24, 16).unwrap();

    assert_eq!(frame.width(), 24);
    assert_eq!(frame.height(), 16);
}

#[test]
fn a_sprite_is_never_blank() {
    for silhouette in Silhouette::ALL {
        let frame = generate::generate(&params(silhouette), 20, 20).unwrap();
        assert!(!frame.is_blank(), "{silhouette:?} ne dessine rien");
    }
}

#[test]
fn an_invalid_colour_is_refused() {
    let params = Params {
        base: "pas une couleur".to_owned(),
        ..Params::default()
    };

    assert!(generate::generate(&params, 16, 16).is_err());
}

#[test]
fn the_outline_takes_the_darkest_shade() {
    let frame = generate::generate(&params(Silhouette::Hexagon), 20, 20).unwrap();

    // Le premier pixel rencontre est sur le contour.
    let contour = (0..20)
        .flat_map(|y| (0..20).map(move |x| (x, y)))
        .find_map(|(x, y)| light_at(&frame, x, y))
        .expect("le sprite dessine quelque chose");

    // Le centre est eclaire, le bord ne l'est pas.
    let centre = light_at(&frame, 10, 10).expect("le centre est plein");

    assert!(
        contour < centre,
        "le contour ({contour:.3}) doit etre plus sombre que le corps ({centre:.3})"
    );
}

#[test]
fn turning_off_the_outline_lightens_the_edge() {
    let avec = generate::generate(&params(Silhouette::Rect), 20, 20).unwrap();
    let sans = generate::generate(
        &Params {
            outline: false,
            ..params(Silhouette::Rect)
        },
        20,
        20,
    )
    .unwrap();

    let bord_avec = light_at(&avec, 0, 10).unwrap();
    let bord_sans = light_at(&sans, 0, 10).unwrap();

    assert!(
        bord_sans > bord_avec,
        "sans contour, le bord doit s'eclaircir"
    );
}

#[test]
fn the_light_direction_changes_where_the_highlight_is() {
    let gauche = generate::generate(
        &Params {
            light: Light::Left,
            outline: false,
            ..params(Silhouette::Rect)
        },
        20,
        20,
    )
    .unwrap();

    let droite = generate::generate(
        &Params {
            light: Light::Right,
            outline: false,
            ..params(Silhouette::Rect)
        },
        20,
        20,
    )
    .unwrap();

    // Ce qui est clair a gauche doit etre sombre a droite, et l'inverse.
    let g_gauche = light_at(&gauche, 1, 10).unwrap();
    let g_droite = light_at(&droite, 1, 10).unwrap();

    assert!(
        g_gauche > g_droite,
        "le bord gauche est plus clair sous une lumiere de gauche : {g_gauche:.3} contre {g_droite:.3}"
    );
}

#[test]
fn every_light_direction_produces_a_gradient() {
    for light in Light::ALL {
        let frame = generate::generate(
            &Params {
                light,
                outline: false,
                ..params(Silhouette::Rect)
            },
            20,
            20,
        )
        .unwrap();

        let valeurs: Vec<f32> = (0..20)
            .flat_map(|y| (0..20).filter_map(move |x| (x, y).into()))
            .filter_map(|(x, y): (i32, i32)| light_at(&frame, x, y))
            .collect();

        let min = valeurs.iter().cloned().fold(f32::MAX, f32::min);
        let max = valeurs.iter().cloned().fold(f32::MIN, f32::max);

        assert!(
            max - min > 0.05,
            "{light:?} ne produit aucun degrade : {min:.3} a {max:.3}"
        );
    }
}

#[test]
fn dithering_changes_the_image() {
    let net = generate::generate(&params(Silhouette::Rect), 24, 24).unwrap();
    let trame = generate::generate(
        &Params {
            dither: Dither::Bayer4,
            ..params(Silhouette::Rect)
        },
        24,
        24,
    )
    .unwrap();

    assert_ne!(net.pixels(), trame.pixels(), "le tramage doit se voir");
}

#[test]
fn dithering_never_adds_a_colour_outside_the_ramp() {
    // Le tramage entremele des paliers existants, il n'en invente pas.
    let net = generate::generate(&params(Silhouette::Rect), 24, 24).unwrap();
    let trame = generate::generate(
        &Params {
            dither: Dither::Bayer4,
            ..params(Silhouette::Rect)
        },
        24,
        24,
    )
    .unwrap();

    let teintes = |f: &raster_ed_visual::Frame| {
        let mut v: Vec<String> = f
            .pixels()
            .iter()
            .filter(|c| c.to_array()[3] > 0.0)
            .map(|c| format!("{:?}", c.to_array()))
            .collect();
        v.sort();
        v.dedup();
        v
    };

    let du_net = teintes(&net);
    for teinte in teintes(&trame) {
        assert!(
            du_net.contains(&teinte),
            "le tramage a invente une couleur : {teinte}"
        );
    }
}

#[test]
fn a_generated_sprite_only_uses_the_ramp() {
    use rasterie::ramp::{self, RampOptions};

    let params = Params::default();
    let shades = ramp::generate(&params.base, RampOptions::default()).unwrap();
    let frame = generate::generate(&params, 20, 20).unwrap();

    // Chaque pixel doit venir de la rampe : une couleur hors palette casserait
    // l'echange de palette et l'audit.
    let attendues: Vec<[f32; 4]> = shades
        .iter()
        .map(|s| {
            let rgb = rasterie::oklch::Rgb::from_hex(s).unwrap();
            [rgb.r as f32, rgb.g as f32, rgb.b as f32, 1.0]
        })
        .collect();

    for pixel in frame.pixels().iter().filter(|c| c.to_array()[3] > 0.0) {
        let a = pixel.to_array();
        assert!(
            attendues
                .iter()
                .any(|e| e.iter().zip(a.iter()).all(|(x, y)| (x - y).abs() < 1e-4)),
            "couleur hors rampe : {a:?}"
        );
    }
}

#[test]
fn a_tiny_sprite_still_works() {
    // La grammaire impose un minimum de trois : demander moins ne doit pas
    // paniquer.
    let frame = generate::generate(&Params::default(), 1, 1).unwrap();

    assert_eq!(frame.width(), 3);
    assert!(!frame.is_blank());
}

#[test]
fn the_same_parameters_always_give_the_same_sprite() {
    // Sans cela, l'apercu changerait a chaque frame.
    let a = generate::generate(&Params::default(), 20, 20).unwrap();
    let b = generate::generate(&Params::default(), 20, 20).unwrap();

    assert_eq!(a.pixels(), b.pixels());
}
