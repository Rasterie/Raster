use raster_2d::{Chunk, Collision, TileId, TileKind, Tilemap, Tileset};
use raster_math::{IRect, IVec2, Rect, Vec2};

fn tileset() -> Tileset {
    let mut set = Tileset::new(16);
    set.add(TileKind {
        name: "pierre".to_owned(),
        collision: Collision::Solid,
        autotile: true,
        ..TileKind::default()
    });
    set.add(TileKind {
        name: "plateforme".to_owned(),
        collision: Collision::OneWay,
        ..TileKind::default()
    });
    set.add(TileKind {
        name: "herbe".to_owned(),
        collision: Collision::None,
        ..TileKind::default()
    });
    set
}

fn carte() -> Tilemap {
    Tilemap::new(tileset())
}

const PIERRE: TileId = TileId(1);
const PLATEFORME: TileId = TileId(2);
const HERBE: TileId = TileId(3);

#[test]
fn une_carte_neuve_est_vide_partout() {
    let map = carte();

    assert_eq!(map.get(IVec2::ZERO), TileId::EMPTY);
    assert_eq!(map.get(IVec2::new(9999, -9999)), TileId::EMPTY);
    assert_eq!(map.chunk_count(), 0, "un monde vide ne doit rien allouer");
}

#[test]
fn une_tuile_posee_se_relit() {
    let mut map = carte();
    assert!(map.set(IVec2::new(5, 3), PIERRE));
    assert_eq!(map.get(IVec2::new(5, 3)), PIERRE);
}

#[test]
fn reposer_la_meme_tuile_ne_change_rien() {
    let mut map = carte();
    assert!(map.set(IVec2::ZERO, PIERRE));
    assert!(
        !map.set(IVec2::ZERO, PIERRE),
        "la carte s'est dite modifiee"
    );
}

#[test]
fn une_tuile_s_efface() {
    let mut map = carte();
    map.set(IVec2::ZERO, PIERRE);

    assert!(map.clear(IVec2::ZERO));
    assert_eq!(map.get(IVec2::ZERO), TileId::EMPTY);
}

/// Casser une tuile inexistante ne doit pas faire grandir la carte : sinon un
/// joueur qui creuse dans le vide alloue un chunk a chaque coup.
#[test]
fn effacer_dans_le_vide_n_alloue_rien() {
    let mut map = carte();
    assert!(!map.clear(IVec2::new(1000, 1000)));
    assert_eq!(map.chunk_count(), 0);
}

/// Le monde s'etend dans les quatre directions : une division tronquee ferait
/// se chevaucher les tuiles -1 et 0.
#[test]
fn les_coordonnees_negatives_fonctionnent() {
    let mut map = carte();

    map.set(IVec2::new(-1, -1), PIERRE);
    map.set(IVec2::new(0, 0), PLATEFORME);

    assert_eq!(map.get(IVec2::new(-1, -1)), PIERRE);
    assert_eq!(map.get(IVec2::new(0, 0)), PLATEFORME);
    assert_eq!(
        map.chunk_count(),
        2,
        "les deux tuiles devraient etre dans des chunks distincts"
    );
}

#[test]
fn le_chunk_d_une_tuile_negative_est_negatif() {
    assert_eq!(Chunk::coord_of(IVec2::new(-1, -1)), IVec2::new(-1, -1));
    assert_eq!(Chunk::coord_of(IVec2::new(0, 0)), IVec2::ZERO);
    assert_eq!(Chunk::coord_of(IVec2::new(31, 31)), IVec2::ZERO);
    assert_eq!(Chunk::coord_of(IVec2::new(32, 32)), IVec2::new(1, 1));
}

#[test]
fn la_position_dans_le_chunk_reste_positive() {
    for i in -100..100 {
        let local = Chunk::local_of(IVec2::splat(i));
        assert!(
            local.x >= 0 && local.x < Chunk::SIZE,
            "tuile {i} donne une position locale {local}"
        );
    }
}

// --- collision ----------------------------------------------------------------

#[test]
fn la_collision_vient_du_tileset() {
    let mut map = carte();
    map.set(IVec2::new(0, 0), PIERRE);
    map.set(IVec2::new(1, 0), PLATEFORME);
    map.set(IVec2::new(2, 0), HERBE);

    assert_eq!(map.collision(IVec2::new(0, 0)), Collision::Solid);
    assert_eq!(map.collision(IVec2::new(1, 0)), Collision::OneWay);
    assert_eq!(map.collision(IVec2::new(2, 0)), Collision::None);
    assert_eq!(map.collision(IVec2::new(9, 9)), Collision::None);
}

#[test]
fn seule_une_tuile_pleine_bloque() {
    let mut map = carte();
    map.set(IVec2::new(0, 0), PIERRE);
    map.set(IVec2::new(1, 0), PLATEFORME);

    assert!(map.is_solid(IVec2::new(0, 0)));
    assert!(
        !map.is_solid(IVec2::new(1, 0)),
        "une plateforme ne bloque que par le haut"
    );
}

// --- conversion monde / tuiles -------------------------------------------------

#[test]
fn un_point_du_monde_tombe_dans_la_bonne_tuile() {
    let map = carte();

    assert_eq!(map.tile_at(Vec2::new(0.0, 0.0)), IVec2::ZERO);
    assert_eq!(map.tile_at(Vec2::new(15.9, 15.9)), IVec2::ZERO);
    assert_eq!(map.tile_at(Vec2::new(16.0, 16.0)), IVec2::new(1, 1));
}

/// Une troncature ferait tomber -0,5 dans la tuile 0, qui contient deja 0,5 :
/// deux points a un pixel d'ecart se retrouveraient dans la meme tuile.
#[test]
fn un_point_negatif_tombe_dans_la_tuile_negative() {
    let map = carte();

    assert_eq!(map.tile_at(Vec2::new(-0.5, -0.5)), IVec2::new(-1, -1));
    assert_eq!(map.tile_at(Vec2::new(-16.0, -16.0)), IVec2::new(-1, -1));
    assert_eq!(map.tile_at(Vec2::new(-16.1, -16.1)), IVec2::new(-2, -2));
}

#[test]
fn une_tuile_couvre_son_rectangle() {
    let map = carte();
    let bounds = map.tile_bounds(IVec2::new(2, 3));

    assert_eq!(bounds.position, Vec2::new(32.0, 48.0));
    assert_eq!(bounds.size, Vec2::splat(16.0));
}

/// Ce que la collision utilisera : un corps ne teste que les quelques tuiles
/// qu'il chevauche, pas le monde entier.
#[test]
fn les_tuiles_touchees_par_un_rectangle() {
    let map = carte();

    // Un rectangle strictement dans une tuile.
    let une = map.tiles_in(Rect::new(1.0, 1.0, 4.0, 4.0));
    assert_eq!(une.size, IVec2::ONE);

    // Un rectangle a cheval sur quatre tuiles.
    let quatre = map.tiles_in(Rect::new(12.0, 12.0, 8.0, 8.0));
    assert_eq!(quatre.size, IVec2::splat(2));
}

/// Un rectangle qui s'arrete pile sur une frontiere ne doit pas inclure la
/// tuile suivante — sinon un corps colle a un mur declencherait une collision
/// avec la tuile d'apres.
#[test]
fn un_rectangle_aligne_n_inclut_pas_la_tuile_suivante() {
    let map = carte();
    let aligne = map.tiles_in(Rect::new(0.0, 0.0, 16.0, 16.0));

    assert_eq!(aligne.size, IVec2::ONE, "obtenu {aligne}");
    assert_eq!(aligne.min(), IVec2::ZERO);
}

// --- chunks --------------------------------------------------------------------

#[test]
fn un_chunk_se_cree_a_la_premiere_ecriture() {
    let mut map = carte();
    assert_eq!(map.chunk_count(), 0);

    map.set(IVec2::new(5, 5), PIERRE);
    assert_eq!(map.chunk_count(), 1);

    // Une seconde tuile dans le meme chunk n'en cree pas d'autre.
    map.set(IVec2::new(6, 6), PIERRE);
    assert_eq!(map.chunk_count(), 1);
}

#[test]
fn les_chunks_vides_se_nettoient() {
    let mut map = carte();
    map.set(IVec2::new(5, 5), PIERRE);
    map.set(IVec2::new(100, 100), PIERRE);
    assert_eq!(map.chunk_count(), 2);

    map.clear(IVec2::new(5, 5));
    assert_eq!(map.prune(), 1, "le chunk vide aurait du disparaitre");
    assert_eq!(map.chunk_count(), 1);
}

#[test]
fn seuls_les_chunks_visibles_sont_parcourus() {
    let mut map = carte();
    map.set(IVec2::new(0, 0), PIERRE);
    // Tres loin : hors de toute vue raisonnable.
    map.set(IVec2::new(500, 500), PIERRE);

    let vue = Rect::new(0.0, 0.0, 320.0, 180.0);
    let visibles: Vec<_> = map.chunks_in(vue).collect();

    assert_eq!(visibles.len(), 1, "le chunk lointain a ete parcouru");
    assert_eq!(visibles[0].0, IVec2::ZERO);
}

#[test]
fn remplir_une_zone_pose_chaque_tuile() {
    let mut map = carte();
    map.fill(IRect::new(0, 0, 4, 3), PIERRE);

    assert_eq!(map.tile_count(), 12);
    assert_eq!(map.get(IVec2::new(3, 2)), PIERRE);
    assert_eq!(
        map.get(IVec2::new(4, 3)),
        TileId::EMPTY,
        "deborde du rectangle"
    );
}

// --- reconstruction ------------------------------------------------------------

#[test]
fn un_chunk_modifie_demande_une_reconstruction() {
    let mut map = carte();
    map.set(IVec2::new(5, 5), PIERRE);

    assert_eq!(map.dirty_chunks().count(), 1);
    map.clear_dirty();
    assert_eq!(map.dirty_chunks().count(), 0);

    map.set(IVec2::new(6, 6), PIERRE);
    assert_eq!(map.dirty_chunks().count(), 1);
}

/// L'autotuilage regarde les voisins, donc une tuile posee au bord d'un chunk
/// change le dessin de celle d'en face. Sans cela, des coutures apparaissent
/// exactement sur les frontieres de chunks.
#[test]
fn une_tuile_au_bord_salit_le_chunk_voisin() {
    let mut map = carte();

    // Cree d'abord le chunk voisin.
    map.set(IVec2::new(Chunk::SIZE, 5), PIERRE);
    map.clear_dirty();

    // Pose une tuile sur le bord droit du chunk 0.
    map.set(IVec2::new(Chunk::SIZE - 1, 5), PIERRE);

    let sales: Vec<_> = map.dirty_chunks().map(|(c, _)| c).collect();
    assert!(sales.contains(&IVec2::ZERO), "le chunk modifie");
    assert!(
        sales.contains(&IVec2::new(1, 0)),
        "le chunk voisin n'a pas ete marque"
    );
}

#[test]
fn une_tuile_au_centre_ne_salit_pas_les_voisins() {
    let mut map = carte();
    map.set(IVec2::new(Chunk::SIZE, 5), PIERRE);
    map.set(IVec2::new(5, 5), PIERRE);
    map.clear_dirty();

    map.set(IVec2::new(6, 6), PIERRE);

    let sales: Vec<_> = map.dirty_chunks().map(|(c, _)| c).collect();
    assert_eq!(
        sales,
        vec![IVec2::ZERO],
        "un voisin a ete marque sans raison"
    );
}

// --- tileset -------------------------------------------------------------------

#[test]
fn le_vide_occupe_toujours_l_emplacement_zero() {
    let set = Tileset::new(16);
    assert_eq!(set.collision(TileId::EMPTY), Collision::None);
    assert!(set.is_empty(), "un tileset neuf ne contient que le vide");
}

#[test]
fn une_tuile_inconnue_se_comporte_comme_un_trou() {
    let set = tileset();
    assert_eq!(set.collision(TileId(9999)), Collision::None);
}

#[test]
fn un_type_se_retrouve_par_son_nom() {
    let set = tileset();
    assert_eq!(set.find("pierre"), Some(PIERRE));
    assert_eq!(set.find("plateforme"), Some(PLATEFORME));
    assert_eq!(set.find("inexistant"), None);
}

// --- echelle --------------------------------------------------------------------

/// Un grand monde compte des millions de tuiles : le stockage doit tenir, et
/// une tuile ne doit pas couter plus que quelques octets.
#[test]
fn un_grand_monde_tient_en_memoire() {
    let mut map = carte();

    // 200 x 200 tuiles, soit environ 40 chunks.
    map.fill(IRect::new(0, 0, 200, 200), PIERRE);

    assert_eq!(map.tile_count(), 40_000);
    // 200/32 arrondi au superieur, au carre.
    assert_eq!(map.chunk_count(), 7 * 7);
}

#[test]
fn une_tuile_tient_en_deux_octets() {
    assert_eq!(size_of::<TileId>(), 2);
}
