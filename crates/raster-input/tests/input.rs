use raster_input::{Action, Axis, Binding, Bindings, Grace, Input, Key, MouseButton};
use raster_math::Vec2;

const FRAME: f32 = 1.0 / 60.0;

fn input() -> Input {
    Input::new(Bindings::default_layout())
}

/// Une frame avec les touches deja posees : `begin_frame` echantillonne l'etat
/// courant, donc l'ordre compte.
fn frame(input: &mut Input) {
    input.begin_frame(FRAME);
}

#[test]
fn une_touche_enfoncee_declenche_son_action() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);

    assert!(input.held(&Action::JUMP));
    assert!(input.pressed(&Action::JUMP));
    assert!(!input.released(&Action::JUMP));
}

#[test]
fn une_pression_ne_dure_qu_une_frame() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);
    assert!(input.pressed(&Action::JUMP));

    frame(&mut input);
    assert!(
        !input.pressed(&Action::JUMP),
        "la pression a dure deux frames"
    );
    assert!(
        input.held(&Action::JUMP),
        "la touche est pourtant toujours enfoncee"
    );
}

#[test]
fn relacher_est_signale_une_fois() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);

    input.key_up(Key::Space);
    frame(&mut input);
    assert!(input.released(&Action::JUMP));
    assert!(!input.held(&Action::JUMP));

    frame(&mut input);
    assert!(
        !input.released(&Action::JUMP),
        "le relachement a dure deux frames"
    );
}

/// Une action liee a plusieurs touches se declenche par n'importe laquelle.
#[test]
fn plusieurs_liaisons_declenchent_la_meme_action() {
    let mut input = input();
    let gauche = Action("left");

    input.key_down(Key::A);
    frame(&mut input);
    assert!(input.held(&gauche));

    input.key_up(Key::A);
    input.key_down(Key::Left);
    frame(&mut input);
    assert!(
        input.held(&gauche),
        "la fleche gauche devrait aussi marcher"
    );
}

#[test]
fn un_axe_va_de_moins_un_a_un() {
    let mut input = input();

    frame(&mut input);
    assert_eq!(input.axis(&Axis::HORIZONTAL), 0.0);

    input.key_down(Key::D);
    frame(&mut input);
    assert_eq!(input.axis(&Axis::HORIZONTAL), 1.0);

    input.key_down(Key::A);
    frame(&mut input);
    assert_eq!(
        input.axis(&Axis::HORIZONTAL),
        0.0,
        "les deux sens s'annulent"
    );

    input.key_up(Key::D);
    frame(&mut input);
    assert_eq!(input.axis(&Axis::HORIZONTAL), -1.0);
}

/// Sans normalisation, aller en diagonale serait 41 % plus rapide.
#[test]
fn une_diagonale_ne_va_pas_plus_vite_qu_une_ligne_droite() {
    let mut input = input();
    input.key_down(Key::D);
    input.key_down(Key::S);
    frame(&mut input);

    let diagonale = input.direction();
    assert!(
        (diagonale.length() - 1.0).abs() < 1e-5,
        "longueur {}",
        diagonale.length()
    );
}

#[test]
fn une_direction_nulle_reste_nulle() {
    let mut input = input();
    frame(&mut input);
    assert_eq!(input.direction(), Vec2::ZERO);
}

// --- memorisation des pressions ---------------------------------------------

/// Le point de la memorisation : un saut demande juste avant l'atterrissage
/// doit quand meme se declencher.
#[test]
fn une_pression_reste_disponible_pendant_la_fenetre() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);
    input.key_up(Key::Space);

    // Quatre frames plus tard, la pression compte toujours.
    for _ in 0..4 {
        frame(&mut input);
    }
    assert!(input.buffered(&Action::JUMP));
}

#[test]
fn une_pression_trop_ancienne_expire() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);
    input.key_up(Key::Space);

    // Bien au-dela de la fenetre par defaut de 0,12 s.
    for _ in 0..20 {
        frame(&mut input);
    }
    assert!(!input.buffered(&Action::JUMP));
}

/// Un saut qui a consomme sa pression memorisee ne doit pas resauter a la
/// frame suivante avec la meme.
#[test]
fn une_pression_memorisee_ne_se_consomme_qu_une_fois() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);
    input.key_up(Key::Space);

    assert!(input.consume_buffered(&Action::JUMP));
    assert!(
        !input.consume_buffered(&Action::JUMP),
        "consommee deux fois"
    );
    assert!(!input.buffered(&Action::JUMP));
}

#[test]
fn lire_la_memorisation_ne_la_consomme_pas() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);

    assert!(input.buffered(&Action::JUMP));
    assert!(
        input.buffered(&Action::JUMP),
        "la lecture a consomme la pression"
    );
}

#[test]
fn la_fenetre_de_memorisation_est_reglable() {
    let mut input = input();
    input.set_buffer_window(0.0);

    input.key_down(Key::Space);
    frame(&mut input);
    input.key_up(Key::Space);
    frame(&mut input);

    assert!(
        !input.buffered(&Action::JUMP),
        "une fenetre nulle memorise quand meme"
    );
}

// --- coyote time -------------------------------------------------------------

#[test]
fn le_delai_de_grace_est_actif_pendant_la_condition() {
    let mut grace = Grace::new(0.1);
    grace.update(true, FRAME);
    assert!(grace.active());
}

/// Sauter juste apres avoir quitte une plateforme doit encore fonctionner.
#[test]
fn le_delai_de_grace_survit_a_la_condition() {
    let mut grace = Grace::new(0.1);
    grace.update(true, FRAME);

    // Trois frames en l'air : encore dans la fenetre.
    for _ in 0..3 {
        grace.update(false, FRAME);
    }
    assert!(grace.active());
}

#[test]
fn le_delai_de_grace_finit_par_expirer() {
    let mut grace = Grace::new(0.1);
    grace.update(true, FRAME);

    for _ in 0..20 {
        grace.update(false, FRAME);
    }
    assert!(!grace.active());
}

#[test]
fn un_delai_de_grace_consomme_ne_sert_pas_deux_fois() {
    let mut grace = Grace::new(0.1);
    grace.update(true, FRAME);

    assert!(grace.active());
    grace.consume();
    assert!(!grace.active(), "utilisable apres consommation");
}

/// Sans etat initial expire, un personnage pourrait sauter au tout premier
/// instant sans jamais avoir touche le sol.
#[test]
fn un_delai_de_grace_neuf_est_inactif() {
    assert!(!Grace::new(0.1).active());
    assert!(!Grace::default().active());
}

#[test]
fn la_condition_reactive_le_delai_apres_expiration() {
    let mut grace = Grace::new(0.1);
    grace.update(true, FRAME);
    grace.consume();

    grace.update(true, FRAME);
    assert!(grace.active(), "toucher le sol devrait rearmer");
}

// --- souris ------------------------------------------------------------------

#[test]
fn la_souris_rapporte_sa_position_et_son_deplacement() {
    let mut input = input();

    input.set_mouse_position(Vec2::new(100.0, 50.0));
    assert_eq!(input.mouse_position(), Vec2::new(100.0, 50.0));

    input.set_mouse_position(Vec2::new(110.0, 50.0));
    assert_eq!(input.mouse_delta(), Vec2::new(110.0, 50.0));
}

#[test]
fn le_deplacement_se_remet_a_zero_chaque_frame() {
    let mut input = input();
    input.set_mouse_position(Vec2::new(10.0, 10.0));
    frame(&mut input);

    assert_eq!(input.mouse_delta(), Vec2::ZERO);
}

#[test]
fn un_bouton_de_souris_declenche_son_action() {
    let mut input = input();
    input.mouse_down(MouseButton::Left);
    frame(&mut input);

    assert!(input.held(&Action::ATTACK));
    assert!(input.pressed(&Action::ATTACK));
}

#[test]
fn le_defilement_s_accumule_puis_se_remet_a_zero() {
    let mut input = input();
    input.add_scroll(1.0);
    input.add_scroll(0.5);
    assert_eq!(input.scroll_delta(), 1.5);

    frame(&mut input);
    assert_eq!(input.scroll_delta(), 0.0);
}

/// Une touche maintenue pendant un changement de fenetre resterait enfoncee
/// pour toujours, l'evenement de relachement partant ailleurs.
#[test]
fn perdre_le_focus_relache_tout() {
    let mut input = input();
    input.key_down(Key::Space);
    input.key_down(Key::D);
    frame(&mut input);
    assert!(input.held(&Action::JUMP));

    input.release_all();
    frame(&mut input);

    assert!(!input.held(&Action::JUMP));
    assert_eq!(input.axis(&Axis::HORIZONTAL), 0.0);
}

// --- liaisons -----------------------------------------------------------------

#[test]
fn une_action_peut_etre_reliee_autrement() {
    let mut input = input();
    input
        .bindings_mut()
        .rebind(Action::JUMP, vec![Binding::Key(Key::Z)]);

    input.key_down(Key::Space);
    frame(&mut input);
    assert!(
        !input.held(&Action::JUMP),
        "l'ancienne liaison marche encore"
    );

    input.key_up(Key::Space);
    input.key_down(Key::Z);
    frame(&mut input);
    assert!(input.held(&Action::JUMP));
}

#[test]
fn une_action_sans_liaison_ne_se_declenche_jamais() {
    let mut input = input();
    input.bindings_mut().clear(&Action::JUMP);

    input.key_down(Key::Space);
    frame(&mut input);

    assert!(!input.held(&Action::JUMP));
    assert!(!input.pressed(&Action::JUMP));
}

#[test]
fn une_action_inconnue_est_inerte() {
    let mut input = input();
    input.key_down(Key::Space);
    frame(&mut input);

    let inventee = Action("cette action n'existe pas");
    assert!(!input.held(&inventee));
    assert!(!input.pressed(&inventee));
    assert!(!input.buffered(&inventee));
}
