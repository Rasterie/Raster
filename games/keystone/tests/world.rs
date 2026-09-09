use keystone::world::{Enemy, Kind, Player, Room, TILE};
use raster_math::Vec2;

fn room() -> Room {
    Room {
        name: "test".to_owned(),
        tiles: vec![
            "........".to_owned(),
            "....==..".to_owned(),
            "........".to_owned(),
            "########".to_owned(),
        ],
        spawn: Vec2::new(16.0, 16.0),
    }
}

#[test]
fn a_room_knows_its_shape() {
    let room = room();
    assert_eq!(room.width(), 8);
    assert_eq!(room.height(), 4);
}

#[test]
fn a_solid_tile_blocks_and_a_platform_does_not() {
    let room = room();

    assert!(room.solid(0, 3), "le sol doit etre solide");
    assert!(!room.solid(4, 1), "une plateforme n'est pas solide");
    assert!(room.platform(4, 1));
    assert!(!room.platform(0, 3));
}

#[test]
fn outside_the_room_is_empty_not_a_crash() {
    let room = room();

    // Un ennemi qui marche jusqu'au bord interroge des cases hors salle.
    assert_eq!(room.at(-1, 0), '.');
    assert_eq!(room.at(0, -1), '.');
    assert_eq!(room.at(99, 99), '.');
    assert!(!room.solid(-5, -5));
}

#[test]
fn ground_is_found_under_a_position() {
    let room = room();

    // Le sol occupe la ligne 3, soit y de 48 a 64.
    assert!(room.ground_under(Vec2::new(8.0, 3.0 * TILE)));
    assert!(!room.ground_under(Vec2::new(8.0, 0.0)));

    // Sous une plateforme, colonne 4, ligne 1.
    assert!(room.ground_under(Vec2::new(4.0 * TILE, TILE)));
}

#[test]
fn an_enemy_reads_its_kind_from_a_number() {
    let mut enemy = Enemy::default();

    assert_eq!(enemy.kind(), Kind::Walker);
    enemy.kind = 1;
    assert_eq!(enemy.kind(), Kind::Spike);
    enemy.kind = 2;
    assert_eq!(enemy.kind(), Kind::Flyer);

    // Une scene qui nomme un type inconnu ne doit pas planter le jeu.
    enemy.kind = 99;
    assert_eq!(enemy.kind(), Kind::Walker);
}

#[test]
fn bounds_follow_position() {
    let mut player = Player::default();
    player.position = Vec2::new(100.0, 50.0);

    let bounds = player.bounds();
    assert_eq!(bounds.position, Vec2::new(100.0, 50.0));
    assert!(bounds.size.x > 0.0 && bounds.size.y > 0.0);
}
