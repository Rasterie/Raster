use raster_2d::{Collision, TileId, TileKind, Tilemap, Tileset};
use raster_math::{IRect, Rect, Vec2};
use raster_physics::{Body, Contacts, grounded, move_body, overlapping};

const SOL: TileId = TileId(1);
const PLATEFORME: TileId = TileId(2);
const FRAME: f32 = 1.0 / 60.0;

fn carte() -> Tilemap {
    let mut set = Tileset::new(16);
    set.add(TileKind {
        name: "sol".to_owned(),
        collision: Collision::Solid,
        ..TileKind::default()
    });
    set.add(TileKind {
        name: "plateforme".to_owned(),
        collision: Collision::OneWay,
        ..TileKind::default()
    });
    Tilemap::new(set)
}

/// Un corps de 8x8, place a une position du monde.
fn corps(x: f32, y: f32) -> Body {
    Body::new(Rect::new(x, y, 8.0, 8.0))
}

/// Simule jusqu'au premier contact, en maintenant la vitesse — un test d'une
/// seule frame n'atteindrait pas une tuile situee a plus de `v * dt` pixels.
fn avancer(body: &mut Body, map: &Tilemap, velocity: Vec2, frames: u32) -> Contacts {
    let mut total = Contacts::default();
    for _ in 0..frames {
        body.velocity = velocity;
        let c = move_body(body, map, FRAME);
        total.left |= c.left;
        total.right |= c.right;
        total.above |= c.above;
        total.below |= c.below;
    }
    total
}

#[test]
fn un_corps_se_deplace_librement_dans_le_vide() {
    let map = carte();
    let mut body = corps(0.0, 0.0);
    body.velocity = Vec2::new(60.0, 0.0);

    let contacts = move_body(&mut body, &map, FRAME);

    assert!(body.position().x > 0.0);
    assert!(!contacts.any(), "un contact dans le vide");
}

#[test]
fn un_corps_s_arrete_contre_un_mur() {
    let mut map = carte();
    // Un mur a la tuile x = 2, soit x = 32 en pixels.
    map.fill(IRect::new(2, 0, 1, 4), SOL);

    let mut body = corps(0.0, 0.0);
    let contacts = avancer(&mut body, &map, Vec2::new(600.0, 0.0), 10);

    assert!(contacts.right, "le mur n'a pas ete detecte");
    assert!(
        body.bounds.right() <= 32.0,
        "entre dans le mur : {}",
        body.bounds.right()
    );
    assert_eq!(body.velocity.x, 0.0, "la vitesse aurait du etre annulee");
}

#[test]
fn un_corps_s_arrete_contre_un_mur_a_gauche() {
    let mut map = carte();
    map.fill(IRect::new(0, 0, 1, 4), SOL);

    let mut body = corps(32.0, 0.0);
    let contacts = avancer(&mut body, &map, Vec2::new(-600.0, 0.0), 10);

    assert!(contacts.left);
    assert!(
        body.bounds.left() >= 16.0,
        "entre dans le mur : {}",
        body.bounds.left()
    );
}

#[test]
fn un_corps_atterrit_sur_le_sol() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 2), SOL);

    let mut body = corps(0.0, 0.0);
    let contacts = avancer(&mut body, &map, Vec2::new(0.0, 600.0), 20);

    assert!(contacts.below, "le sol n'a pas ete detecte");
    assert!(contacts.grounded());
    assert!(
        body.bounds.bottom() <= 64.0,
        "traverse le sol : {}",
        body.bounds.bottom()
    );
}

#[test]
fn un_corps_cogne_le_plafond() {
    let mut map = carte();
    map.fill(IRect::new(0, 0, 10, 1), SOL);

    let mut body = corps(0.0, 32.0);
    let contacts = avancer(&mut body, &map, Vec2::new(0.0, -600.0), 10);

    assert!(contacts.above);
    assert!(
        body.bounds.top() >= 16.0,
        "traverse le plafond : {}",
        body.bounds.top()
    );
}

/// A 3000 px/s un corps parcourt 50 px en une frame, soit trois tuiles : sans
/// decoupage du pas il franchirait le mur sans le voir.
#[test]
fn un_corps_rapide_ne_traverse_pas_un_mur() {
    let mut map = carte();
    // Un mur d'une seule tuile, a mi-parcours du deplacement.
    map.fill(IRect::new(2, 0, 1, 4), SOL);

    let mut body = corps(0.0, 0.0);
    body.velocity = Vec2::new(3000.0, 0.0);
    let contacts = move_body(&mut body, &map, FRAME);

    assert!(contacts.right, "le mur a ete traverse");
    assert!(
        body.bounds.right() <= 32.0,
        "de l'autre cote : {}",
        body.bounds.right()
    );
}

/// 5000 px/s : 83 px en une frame, soit cinq tuiles franchies d'un coup.
#[test]
fn une_chute_rapide_ne_traverse_pas_le_sol() {
    let mut map = carte();
    // Un sol d'une seule tuile d'epaisseur, a mi-chute.
    map.fill(IRect::new(0, 3, 10, 1), SOL);

    let mut body = corps(0.0, 0.0);
    body.velocity = Vec2::new(0.0, 5000.0);
    let contacts = move_body(&mut body, &map, FRAME);

    assert!(contacts.below, "le sol a ete traverse");
    assert!(
        body.bounds.bottom() <= 48.5,
        "sous le sol : {}",
        body.bounds.bottom()
    );
}

/// Un corps qui longe un sol plat ne doit pas accrocher aux jointures entre
/// deux tuiles : c'est ce que le deplacement axe par axe evite.
#[test]
fn un_corps_glisse_le_long_d_un_sol_plat() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 20, 2), SOL);

    let mut body = corps(0.0, 56.0);
    let depart = body.position().x;

    for _ in 0..60 {
        body.velocity = Vec2::new(120.0, 200.0);
        move_body(&mut body, &map, FRAME);
    }

    let parcouru = body.position().x - depart;
    assert!(
        parcouru > 100.0,
        "a accroche en chemin : {parcouru} px parcourus"
    );
}

#[test]
fn un_corps_pose_se_sait_au_sol() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 2), SOL);

    // Pose exactement sur le sol, sans vitesse.
    let mut body = corps(0.0, 64.0 - 8.0);
    let contacts = move_body(&mut body, &map, FRAME);

    assert!(contacts.grounded(), "un corps pose se croit en l'air");
    assert!(grounded(&body, &map));
}

#[test]
fn un_corps_en_l_air_ne_se_croit_pas_au_sol() {
    let mut map = carte();
    map.fill(IRect::new(0, 10, 10, 2), SOL);

    let body = corps(0.0, 0.0);
    assert!(!grounded(&body, &map));
}

// --- plateformes traversables --------------------------------------------------

#[test]
fn une_plateforme_arrete_une_chute() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 1), PLATEFORME);

    let mut body = corps(0.0, 40.0);
    let contacts = avancer(&mut body, &map, Vec2::new(0.0, 300.0), 20);

    assert!(contacts.below, "la plateforme n'a pas porte");
    assert!(
        body.bounds.bottom() <= 64.5,
        "a traverse : {}",
        body.bounds.bottom()
    );
}

/// Sauter a travers une plateforme par en dessous est ce qui la distingue d'un
/// bloc plein.
#[test]
fn une_plateforme_se_traverse_par_en_dessous() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 1), PLATEFORME);

    let mut body = corps(0.0, 80.0);
    let contacts = avancer(&mut body, &map, Vec2::new(0.0, -300.0), 5);

    assert!(!contacts.above, "la plateforme a bloque par en dessous");
    assert!(body.position().y < 80.0, "n'a pas monte");
}

#[test]
fn une_plateforme_se_traverse_en_descendant_a_la_demande() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 1), PLATEFORME);

    let mut body = corps(0.0, 40.0);
    body.drop_through = true;
    let contacts = avancer(&mut body, &map, Vec2::new(0.0, 300.0), 20);

    assert!(!contacts.below, "la plateforme a porte malgre drop_through");
    assert!(body.bounds.top() > 64.0, "n'est pas passe au travers");
}

// --- cas limites -----------------------------------------------------------------

#[test]
fn un_corps_immobile_ne_bouge_pas() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 2), SOL);

    let mut body = corps(10.0, 20.0);
    let avant = body.position();
    move_body(&mut body, &map, FRAME);

    assert_eq!(body.position(), avant);
}

#[test]
fn une_carte_vide_n_arrete_rien() {
    let map = carte();
    let mut body = corps(0.0, 0.0);
    let contacts = avancer(&mut body, &map, Vec2::new(500.0, 500.0), 10);
    assert!(!contacts.any());
}

#[test]
fn les_coordonnees_negatives_collisionnent_aussi() {
    let mut map = carte();
    map.fill(IRect::new(-5, -5, 2, 10), SOL);

    // Le mur va de la tuile -5 a -4, soit x de -80 a -48.
    let mut body = corps(-100.0, -40.0);
    let contacts = avancer(&mut body, &map, Vec2::new(600.0, 0.0), 10);
    assert!(
        contacts.right,
        "un mur en coordonnees negatives a ete ignore"
    );
}

#[test]
fn le_chevauchement_se_detecte() {
    let mut map = carte();
    map.fill(IRect::new(0, 0, 4, 4), SOL);

    assert!(overlapping(&corps(8.0, 8.0), &map));
    assert!(!overlapping(&corps(100.0, 100.0), &map));
}

/// Un corps qui tombe dans un couloir d'une tuile de large ne doit pas rester
/// coince : c'est le cas ou une resolution simultanee des deux axes echoue.
#[test]
fn un_corps_tombe_dans_un_couloir_etroit() {
    let mut map = carte();
    map.fill(IRect::new(0, 0, 1, 20), SOL);
    map.fill(IRect::new(2, 0, 1, 20), SOL);

    // Le couloir fait 16 px de large, le corps 8.
    let mut body = corps(20.0, 0.0);
    let depart = body.position().y;

    for _ in 0..30 {
        body.velocity = Vec2::new(0.0, 300.0);
        move_body(&mut body, &map, FRAME);
    }

    assert!(
        body.position().y > depart + 100.0,
        "coince : {}",
        body.position().y
    );
}

#[test]
fn un_corps_ne_s_enfonce_pas_avec_le_temps() {
    let mut map = carte();
    map.fill(IRect::new(0, 4, 10, 2), SOL);

    let mut body = corps(0.0, 40.0);

    // Cent frames de gravite : la position finale doit rester stable.
    for _ in 0..100 {
        body.velocity = Vec2::new(0.0, 500.0);
        move_body(&mut body, &map, FRAME);
    }

    let repos = body.bounds.bottom();
    for _ in 0..100 {
        body.velocity = Vec2::new(0.0, 500.0);
        move_body(&mut body, &map, FRAME);
    }

    assert!(
        (body.bounds.bottom() - repos).abs() < 0.001,
        "s'est enfonce de {} px",
        body.bounds.bottom() - repos
    );
}
