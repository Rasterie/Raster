use keystone::rooms;
use keystone::world::TILE;

#[test]
fn every_room_is_well_formed() {
    for room in rooms::all() {
        if let Err(e) = rooms::check(&room) {
            panic!("{e}");
        }
    }
}

#[test]
fn the_game_has_rooms_and_they_are_named() {
    let rooms = rooms::all();

    assert!(rooms.len() >= 3, "trop peu de salles : {}", rooms.len());
    for room in &rooms {
        assert!(!room.name.is_empty(), "une salle sans nom");
    }
}

#[test]
fn every_spawn_is_inside_its_room_and_not_in_a_wall() {
    for room in rooms::all() {
        let x = (room.spawn.x / TILE).floor() as i32;
        let y = (room.spawn.y / TILE).floor() as i32;

        assert!(
            x >= 0 && (x as usize) < room.width(),
            "`{}` : depart hors salle en x",
            room.name
        );
        assert!(
            y >= 0 && (y as usize) < room.height(),
            "`{}` : depart hors salle en y",
            room.name
        );
        assert!(
            !room.solid(x, y),
            "`{}` : le joueur apparait dans un mur",
            room.name
        );
    }
}

#[test]
fn every_room_has_ground_somewhere_under_the_spawn() {
    for room in rooms::all() {
        let x = (room.spawn.x / TILE).floor() as i32;
        let sous = (0..room.height() as i32).any(|y| room.solid(x, y) || room.platform(x, y));

        assert!(
            sous,
            "`{}` : le joueur tombe dans le vide au depart",
            room.name
        );
    }
}

#[test]
fn a_malformed_room_is_reported() {
    use keystone::world::Room;
    use raster_math::Vec2;

    let courte = Room {
        name: "cassee".to_owned(),
        tiles: vec!["###".to_owned()],
        spawn: Vec2::ZERO,
    };
    assert!(
        rooms::check(&courte).is_err(),
        "une ligne trop courte est passee"
    );

    let inconnu = Room {
        name: "cassee".to_owned(),
        tiles: vec!["#########X##########".to_owned()],
        spawn: Vec2::ZERO,
    };
    match rooms::check(&inconnu) {
        Err(e) => assert!(e.contains('X'), "{e}"),
        Ok(()) => panic!("un caractere inconnu est passe"),
    }

    let vide = Room {
        name: "vide".to_owned(),
        tiles: Vec::new(),
        spawn: Vec2::ZERO,
    };
    assert!(rooms::check(&vide).is_err());
}
