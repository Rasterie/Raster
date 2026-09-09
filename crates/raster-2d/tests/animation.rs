use raster_2d::{Animation, Animator, Repeat, StateMachine};

const FRAME: f32 = 1.0 / 60.0;

/// Quatre images a 10 par seconde : chacune dure 0,1 s.
fn marche() -> Animation {
    Animation::new([0, 1, 2, 3], 10.0)
}

#[test]
fn une_animation_commence_sur_sa_premiere_image() {
    let anim = marche();
    let player = Animator::new();

    assert_eq!(player.frame(), 0);
    assert_eq!(player.index(&anim), 0);
}

#[test]
fn le_temps_fait_avancer_les_images() {
    let anim = marche();
    let mut player = Animator::new();

    // Six frames de 1/60 s font 0,1 s, soit une image.
    for _ in 0..6 {
        player.advance(&anim, FRAME);
    }
    assert_eq!(player.frame(), 1);
}

#[test]
fn une_animation_en_boucle_revient_au_debut() {
    let anim = marche();
    let mut player = Animator::new();

    // Un cycle complet : quatre images de 0,1 s.
    player.advance(&anim, 0.4);

    assert_eq!(player.frame(), 0, "la boucle n'a pas repris au debut");
    assert!(!player.finished());

    // Et elle continue de tourner.
    player.advance(&anim, 0.1);
    assert_eq!(player.frame(), 1);
}

#[test]
fn une_animation_unique_s_arrete_sur_la_derniere_image() {
    let anim = marche().with_repeat(Repeat::Once);
    let mut player = Animator::new();

    for _ in 0..60 {
        player.advance(&anim, FRAME);
    }

    assert_eq!(player.frame(), 3, "ne s'est pas arretee sur la derniere");
    assert!(player.finished());
}

#[test]
fn le_ping_pong_repart_en_arriere() {
    let anim = Animation::new([0, 1, 2], 10.0).with_repeat(Repeat::PingPong);
    let mut player = Animator::new();

    let mut vues = vec![player.frame()];
    for _ in 0..30 {
        let avant = player.frame();
        player.advance(&anim, FRAME);
        if player.frame() != avant {
            vues.push(player.frame());
        }
    }

    // Aller-retour : 0 1 2 1 0 1 2...
    assert_eq!(&vues[..5], &[0, 1, 2, 1, 0], "obtenu {vues:?}");
}

#[test]
fn un_ping_pong_d_une_seule_image_ne_bloque_pas() {
    let anim = Animation::new([7], 10.0).with_repeat(Repeat::PingPong);
    let mut player = Animator::new();

    for _ in 0..30 {
        player.advance(&anim, FRAME);
    }
    assert_eq!(player.index(&anim), 7);
}

/// Une image plus courte que le pas de temps doit quand meme etre traversee :
/// sinon une animation rapide se fige sur sa premiere image.
#[test]
fn un_grand_pas_traverse_plusieurs_images() {
    let anim = Animation::new([0, 1, 2, 3], 60.0).with_repeat(Repeat::Once);
    let mut player = Animator::new();

    // Un seul pas de 0,1 s pour des images de 1/60 s.
    player.advance(&anim, 0.1);

    assert!(player.finished(), "n'a pas traverse les images courtes");
}

#[test]
fn la_vitesse_change_la_cadence() {
    let anim = marche();

    let mut normal = Animator::new();
    let mut rapide = Animator::new();
    rapide.set_speed(2.0);

    for _ in 0..6 {
        normal.advance(&anim, FRAME);
        rapide.advance(&anim, FRAME);
    }

    assert_eq!(normal.frame(), 1);
    assert_eq!(rapide.frame(), 2, "la vitesse double n'a pas ete appliquee");
}

#[test]
fn une_vitesse_nulle_fige_l_animation() {
    let anim = marche();
    let mut player = Animator::new();
    player.set_speed(0.0);

    for _ in 0..60 {
        player.advance(&anim, FRAME);
    }
    assert_eq!(player.frame(), 0);
}

#[test]
fn redemarrer_revient_au_debut() {
    let anim = marche().with_repeat(Repeat::Once);
    let mut player = Animator::new();

    for _ in 0..60 {
        player.advance(&anim, FRAME);
    }
    assert!(player.finished());

    player.restart();
    assert_eq!(player.frame(), 0);
    assert!(!player.finished());
}

// --- evenements -----------------------------------------------------------------

#[test]
fn un_evenement_se_declenche_en_entrant_dans_son_image() {
    let anim = marche().with_event(2, "pas");
    let mut player = Animator::new();

    let mut declenches = Vec::new();
    for _ in 0..18 {
        declenches.extend(player.advance(&anim, FRAME).iter().map(|s| s.to_string()));
    }

    assert_eq!(declenches, vec!["pas"], "obtenu {declenches:?}");
}

#[test]
fn plusieurs_evenements_tiennent_sur_une_image() {
    let anim = marche().with_event(1, "pas").with_event(1, "poussiere");
    let mut player = Animator::new();

    let mut declenches = Vec::new();
    for _ in 0..8 {
        declenches.extend(player.advance(&anim, FRAME).iter().map(|s| s.to_string()));
    }

    assert_eq!(declenches.len(), 2, "obtenu {declenches:?}");
    assert!(declenches.contains(&"pas".to_string()));
}

/// Un pas de temps qui saute par-dessus une image ne doit pas en perdre
/// l'evenement : une hitbox manquee est un coup qui ne touche pas.
#[test]
fn un_grand_pas_ne_perd_pas_d_evenement() {
    let anim = Animation::new([0, 1, 2, 3], 60.0)
        .with_repeat(Repeat::Once)
        .with_event(1, "a")
        .with_event(2, "b")
        .with_event(3, "c");
    let mut player = Animator::new();

    let declenches: Vec<String> = player
        .advance(&anim, 0.1)
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(declenches, vec!["a", "b", "c"], "obtenu {declenches:?}");
}

#[test]
fn une_boucle_redeclenche_ses_evenements() {
    let anim = marche().with_event(0, "cycle");
    let mut player = Animator::new();

    let mut compte = 0;
    for _ in 0..60 {
        compte += player.advance(&anim, FRAME).len();
    }

    assert!(
        compte >= 2,
        "un seul declenchement en deux cycles : {compte}"
    );
}

// --- machine a etats ---------------------------------------------------------------

fn machine() -> StateMachine {
    let mut m = StateMachine::new();
    m.add("repos", Animation::new([0], 10.0));
    m.add("marche", marche());
    m.add("saut", Animation::new([8], 10.0).with_repeat(Repeat::Once));
    m.add(
        "attaque",
        Animation::new([10, 11, 12], 20.0).with_repeat(Repeat::Once),
    );

    m.transition("repos", "marche", "bouge");
    m.transition("marche", "repos", "immobile");
    m.transition_any("saut", "en l'air");
    m.transition_after("attaque", "repos", "termine");
    m
}

#[test]
fn le_premier_etat_ajoute_devient_courant() {
    let m = machine();
    assert_eq!(m.state(), "repos");
}

#[test]
fn une_condition_declenche_sa_transition() {
    let mut m = machine();
    m.update(&["bouge"], FRAME);
    assert_eq!(m.state(), "marche");
}

#[test]
fn une_condition_absente_ne_change_rien() {
    let mut m = machine();
    m.update(&[], FRAME);
    assert_eq!(m.state(), "repos");
}

#[test]
fn une_transition_depuis_n_importe_ou_s_applique() {
    let mut m = machine();
    m.update(&["bouge"], FRAME);
    assert_eq!(m.state(), "marche");

    m.update(&["en l'air"], FRAME);
    assert_eq!(m.state(), "saut", "la transition globale n'a pas joue");
}

#[test]
fn changer_d_etat_redemarre_l_animation() {
    let mut m = machine();
    m.update(&["bouge"], FRAME);

    for _ in 0..12 {
        m.update(&["bouge"], FRAME);
    }
    let avance = m.index();

    m.update(&["immobile"], FRAME);
    m.update(&["bouge"], FRAME);

    assert_ne!(m.index(), avance, "l'animation n'a pas redemarre");
}

/// Une attaque doit aller au bout avant de rendre la main.
#[test]
fn une_transition_differee_attend_la_fin() {
    let mut m = machine();
    m.force("attaque");

    // La condition tient des la premiere frame, mais l'animation dure.
    m.update(&["termine"], FRAME);
    assert_eq!(m.state(), "attaque", "a coupe l'animation");

    for _ in 0..30 {
        m.update(&["termine"], FRAME);
    }
    assert_eq!(m.state(), "repos", "n'a jamais rendu la main");
}

#[test]
fn forcer_un_etat_ignore_les_transitions() {
    let mut m = machine();
    m.force("saut");
    assert_eq!(m.state(), "saut");
}

#[test]
fn forcer_un_etat_inconnu_ne_fait_rien() {
    let mut m = machine();
    m.force("inexistant");
    assert_eq!(m.state(), "repos");
}

#[test]
fn la_machine_remonte_les_evenements() {
    let mut m = StateMachine::new();
    m.add("marche", marche().with_event(1, "pas"));

    let mut declenches = Vec::new();
    for _ in 0..8 {
        declenches.extend(m.update(&[], FRAME));
    }

    assert_eq!(declenches, vec!["pas"]);
}

#[test]
fn une_machine_vide_ne_plante_pas() {
    let mut m = StateMachine::new();
    assert!(m.update(&["quelque chose"], FRAME).is_empty());
    assert_eq!(m.index(), 0);
}
