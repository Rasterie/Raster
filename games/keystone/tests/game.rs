use keystone::game::{GRAVITY, Game, JUMP, SPEED, Screen};
use keystone::rules::MAX_HEALTH;
use keystone::world::{Kind, TILE};
use raster_math::Vec2;

const DT: f32 = 1.0 / 60.0;

/// Avance le jeu de `frames` pas, avec les memes commandes tout du long.
fn run(game: &mut Game, left: bool, right: bool, jump: bool, frames: usize) {
    for _ in 0..frames {
        game.step(left, right, jump, DT);
    }
}

fn playing() -> Game {
    let mut game = Game::new();
    game.start();
    game
}

#[test]
fn a_new_game_starts_on_the_title_screen() {
    let game = Game::new();
    assert_eq!(game.screen, Screen::Title);
}

#[test]
fn starting_puts_the_player_in_the_first_room_at_full_health() {
    let game = playing();

    assert_eq!(game.screen, Screen::Playing);
    assert_eq!(game.progress.room, 0);
    assert_eq!(game.progress.health, MAX_HEALTH);
    assert_eq!(game.player, game.room().spawn);
}

#[test]
fn gravity_brings_the_player_down_to_the_ground() {
    let mut game = playing();
    let depart = game.player.y;

    run(&mut game, false, false, false, 120);

    assert!(game.player.y > depart, "le joueur n'est pas tombe");
    assert!(game.on_ground, "le joueur n'a jamais touche le sol");
    assert!(
        game.player.y < game.room().height() as f32 * TILE,
        "le joueur est passe a travers le sol : {}",
        game.player.y
    );
}

#[test]
fn the_player_stops_falling_once_grounded() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);

    let pose = game.player.y;
    run(&mut game, false, false, false, 60);

    assert!(
        (game.player.y - pose).abs() < 0.5,
        "le joueur s'enfonce : {pose} puis {}",
        game.player.y
    );
}

#[test]
fn walking_moves_the_player_and_turns_it() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);

    let depart = game.player.x;
    run(&mut game, false, true, false, 30);

    assert!(game.player.x > depart, "le joueur n'avance pas a droite");
    assert_eq!(game.facing, 1.0);

    let droite = game.player.x;
    run(&mut game, true, false, false, 30);

    assert!(game.player.x < droite, "le joueur n'avance pas a gauche");
    assert_eq!(game.facing, -1.0);
}

#[test]
fn jumping_only_works_from_the_ground() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);
    assert!(game.on_ground);

    let events = game.step(false, false, true, DT);
    assert!(events.jumped);
    assert!(game.velocity.y < 0.0, "le saut n'a pas envoye vers le haut");

    // En l'air, le saut ne repart pas : pas de double saut.
    let en_lair = game.step(false, false, true, DT);
    assert!(!en_lair.jumped, "un second saut est parti en plein vol");
}

#[test]
fn a_jump_comes_back_down() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);
    let sol = game.player.y;

    game.step(false, false, true, DT);
    run(&mut game, false, false, false, 10);
    let sommet = game.player.y;
    assert!(sommet < sol, "le saut n'a pas decolle");

    run(&mut game, false, false, false, 120);
    assert!(
        (game.player.y - sol).abs() < 1.0,
        "le joueur n'est pas revenu au sol : {sol} puis {}",
        game.player.y
    );
}

#[test]
fn a_jump_clears_at_least_two_tiles() {
    // Sans cela, les plateformes du niveau seraient hors d'atteinte.
    let hauteur = JUMP * JUMP / (2.0 * GRAVITY);
    assert!(
        hauteur > 2.0 * TILE,
        "un saut monte de {hauteur} px, moins de deux tuiles"
    );
}

#[test]
fn walls_stop_the_player() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);

    // Fonce a droite bien plus longtemps qu'il n'en faut pour traverser.
    run(&mut game, false, true, false, 600);

    let bord = game.room().width() as f32 * TILE;
    assert!(
        game.player.x + 12.0 <= bord + 0.5,
        "le joueur est sorti de la salle : {} sur {bord}",
        game.player.x
    );
}

#[test]
fn falling_out_of_the_room_kills() {
    let mut game = playing();
    game.player = Vec2::new(50.0, game.room().height() as f32 * TILE + 100.0);

    game.step(false, false, false, DT);

    assert_eq!(game.screen, Screen::Dead);
    assert_eq!(game.progress.health, 0);
}

#[test]
fn touching_an_enemy_costs_a_heart() {
    let mut game = playing();
    game.progress.room = 1;
    game.enter_room(1);

    let foe = game.foes[0].position;
    game.player = foe;
    let events = game.step(false, false, false, DT);

    assert!(events.hurt);
    assert_eq!(game.progress.health, MAX_HEALTH - 1);
    assert!(game.invulnerable > 0.0);
}

#[test]
fn landing_on_a_walker_squashes_it() {
    let mut game = playing();
    game.enter_room(1);

    let foe = game.foes[0].position;
    // Juste au-dessus, en train de tomber vite.
    game.player = Vec2::new(foe.x, foe.y - 13.0);
    game.velocity = Vec2::new(0.0, 200.0);

    let events = game.step(false, false, false, DT);

    assert!(events.stomped, "l'ennemi n'a pas ete ecrase");
    assert!(!game.foes[0].alive);
    assert_eq!(game.progress.health, MAX_HEALTH, "l'ecrasement a blesse");
    assert!(game.velocity.y < 0.0, "l'ecrasement n'a pas fait rebondir");
}

#[test]
fn a_spike_cannot_be_squashed() {
    let mut game = playing();
    game.enter_room(1);

    let index = game
        .foes
        .iter()
        .position(|f| f.kind == Kind::Spike)
        .expect("la salle 1 a un pic");
    let spike = game.foes[index].position;

    game.player = Vec2::new(spike.x, spike.y - 13.0);
    game.velocity = Vec2::new(0.0, 200.0);
    let events = game.step(false, false, false, DT);

    assert!(!events.stomped, "un pic a ete ecrase");
    assert!(game.foes[index].alive);
    assert!(events.hurt, "sauter sur un pic doit blesser");
}

#[test]
fn a_walker_turns_around_instead_of_walking_off() {
    let mut game = playing();
    game.enter_room(1);

    let depart = game.foes[0].position.x;
    let mut min = depart;
    let mut max = depart;

    // Assez long pour plusieurs allers-retours.
    for _ in 0..1200 {
        game.step(false, false, false, DT);
        // Le joueur reste hors de portee.
        game.player = Vec2::new(-100.0, 0.0);
        min = min.min(game.foes[0].position.x);
        max = max.max(game.foes[0].position.x);
    }

    assert!(max > min, "l'ennemi n'a pas bouge");
    let room = game.room().clone();
    assert!(
        game.foes[0].position.x >= 0.0 && game.foes[0].position.x < room.width() as f32 * TILE,
        "l'ennemi est sorti de la salle : {}",
        game.foes[0].position.x
    );
}

#[test]
fn a_flyer_stays_within_its_range() {
    let mut game = playing();
    game.enter_room(2);

    let index = game
        .foes
        .iter()
        .position(|f| f.kind == Kind::Flyer)
        .expect("la salle 2 a un voltigeur");
    let origin = game.foes[index].origin.x;
    let range = game.foes[index].range;

    for _ in 0..1200 {
        game.step(false, false, false, DT);
        game.player = Vec2::new(-100.0, 0.0);

        let ecart = (game.foes[index].position.x - origin).abs();
        assert!(
            ecart <= range + 2.0,
            "le voltigeur s'est echappe : {ecart} pour une portee de {range}"
        );
    }
}

#[test]
fn the_door_needs_a_key() {
    let mut game = playing();
    game.player = game.door;

    game.step(false, false, false, DT);
    assert_eq!(game.progress.room, 0, "la porte s'est ouverte sans clef");

    // Avec la clef, elle mene a la salle suivante.
    game.player = game.key.expect("la salle 0 a une clef");
    let pris = game.step(false, false, false, DT);
    assert!(pris.took_key);
    assert_eq!(game.progress.keys, 1);

    game.player = game.door;
    let passe = game.step(false, false, false, DT);
    assert!(passe.cleared_room);
    assert_eq!(game.progress.room, 1);
}

#[test]
fn the_game_can_be_finished() {
    let mut game = playing();

    // Traverse chaque salle : prendre la clef, atteindre la porte.
    for _ in 0..game.rooms.len() {
        let key = game.key.expect("chaque salle a une clef");
        game.player = key;
        game.step(false, false, false, DT);

        game.player = game.door;
        game.step(false, false, false, DT);
    }

    assert_eq!(game.screen, Screen::Won, "le jeu ne s'est jamais termine");
    assert_eq!(game.progress.cleared.len(), game.rooms.len());
}

#[test]
fn dying_and_respawning_keeps_the_rooms_already_cleared() {
    let mut game = playing();

    // Franchit la premiere salle.
    game.player = game.key.unwrap();
    game.step(false, false, false, DT);
    game.player = game.door;
    game.step(false, false, false, DT);
    assert_eq!(game.progress.room, 1);

    game.progress.health = 1;
    game.invulnerable = 0.0;
    game.player = game.foes[0].position;
    game.step(false, false, false, DT);
    assert_eq!(game.screen, Screen::Dead);

    game.respawn();

    assert_eq!(game.screen, Screen::Playing);
    assert_eq!(game.progress.health, MAX_HEALTH);
    assert_eq!(game.progress.room, 1, "le respawn a renvoye au debut");
    assert!(
        game.progress.has_cleared(0),
        "les salles franchies sont perdues"
    );
}

#[test]
fn nothing_happens_while_paused() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);

    game.screen = Screen::Paused;
    let avant = game.player;
    run(&mut game, false, true, true, 60);

    assert_eq!(game.player, avant, "le jeu a avance en pause");
}

#[test]
fn the_player_never_crosses_a_wall_at_full_speed() {
    let mut game = playing();
    run(&mut game, false, false, false, 120);

    // Un pas entier a pleine vitesse reste plus petit qu'une tuile : sans cela
    // le joueur traverserait les murs entre deux frames.
    let pas = SPEED * DT;
    assert!(
        pas < TILE,
        "un pas de {pas} px franchit une tuile de {TILE} px"
    );
}
