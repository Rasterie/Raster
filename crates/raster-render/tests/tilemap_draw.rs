//! Le rendu de tuiles : ce qui se calcule sans GPU, c'est-a-dire le choix de la
//! variante et l'elimination hors champ.

use raster_render::tile_variant;

/// La disposition d'un tileset depend de cet ordre : un artiste dessine les
/// variantes dans l'ordre des indices, et le moteur doit les y retrouver.
#[test]
fn les_variantes_couvrent_zero_a_quarante_six() {
    let mut vues = std::collections::BTreeSet::new();
    for masque in 0..=255u8 {
        vues.insert(tile_variant(masque));
    }

    assert_eq!(vues.len(), 47);
    assert_eq!(*vues.first().unwrap(), 0);
    assert_eq!(*vues.last().unwrap(), 46);
}

#[test]
fn une_tuile_isolee_prend_la_premiere_variante() {
    assert_eq!(tile_variant(0), 0);
}

/// Deux masques qui ne different que par une diagonale sans ses cotes doivent
/// donner la meme variante : c'est la reduction qui ramene 256 a 47.
#[test]
fn une_diagonale_orpheline_ne_change_pas_la_variante() {
    const UP_RIGHT: u8 = 1 << 1;
    const DOWN_LEFT: u8 = 1 << 5;

    assert_eq!(tile_variant(0), tile_variant(UP_RIGHT));
    assert_eq!(tile_variant(0), tile_variant(DOWN_LEFT));
    assert_eq!(tile_variant(0), tile_variant(UP_RIGHT | DOWN_LEFT));
}

#[test]
fn le_calcul_est_stable() {
    for masque in 0..=255u8 {
        assert_eq!(tile_variant(masque), tile_variant(masque));
    }
}
