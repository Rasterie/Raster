//! L'autotuilage : 256 voisinages possibles, mais seulement 47 apparences
//! distinctes. C'est cette reduction qui rend le dessin d'un tileset faisable.

use raster_2d::{Collision, Neighbours, TileId, TileKind, Tilemap, Tileset, neighbours_of};
use raster_math::IVec2;
use std::collections::BTreeSet;

const PIERRE: TileId = TileId(1);
const TERRE: TileId = TileId(2);

fn carte() -> Tilemap {
    let mut set = Tileset::new(16);
    set.add(TileKind {
        name: "pierre".to_owned(),
        collision: Collision::Solid,
        autotile: true,
        ..TileKind::default()
    });
    set.add(TileKind {
        name: "terre".to_owned(),
        collision: Collision::Solid,
        autotile: true,
        ..TileKind::default()
    });
    Tilemap::new(set)
}

#[test]
fn une_tuile_isolee_n_a_aucun_voisin() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);

    assert_eq!(neighbours_of(&map, IVec2::ZERO), Neighbours(0));
}

#[test]
fn le_vide_n_a_pas_de_voisinage() {
    let map = carte();
    assert_eq!(neighbours_of(&map, IVec2::ZERO), Neighbours(0));
}

#[test]
fn les_directions_tombent_sur_les_bons_bits() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);

    map.set(IVec2::new(0, -1), PIERRE); // au-dessus
    assert!(neighbours_of(&map, IVec2::ZERO).has(Neighbours::UP));

    map.set(IVec2::new(1, 0), PIERRE); // a droite
    assert!(neighbours_of(&map, IVec2::ZERO).has(Neighbours::RIGHT));

    map.set(IVec2::new(0, 1), PIERRE); // en dessous
    assert!(neighbours_of(&map, IVec2::ZERO).has(Neighbours::DOWN));

    map.set(IVec2::new(-1, 0), PIERRE); // a gauche
    assert!(neighbours_of(&map, IVec2::ZERO).has(Neighbours::LEFT));
}

/// Une tuile de pierre se raccorde a la pierre, pas a la terre : sinon deux
/// materiaux differents fusionneraient visuellement.
#[test]
fn seules_les_tuiles_de_meme_type_se_raccordent() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);
    map.set(IVec2::new(1, 0), TERRE);

    assert!(!neighbours_of(&map, IVec2::ZERO).has(Neighbours::RIGHT));
}

/// La reduction qui donne 47 : un coin ne compte que si les deux cotes qui
/// l'encadrent sont remplis, sinon il est visible et ne change rien au dessin.
#[test]
fn une_diagonale_seule_ne_change_pas_la_variante() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);

    let isolee = neighbours_of(&map, IVec2::ZERO).variant();

    // Ajoute uniquement le voisin en haut a droite.
    map.set(IVec2::new(1, -1), PIERRE);
    let avec_diagonale = neighbours_of(&map, IVec2::ZERO).variant();

    assert_eq!(
        isolee, avec_diagonale,
        "une diagonale sans ses cotes a change la variante"
    );
}

#[test]
fn une_diagonale_entouree_change_la_variante() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);
    map.set(IVec2::new(0, -1), PIERRE); // haut
    map.set(IVec2::new(1, 0), PIERRE); // droite

    let sans_coin = neighbours_of(&map, IVec2::ZERO).variant();

    map.set(IVec2::new(1, -1), PIERRE); // le coin entre les deux
    let avec_coin = neighbours_of(&map, IVec2::ZERO).variant();

    assert_ne!(
        sans_coin, avec_coin,
        "le coin rempli devrait changer le dessin"
    );
}

/// Le chiffre canonique du bitmask a 8 directions. S'en ecarter signale une
/// erreur dans la reduction.
#[test]
fn il_existe_exactement_quarante_sept_variantes() {
    let mut vues = BTreeSet::new();

    for masque in 0..=255u8 {
        vues.insert(Neighbours(masque).variant());
    }

    assert_eq!(vues.len(), 47, "obtenu {} variantes", vues.len());
}

#[test]
fn les_variantes_sont_contigues_depuis_zero() {
    let mut vues = BTreeSet::new();
    for masque in 0..=255u8 {
        vues.insert(Neighbours(masque).variant());
    }

    let attendues: BTreeSet<u8> = (0..47).collect();
    assert_eq!(
        vues, attendues,
        "les indices ne couvrent pas 0..47 sans trou"
    );
}

#[test]
fn une_tuile_entouree_de_partout_a_une_variante_stable() {
    let mut map = carte();
    for y in -1..=1 {
        for x in -1..=1 {
            map.set(IVec2::new(x, y), PIERRE);
        }
    }

    let pleine = neighbours_of(&map, IVec2::ZERO);
    assert_eq!(pleine.0, 0xFF, "les huit voisins devraient etre presents");

    // Deux lectures successives donnent le meme resultat.
    assert_eq!(pleine.variant(), neighbours_of(&map, IVec2::ZERO).variant());
}

/// Le voisinage traverse les frontieres de chunks : sans cela, une couture
/// apparaitrait tous les 32 tuiles.
#[test]
fn le_voisinage_traverse_les_chunks() {
    use raster_2d::Chunk;

    let mut map = carte();
    let bord = IVec2::new(Chunk::SIZE - 1, 5);
    let apres = IVec2::new(Chunk::SIZE, 5);

    map.set(bord, PIERRE);
    map.set(apres, PIERRE);

    assert!(
        neighbours_of(&map, bord).has(Neighbours::RIGHT),
        "la tuile du chunk voisin n'a pas ete vue"
    );
}

#[test]
fn seuls_les_types_marques_sont_autotuiles() {
    let mut set = Tileset::new(16);
    let avec = set.add(TileKind {
        name: "avec".to_owned(),
        autotile: true,
        ..TileKind::default()
    });
    let sans = set.add(TileKind {
        name: "sans".to_owned(),
        autotile: false,
        ..TileKind::default()
    });
    let map = Tilemap::new(set);

    assert!(raster_2d::should_autotile(&map, avec));
    assert!(!raster_2d::should_autotile(&map, sans));
}
