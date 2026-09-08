//! Verifie la traduction des evenements winit vers les entrees du moteur.
//!
//! La boucle elle-meme demande une fenetre, mais la traduction est du code pur
//! et c'est la que se trouvent les erreurs faciles.

use raster_input::{Action, Axis, Bindings, Input, Key};
use raster_math::Vec2;

/// Rejoue une sequence de touches et rend l'etat final, comme le ferait la
/// boucle de fenetre.
fn simuler(sequence: &[(Key, bool)]) -> Input {
    let mut input = Input::new(Bindings::default_layout());
    for (key, enfoncee) in sequence {
        if *enfoncee {
            input.key_down(*key);
        } else {
            input.key_up(*key);
        }
    }
    input.begin_frame(1.0 / 60.0);
    input
}

#[test]
fn la_chaine_complete_traduit_une_touche_en_mouvement() {
    let input = simuler(&[(Key::D, true)]);
    assert_eq!(input.axis(&Axis::HORIZONTAL), 1.0);

    // Ce que ferait le jeu avec cette valeur.
    let direction = Vec2::new(input.axis(&Axis::HORIZONTAL), input.axis(&Axis::VERTICAL));
    let deplacement = direction.normalized() * 90.0 * (1.0 / 60.0);
    assert!(
        deplacement.x > 0.0,
        "le joueur devrait aller vers la droite"
    );
    assert_eq!(deplacement.y, 0.0);
}

#[test]
fn les_touches_de_deplacement_sont_toutes_reconnues() {
    for (key, attendu) in [
        (Key::A, -1.0),
        (Key::Left, -1.0),
        (Key::D, 1.0),
        (Key::Right, 1.0),
    ] {
        let input = simuler(&[(key, true)]);
        assert_eq!(
            input.axis(&Axis::HORIZONTAL),
            attendu,
            "{key:?} devrait donner {attendu}"
        );
    }
}

#[test]
fn espace_declenche_le_saut() {
    let input = simuler(&[(Key::Space, true)]);
    assert!(input.pressed(&Action::JUMP));
    assert!(input.held(&Action::JUMP));
}

#[test]
fn echap_declenche_la_pause() {
    let input = simuler(&[(Key::Escape, true)]);
    assert!(input.pressed(&Action::PAUSE));
}

#[test]
fn une_diagonale_reste_de_norme_un() {
    let input = simuler(&[(Key::D, true), (Key::S, true)]);
    let direction = input.direction();

    assert!((direction.length() - 1.0).abs() < 1e-5);
    assert!(direction.x > 0.0 && direction.y > 0.0);
}
