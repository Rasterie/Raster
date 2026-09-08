//! L'agrandissement entier est la regle qui distingue un moteur fait pour le
//! pixel art d'un moteur qui l'accepte : sans elle, certains pixels sont plus
//! larges que d'autres.
//!
//! La cible de rendu demande un GPU, mais son calcul d'echelle est du code pur
//! et c'est la que se trouvent les erreurs.

use raster_math::Vec2;
use raster_render::Camera;

/// Reproduit le calcul de `RenderTarget::scale`, qui ne peut pas etre appele
/// sans GPU. Les deux doivent rester identiques — le test ci-dessous le
/// verifie contre la camera, qui expose le meme calcul.
fn facteur(resolution: (u32, u32), fenetre: (f32, f32)) -> u32 {
    let x = (fenetre.0 / resolution.0 as f32).floor() as u32;
    let y = (fenetre.1 / resolution.1 as f32).floor() as u32;
    x.min(y).max(1)
}

#[test]
fn la_camera_et_la_cible_calculent_le_meme_facteur() {
    let camera = Camera::new(320, 180);

    for fenetre in [
        (960.0, 540.0),
        (1000.0, 600.0),
        (1280.0, 720.0),
        (100.0, 50.0),
    ] {
        assert_eq!(
            camera.window_scale(Vec2::new(fenetre.0, fenetre.1)),
            facteur((320, 180), fenetre),
            "desaccord pour une fenetre {fenetre:?}"
        );
    }
}

/// Le defaut que la cible de rendu corrige : dessiner directement dans une
/// fenetre de 1000 pixels de large pour un jeu de 320 donne une echelle de
/// 3,125, donc un pixel sur huit plus large que ses voisins.
#[test]
fn une_fenetre_non_multiple_donne_un_facteur_entier() {
    let camera = Camera::new(320, 180);
    let fenetre = Vec2::new(1000.0, 600.0);

    let entier = camera.window_scale(fenetre);
    let reel = fenetre.x / 320.0;

    assert_eq!(entier, 3);
    assert!(reel > 3.0, "l'echelle reelle vaut {reel}, d'ou les bandes");
}

#[test]
fn les_bandes_absorbent_la_difference() {
    let camera = Camera::new(320, 180);
    let fenetre = Vec2::new(1000.0, 600.0);
    let viewport = camera.viewport(fenetre);

    // 320x180 agrandi trois fois tient dans 1000x600 avec des marges.
    assert_eq!(viewport.size, Vec2::new(960.0, 540.0));
    assert_eq!(viewport.position, Vec2::new(20.0, 30.0));

    // Les marges sont symetriques.
    let marge_droite = fenetre.x - viewport.right();
    let marge_bas = fenetre.y - viewport.bottom();
    assert_eq!(marge_droite, viewport.position.x);
    assert_eq!(marge_bas, viewport.position.y);
}

#[test]
fn la_position_des_bandes_tombe_sur_un_pixel_entier() {
    let camera = Camera::new(320, 180);

    // Une largeur impaire donnerait une marge fractionnaire sans arrondi.
    let viewport = camera.viewport(Vec2::new(1001.0, 601.0));
    assert_eq!(viewport.position.x.fract(), 0.0);
    assert_eq!(viewport.position.y.fract(), 0.0);
}

#[test]
fn une_fenetre_plus_petite_que_le_jeu_garde_un_facteur_de_un() {
    let camera = Camera::new(320, 180);
    assert_eq!(camera.window_scale(Vec2::new(100.0, 50.0)), 1);
}

#[test]
fn le_facteur_suit_la_dimension_la_plus_contrainte() {
    let camera = Camera::new(320, 180);
    // Tres large mais peu haut : c'est la hauteur qui limite.
    assert_eq!(camera.window_scale(Vec2::new(3200.0, 200.0)), 1);
    // Et l'inverse.
    assert_eq!(camera.window_scale(Vec2::new(400.0, 1800.0)), 1);
}
