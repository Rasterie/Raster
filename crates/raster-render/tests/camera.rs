//! La camera et la geometrie des sprites se testent sans GPU. Le rendu
//! lui-meme demande un adaptateur, qui n'existe pas sur tous les runners de CI.

use raster_math::Vec2;
use raster_render::{Camera, Colour, Layer, SpriteDraw};

const EPS: f32 = 1e-4;

#[test]
fn le_centre_du_monde_est_au_centre_de_l_ecran() {
    let camera = Camera::new(320, 180);
    let m = camera.view_projection();

    assert!(m.transform_point(Vec2::ZERO).approx_eq(Vec2::ZERO, EPS));
}

/// L'espace de clip va de -1 a 1 et Y monte, alors que le moteur compte les
/// pixels avec Y qui descend.
#[test]
fn les_bords_tombent_sur_les_limites_du_clip() {
    let camera = Camera::new(320, 180);
    let m = camera.view_projection();

    assert!(
        m.transform_point(Vec2::new(160.0, 0.0))
            .approx_eq(Vec2::new(1.0, 0.0), EPS)
    );
    assert!(
        m.transform_point(Vec2::new(-160.0, 0.0))
            .approx_eq(Vec2::new(-1.0, 0.0), EPS)
    );
    // Y negatif dans le monde = haut de l'ecran = +1 en clip.
    assert!(
        m.transform_point(Vec2::new(0.0, -90.0))
            .approx_eq(Vec2::new(0.0, 1.0), EPS)
    );
    assert!(
        m.transform_point(Vec2::new(0.0, 90.0))
            .approx_eq(Vec2::new(0.0, -1.0), EPS)
    );
}

#[test]
fn deplacer_la_camera_deplace_le_monde_en_sens_inverse() {
    let mut camera = Camera::new(320, 180);
    camera.position = Vec2::new(100.0, 50.0);

    let m = camera.view_projection();
    // Le point sur lequel la camera est centree revient au centre de l'ecran.
    assert!(
        m.transform_point(Vec2::new(100.0, 50.0))
            .approx_eq(Vec2::ZERO, EPS)
    );
}

#[test]
fn le_zoom_reduit_la_zone_visible() {
    let mut camera = Camera::new(320, 180);
    assert_eq!(camera.visible_area().size, Vec2::new(320.0, 180.0));

    camera.zoom = 2;
    assert_eq!(camera.visible_area().size, Vec2::new(160.0, 90.0));
}

#[test]
fn un_zoom_nul_est_traite_comme_un() {
    let mut camera = Camera::new(320, 180);
    camera.zoom = 0;

    assert_eq!(camera.visible_area().size, Vec2::new(320.0, 180.0));
    assert!(camera.view_projection().determinant().is_finite());
}

/// Un facteur de 3,7 rendrait certains pixels plus larges que d'autres. Il
/// n'existe aucune version acceptable de cela.
#[test]
fn le_facteur_d_agrandissement_est_toujours_entier() {
    let camera = Camera::new(320, 180);

    assert_eq!(camera.window_scale(Vec2::new(960.0, 540.0)), 3);
    assert_eq!(
        camera.window_scale(Vec2::new(1000.0, 600.0)),
        3,
        "1000/320 = 3,1"
    );
    assert_eq!(camera.window_scale(Vec2::new(640.0, 400.0)), 2);
}

#[test]
fn le_facteur_suit_la_dimension_la_plus_contraignante() {
    let camera = Camera::new(320, 180);
    // Large mais peu haut : la hauteur limite.
    assert_eq!(camera.window_scale(Vec2::new(2000.0, 200.0)), 1);
}

/// Une fenetre plus petite que la resolution interne ne doit pas donner un
/// facteur nul, qui ferait disparaitre l'image.
#[test]
fn une_fenetre_trop_petite_garde_un_facteur_de_un() {
    let camera = Camera::new(320, 180);
    assert_eq!(camera.window_scale(Vec2::new(100.0, 50.0)), 1);
}

#[test]
fn l_image_est_centree_avec_des_bandes() {
    let camera = Camera::new(320, 180);
    let viewport = camera.viewport(Vec2::new(1000.0, 600.0));

    // Facteur 3 : 960x540 dans une fenetre de 1000x600.
    assert_eq!(viewport.size, Vec2::new(960.0, 540.0));
    assert_eq!(viewport.position, Vec2::new(20.0, 30.0));
}

#[test]
fn la_conversion_ecran_vers_monde_annule_les_bandes() {
    let camera = Camera::new(320, 180);
    let fenetre = Vec2::new(1000.0, 600.0);

    // Le centre de la fenetre correspond au centre de la vue.
    let centre = camera.screen_to_world(fenetre * 0.5, fenetre);
    assert!(centre.approx_eq(camera.position, EPS), "obtenu {centre}");

    // Le coin superieur gauche de l'image, une fois les bandes retirees.
    let viewport = camera.viewport(fenetre);
    let coin = camera.screen_to_world(viewport.position, fenetre);
    assert!(
        coin.approx_eq(Vec2::new(-160.0, -90.0), EPS),
        "obtenu {coin}"
    );
}

#[test]
fn le_suivi_atteint_sa_cible() {
    let mut camera = Camera::new(320, 180);
    let cible = Vec2::new(100.0, 50.0);

    for _ in 0..300 {
        camera.follow(cible, 0.1, 1.0 / 60.0);
    }
    assert!(
        camera.position.approx_eq(cible, 0.01),
        "obtenu {}",
        camera.position
    );
}

#[test]
fn un_lissage_nul_colle_immediatement_a_la_cible() {
    let mut camera = Camera::new(320, 180);
    camera.follow(Vec2::new(100.0, 50.0), 0.0, 1.0 / 60.0);
    assert_eq!(camera.position, Vec2::new(100.0, 50.0));
}

/// Un `lerp` naif par frame suivrait plus vite sur une machine plus rapide.
#[test]
fn le_suivi_ne_depend_pas_de_la_frequence_d_images() {
    let cible = Vec2::new(100.0, 0.0);

    let mut lente = Camera::new(320, 180);
    for _ in 0..30 {
        lente.follow(cible, 0.2, 1.0 / 30.0);
    }

    let mut rapide = Camera::new(320, 180);
    for _ in 0..240 {
        rapide.follow(cible, 0.2, 1.0 / 240.0);
    }

    // Une seconde de suivi dans les deux cas : meme resultat.
    assert!(
        lente.position.approx_eq(rapide.position, 0.5),
        "30fps donne {} et 240fps donne {}",
        lente.position,
        rapide.position
    );
}

#[test]
fn un_sprite_couvre_sa_taille() {
    let sprite = SpriteDraw::new(Vec2::new(10.0, 20.0), Vec2::splat(16.0));
    let bounds = sprite.bounds();

    assert_eq!(bounds.position, Vec2::new(10.0, 20.0));
    assert_eq!(bounds.size, Vec2::splat(16.0));
}

#[test]
fn les_couches_s_ordonnent_du_fond_vers_l_avant() {
    assert!(Layer::BACKGROUND < Layer::DEFAULT);
    assert!(Layer::DEFAULT < Layer::FOREGROUND);
    assert!(Layer::FOREGROUND < Layer::UI);
}

#[test]
fn une_couleur_se_convertit_en_tableau() {
    assert_eq!(Colour::WHITE.to_array(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(Colour::TRANSPARENT.to_array(), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        Colour::rgba(0.5, 0.25, 0.125, 0.75).to_array(),
        [0.5, 0.25, 0.125, 0.75]
    );
}
