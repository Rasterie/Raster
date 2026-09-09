use raster_math::{IRect, IVec2, Rect, Vec2};

/// Ce que `IRect::from_rect` promet : englober, jamais rogner.
#[test]
fn a_float_rectangle_is_enclosed_not_trimmed() {
    let r = IRect::from_rect(Rect::new(10.4, 20.6, 30.2, 40.9));

    assert_eq!(r.position, IVec2::new(10, 20), "l'origine doit descendre");
    // De 10 a 40.6 : englober donne 10 a 41, soit 31 de large.
    assert_eq!(r.size, IVec2::new(31, 42));
}

#[test]
fn a_whole_rectangle_is_unchanged() {
    let r = IRect::from_rect(Rect::new(8.0, 16.0, 32.0, 24.0));

    assert_eq!(r.position, IVec2::new(8, 16));
    assert_eq!(r.size, IVec2::new(32, 24));
}

#[test]
fn an_empty_rectangle_stays_empty() {
    let r = IRect::from_rect(Rect::new(5.0, 5.0, 0.0, 0.0));
    assert_eq!(r.size, IVec2::ZERO);
}

#[test]
fn a_negative_position_survives() {
    // Un panneau peut commencer hors de l'ecran : la decoupe doit le suivre.
    let r = IRect::from_rect(Rect::new(-4.5, -2.0, 10.0, 10.0));
    assert_eq!(r.position, IVec2::new(-5, -2));
}

#[test]
fn nested_clips_intersect() {
    let parent = IRect::from_rect(Rect::new(0.0, 0.0, 100.0, 100.0));
    let enfant = IRect::from_rect(Rect::new(50.0, 50.0, 100.0, 100.0));

    // Un panneau imbrique ne doit jamais deborder de son parent.
    let commun = parent.intersection(enfant).unwrap();
    assert_eq!(commun.position, IVec2::new(50, 50));
    assert_eq!(commun.size, IVec2::new(50, 50));
}

#[test]
fn disjoint_clips_intersect_to_nothing() {
    let a = IRect::from_rect(Rect::new(0.0, 0.0, 10.0, 10.0));
    let b = IRect::from_rect(Rect::new(50.0, 50.0, 10.0, 10.0));

    assert_eq!(a.intersection(b), None, "deux panneaux disjoints");
}

#[test]
fn a_clip_the_size_of_the_view_covers_everything() {
    let vue = Rect::new(0.0, 0.0, 320.0, 180.0);
    let clip = IRect::from_rect(vue);

    assert_eq!(clip.size, IVec2::new(320, 180));
}

/// Vérifie qu'un rectangle du monde tombe bien sur les pixels attendus.
#[test]
fn the_scissor_follows_the_camera() {
    use raster_render::{Camera, SpriteBatch};

    let mut camera = Camera::new(320, 180);
    camera.position = Vec2::new(160.0, 90.0);

    // Un panneau au centre de la vue.
    let clip = IRect::from_rect(Rect::new(160.0, 90.0, 80.0, 40.0));
    let (x, y, w, h) = SpriteBatch::scissor(clip, camera, (320, 180)).unwrap();

    assert_eq!((w, h), (80, 40), "la taille doit etre conservee");
    assert_eq!((x, y), (160, 90), "le coin doit tomber au milieu de la vue");
}

#[test]
fn a_scissor_outside_the_view_is_dropped() {
    use raster_render::{Camera, SpriteBatch};

    let camera = Camera::new(320, 180);
    let loin = IRect::from_rect(Rect::new(10_000.0, 10_000.0, 10.0, 10.0));

    assert_eq!(
        SpriteBatch::scissor(loin, camera, (320, 180)),
        None,
        "une decoupe hors vue ne doit rien laisser passer"
    );
}

#[test]
fn a_scissor_is_clamped_to_the_target() {
    use raster_render::{Camera, SpriteBatch};

    let camera = Camera::new(320, 180);
    // Un panneau qui deborde largement : wgpu refuse un ciseau hors cible.
    let large = IRect::from_rect(Rect::new(-500.0, -500.0, 2000.0, 2000.0));
    let (x, y, w, h) = SpriteBatch::scissor(large, camera, (320, 180)).unwrap();

    assert!(x + w <= 320, "le ciseau deborde en x : {x} + {w}");
    assert!(y + h <= 180, "le ciseau deborde en y : {y} + {h}");
}

/// Le regroupement en tranches : un appel de dessin par texture et par
/// decoupe.
mod runs {
    use super::*;
    use raster_render::SpriteBatch;

    fn clip(x: i32) -> Option<IRect> {
        Some(IRect::from_rect(Rect::new(x as f32, 0.0, 10.0, 10.0)))
    }

    #[test]
    fn sprites_sharing_a_texture_and_a_clip_form_one_run() {
        let mut runs = Vec::new();
        for i in 0..3 {
            SpriteBatch::extend_run(&mut runs, 0, clip(0), i);
        }

        assert_eq!(runs.len(), 1, "trois sprites identiques font un appel");
        assert_eq!(runs[0].3, 3, "les trois doivent etre comptes");
    }

    #[test]
    fn a_different_texture_opens_a_new_run() {
        let mut runs = Vec::new();
        SpriteBatch::extend_run(&mut runs, 0, None, 0);
        SpriteBatch::extend_run(&mut runs, 1, None, 1);

        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn a_different_clip_opens_a_new_run() {
        // Sans cela, un panneau deborderait sur son voisin : les deux
        // partageraient un appel, donc un seul ciseau.
        let mut runs = Vec::new();
        SpriteBatch::extend_run(&mut runs, 0, clip(0), 0);
        SpriteBatch::extend_run(&mut runs, 0, clip(100), 1);

        assert_eq!(runs.len(), 2, "deux decoupes doivent faire deux appels");
        assert_eq!(runs[0].1, clip(0));
        assert_eq!(runs[1].1, clip(100));
    }

    #[test]
    fn losing_a_clip_opens_a_new_run() {
        let mut runs = Vec::new();
        SpriteBatch::extend_run(&mut runs, 0, clip(0), 0);
        SpriteBatch::extend_run(&mut runs, 0, None, 1);

        assert_eq!(runs.len(), 2, "sortir d'une decoupe change d'appel");
    }

    #[test]
    fn a_run_records_where_it_starts() {
        let mut runs = Vec::new();
        SpriteBatch::extend_run(&mut runs, 0, None, 0);
        SpriteBatch::extend_run(&mut runs, 1, None, 7);

        assert_eq!(runs[1].2, 7, "la seconde tranche commence a 7");
    }
}
