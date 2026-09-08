//! La boucle a pas fixe existe pour une raison precise : sans elle, la hauteur
//! d'un saut depend de la machine sur laquelle le jeu tourne.

use raster_core::{FrameLoop, Time};

#[test]
fn une_frame_exacte_donne_un_pas() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    assert_eq!(boucle.advance(1.0 / 60.0).count(), 1);
}

#[test]
fn une_frame_lente_rattrape_plusieurs_pas() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    // Une frame de 50 ms doit trois pas de 16,67 ms.
    assert_eq!(boucle.advance(0.05).count(), 3);
}

#[test]
fn une_frame_rapide_ne_donne_pas_toujours_un_pas() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    // A 240 images/s, seule une frame sur quatre franchit le seuil.
    let total: u32 = (0..4).map(|_| boucle.advance(1.0 / 240.0).count()).sum();
    assert_eq!(total, 1);
}

/// Trois pas de 16,667 ms retires de 50 ms laissent, en f32, 3,7 ns de moins
/// qu'un pas : sans tolerance, le troisieme ne se declenche jamais et le jeu
/// tourne au ralenti a certaines frequences.
#[test]
fn l_imprecision_des_flottants_ne_fait_pas_perdre_de_pas() {
    // Vingt images par seconde : exactement trois pas par frame.
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    let pas: u32 = (0..20).map(|_| boucle.advance(1.0 / 20.0).count()).sum();

    assert_eq!(pas, 60, "une seconde a 20 images/s devrait donner 60 pas");
}

/// Le point de tout le mecanisme : la meme duree simulee, quelle que soit la
/// frequence d'images.
#[test]
fn le_nombre_de_pas_ne_depend_pas_de_la_frequence_d_images() {
    let simuler = |images_par_seconde: u32| {
        let mut boucle = FrameLoop::new(1.0 / 60.0);
        let dt = 1.0 / images_par_seconde as f32;
        (0..images_par_seconde)
            .map(|_| boucle.advance(dt).count())
            .sum::<u32>()
    };

    // Une seconde de jeu vaut soixante pas, a 30 comme a 240 images/s.
    for ips in [30, 60, 120, 144, 240] {
        let pas = simuler(ips);
        assert!(
            (59..=61).contains(&pas),
            "a {ips} images/s, une seconde a donne {pas} pas au lieu de 60"
        );
    }
}

/// Une frame tres longue — un point d'arret, une fenetre deplacee, un portable
/// qui se reveille — doit perdre du temps plutot que de figer le jeu en
/// essayant de le rattraper.
#[test]
fn une_frame_enorme_est_bornee() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    // Dix secondes d'un coup : sans borne, 600 pas.
    let pas = boucle.advance(10.0).count();
    assert!(pas <= FrameLoop::MAX_STEPS, "{pas} pas d'un coup");
}

#[test]
fn le_plafond_de_pas_est_reglable() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    boucle.set_max_steps(2);
    assert_eq!(boucle.advance(1.0).count(), 2);
}

/// Sans remise a zero du reliquat, la dette s'accumulerait et le jeu tournerait
/// au ralenti pour toujours apres une seule frame trop longue.
#[test]
fn le_temps_perdu_n_est_pas_reporte() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    boucle.advance(10.0);

    // La frame suivante est normale : elle ne doit pas rattraper la dette.
    assert_eq!(boucle.advance(1.0 / 60.0).count(), 1);
}

#[test]
fn l_interpolation_reste_entre_zero_et_un() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);

    for i in 1..100 {
        boucle.advance(0.001 * i as f32);
        let alpha = boucle.time().alpha;
        assert!((0.0..1.0).contains(&alpha), "alpha = {alpha}");
    }
}

#[test]
fn l_interpolation_progresse_dans_le_pas() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);

    // Un demi-pas : a mi-chemin.
    boucle.advance(1.0 / 120.0);
    assert!(
        (boucle.time().alpha - 0.5).abs() < 0.01,
        "alpha = {}",
        boucle.time().alpha
    );
}

// --- echelle de temps --------------------------------------------------------

#[test]
fn une_echelle_nulle_met_en_pause() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    boucle.set_scale(0.0);

    assert_eq!(boucle.advance(1.0 / 60.0).count(), 0);
    assert!(boucle.time().paused());
    assert_eq!(boucle.time().delta, 0.0);
}

/// Un menu de pause qui cesse de s'animer quand le jeu est en pause parait
/// casse : c'est pour cela que raw_delta existe.
#[test]
fn le_temps_brut_continue_pendant_la_pause() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    boucle.set_scale(0.0);
    boucle.advance(1.0 / 60.0);

    assert_eq!(boucle.time().delta, 0.0);
    assert!(
        boucle.time().raw_delta > 0.0,
        "le temps brut s'est arrete aussi"
    );
}

#[test]
fn un_ralenti_reduit_le_nombre_de_pas() {
    let mut moitie = FrameLoop::new(1.0 / 60.0);
    moitie.set_scale(0.5);

    let pas: u32 = (0..60).map(|_| moitie.advance(1.0 / 60.0).count()).sum();
    assert!(
        (29..=31).contains(&pas),
        "au ralenti, {pas} pas au lieu de 30"
    );
}

#[test]
fn une_echelle_negative_est_ramenee_a_zero() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    boucle.set_scale(-1.0);

    assert_eq!(boucle.time().scale, 0.0);
    assert!(boucle.time().elapsed >= 0.0, "le temps est remonte");
}

#[test]
fn le_temps_ecoule_suit_l_echelle() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    for _ in 0..60 {
        boucle.advance(1.0 / 60.0);
    }
    assert!((boucle.time().elapsed - 1.0).abs() < 0.01);

    boucle.set_scale(0.0);
    let fige = boucle.time().elapsed;
    for _ in 0..60 {
        boucle.advance(1.0 / 60.0);
    }
    assert_eq!(
        boucle.time().elapsed,
        fige,
        "le temps a avance pendant la pause"
    );
}

// --- iteration ---------------------------------------------------------------

#[test]
fn les_pas_s_iterent_avec_leur_duree() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    let pas = boucle.advance(0.05);

    let durees: Vec<f32> = pas.collect();
    assert_eq!(durees.len(), 3);
    assert!(durees.iter().all(|d| (*d - 1.0 / 60.0).abs() < 1e-6));
}

#[test]
fn une_frame_sans_pas_n_itere_rien() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    assert_eq!(boucle.advance(0.001).count(), 0);
    assert_eq!(boucle.advance(0.001).into_iter().count(), 0);
}

#[test]
fn un_pas_fixe_nul_ne_bloque_pas() {
    // Une division par un pas nul boucle indefiniment : la valeur est bornee.
    let mut boucle = FrameLoop::new(0.0);
    let pas = boucle.advance(1.0 / 60.0);
    assert!(pas.count() <= FrameLoop::MAX_STEPS);
}

#[test]
fn une_duree_negative_est_ignoree() {
    let mut boucle = FrameLoop::new(1.0 / 60.0);
    assert_eq!(boucle.advance(-1.0).count(), 0);
    assert_eq!(boucle.time().raw_delta, 0.0);
}

#[test]
fn un_temps_neuf_a_des_valeurs_saines() {
    let time = Time::default();
    assert_eq!(time.scale, 1.0);
    assert!(!time.paused());
    assert!(time.fixed_delta > 0.0);
}
