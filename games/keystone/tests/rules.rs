use keystone::rules::{
    Hit, INVULNERABLE, MAX_HEALTH, Progress, STOMP_SPEED, Turn, stomps, take_damage, walker_turn,
};
use raster_math::{Rect, Vec2};

#[test]
fn a_hit_costs_a_heart_and_grants_a_moment_of_grace() {
    let mut health = MAX_HEALTH;
    let mut invulnerable = 0.0;

    assert_eq!(take_damage(&mut health, &mut invulnerable, 1), Hit::Hurt);
    assert_eq!(health, MAX_HEALTH - 1);
    assert_eq!(invulnerable, INVULNERABLE);
}

#[test]
fn a_second_hit_during_the_grace_costs_nothing() {
    let mut health = MAX_HEALTH;
    let mut invulnerable = 0.0;

    take_damage(&mut health, &mut invulnerable, 1);
    let apres = health;

    // Sans cela, traverser un ennemi couterait les trois coeurs d'un coup.
    assert_eq!(take_damage(&mut health, &mut invulnerable, 1), Hit::Ignored);
    assert_eq!(health, apres);
}

#[test]
fn the_last_heart_kills() {
    let mut health = 1;
    let mut invulnerable = 0.0;

    assert_eq!(take_damage(&mut health, &mut invulnerable, 1), Hit::Killed);
    assert_eq!(health, 0);
}

#[test]
fn a_dead_player_takes_no_further_damage() {
    let mut health = 0;
    let mut invulnerable = 0.0;

    assert_eq!(take_damage(&mut health, &mut invulnerable, 1), Hit::Ignored);
}

#[test]
fn a_big_hit_never_wraps_below_zero() {
    let mut health = 1;
    let mut invulnerable = 0.0;

    assert_eq!(take_damage(&mut health, &mut invulnerable, 99), Hit::Killed);
    assert_eq!(health, 0, "la sante a boucle sous zero");
}

#[test]
fn falling_onto_an_enemy_squashes_it() {
    let ennemi = Rect::new(100.0, 100.0, 14.0, 14.0);
    // Les pieds a 102 : dans la moitie haute de l'ennemi.
    let joueur = Rect::new(100.0, 88.0, 12.0, 14.0);

    assert!(stomps(joueur, Vec2::new(0.0, 200.0), ennemi));
}

#[test]
fn walking_into_an_enemy_does_not_squash_it() {
    let ennemi = Rect::new(100.0, 100.0, 14.0, 14.0);
    let joueur = Rect::new(92.0, 100.0, 12.0, 14.0);

    assert!(
        !stomps(joueur, Vec2::new(90.0, 0.0), ennemi),
        "un contact lateral ne doit pas ecraser"
    );
}

#[test]
fn jumping_up_into_an_enemy_does_not_squash_it() {
    let ennemi = Rect::new(100.0, 100.0, 14.0, 14.0);
    let joueur = Rect::new(100.0, 105.0, 12.0, 14.0);

    // Vitesse negative : le joueur monte.
    assert!(!stomps(joueur, Vec2::new(0.0, -200.0), ennemi));
}

#[test]
fn a_slow_fall_does_not_squash() {
    let ennemi = Rect::new(100.0, 100.0, 14.0, 14.0);
    let joueur = Rect::new(100.0, 88.0, 12.0, 14.0);

    assert!(!stomps(joueur, Vec2::new(0.0, STOMP_SPEED - 1.0), ennemi));
    assert!(stomps(joueur, Vec2::new(0.0, STOMP_SPEED), ennemi));
}

#[test]
fn falling_past_an_enemy_does_not_squash_it() {
    let ennemi = Rect::new(100.0, 100.0, 14.0, 14.0);
    // Les pieds a 114 : sous la moitie de l'ennemi, le joueur passe a cote.
    let joueur = Rect::new(100.0, 100.0, 12.0, 14.0);

    assert!(
        !stomps(joueur, Vec2::new(0.0, 200.0), ennemi),
        "tomber le long d'un ennemi ne doit pas l'ecraser"
    );
}

#[test]
fn a_walker_turns_at_a_wall_and_at_a_ledge() {
    assert_eq!(walker_turn(true, true), Turn::Reverse, "un mur devant");
    assert_eq!(walker_turn(false, false), Turn::Reverse, "le vide devant");
    assert_eq!(walker_turn(false, true), Turn::Keep);
    assert_eq!(walker_turn(true, false), Turn::Reverse);
}

#[test]
fn progress_starts_whole() {
    let progress = Progress::new();

    assert_eq!(progress.health, MAX_HEALTH);
    assert_eq!(progress.keys, 0);
    assert_eq!(progress.room, 0);
    assert!(progress.alive());
    assert!(progress.cleared.is_empty());
}

#[test]
fn a_room_is_only_cleared_once() {
    let mut progress = Progress::new();

    progress.clear(0);
    progress.clear(0);
    progress.clear(1);

    assert_eq!(progress.cleared, [0, 1]);
    assert!(progress.has_cleared(0));
    assert!(!progress.has_cleared(2));
}

#[test]
fn a_player_without_hearts_is_not_alive() {
    let mut progress = Progress::new();
    progress.health = 0;

    assert!(!progress.alive());
}
