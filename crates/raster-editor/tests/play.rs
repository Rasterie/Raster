use raster_core::reflect::{Reflect, TypeRegistry};
use raster_editor::commands::Editing;
use raster_editor::play::{self, Session, State};
use raster_math::Vec2;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Prop {
    position: Vec2,
    health: u32,
    name: String,
    alive: bool,
}

/// Un second type : une capture qui ne parcourt qu'un pool ne le verrait pas.
#[derive(Reflect, Default, Debug, PartialEq)]
struct Light {
    position: Vec2,
    radius: f32,
}

fn editing() -> Editing {
    let mut registry = TypeRegistry::new();
    registry.register::<Prop>();
    registry.register::<Light>();
    Editing::new(registry)
}

/// Le monde tel qu'il serait apres quelques poses.
fn built() -> Editing {
    let mut editing = editing();
    for i in 0..3 {
        editing.world.spawn(Prop {
            position: Vec2::new(i as f32 * 16.0, 32.0),
            health: 3,
            name: format!("acteur {i}"),
            alive: true,
        });
    }
    editing
}

/// Les acteurs, tries pour comparer deux mondes.
fn contents(editing: &Editing) -> Vec<(Vec2, u32, String, bool)> {
    let mut out: Vec<_> = editing
        .world
        .values::<Prop>()
        .map(|p| (p.position, p.health, p.name.clone(), p.alive))
        .collect();
    out.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
    out
}

#[test]
fn a_fresh_session_is_editing() {
    let session = Session::new();

    assert_eq!(session.state(), State::Editing);
    assert!(!session.state().in_session());
    assert!(!session.has_snapshot());
}

#[test]
fn starting_takes_a_snapshot() {
    let editing = built();
    let mut session = Session::new();

    session.start(&editing);

    assert_eq!(session.state(), State::Playing);
    assert!(session.has_snapshot());
}

#[test]
fn stopping_puts_the_world_back_exactly() {
    let mut editing = built();
    let avant = contents(&editing);
    let mut session = Session::new();

    session.start(&editing);

    // Ce que le jeu ferait : bouger, blesser, tuer.
    for (_, prop) in editing.world.iter_mut::<Prop>() {
        prop.position += Vec2::new(100.0, 100.0);
        prop.health = 0;
        prop.alive = false;
    }
    let ids: Vec<_> = editing.world.actor_ids();
    editing.world.despawn(ids[0]);

    let restored = session.stop(&mut editing).unwrap();

    assert_eq!(restored, 3, "les trois acteurs doivent revenir");
    assert_eq!(
        contents(&editing),
        avant,
        "le monde edite n'a pas ete rendu intact"
    );
}

#[test]
fn stopping_without_playing_does_nothing() {
    let mut editing = built();
    let mut session = Session::new();

    assert_eq!(session.stop(&mut editing), None);
    assert_eq!(editing.world.count::<Prop>(), 3, "rien ne doit disparaitre");
}

#[test]
fn starting_twice_keeps_the_first_snapshot() {
    let mut editing = built();
    let avant = contents(&editing);
    let mut session = Session::new();

    session.start(&editing);
    for (_, prop) in editing.world.iter_mut::<Prop>() {
        prop.health = 0;
    }

    // Relancer perdrait la capture d'origine : l'arret ne rendrait plus le
    // monde edite, mais celui du milieu de partie.
    session.start(&editing);
    session.stop(&mut editing);

    assert_eq!(contents(&editing), avant);
}

#[test]
fn an_actor_created_during_play_does_not_survive() {
    let mut editing = built();
    let mut session = Session::new();

    session.start(&editing);
    editing.world.spawn(Prop {
        name: "projectile".to_owned(),
        ..Prop::default()
    });
    assert_eq!(editing.world.count::<Prop>(), 4);

    session.stop(&mut editing);

    assert_eq!(
        editing.world.count::<Prop>(),
        3,
        "ce que le jeu cree ne doit pas se retrouver dans la scene"
    );
}

#[test]
fn an_empty_world_survives_a_session() {
    let mut editing = editing();
    let mut session = Session::new();

    session.start(&editing);
    editing.world.spawn(Prop::default());
    session.stop(&mut editing);

    assert_eq!(editing.world.count::<Prop>(), 0);
}

#[test]
fn ticking_only_advances_while_playing() {
    let editing = built();
    let mut session = Session::new();

    assert!(!session.tick(0.016), "rien ne tourne en edition");

    session.start(&editing);
    assert!(session.tick(0.016));
    assert_eq!(session.steps, 1);
    assert!((session.elapsed - 0.016).abs() < 1e-6);
}

#[test]
fn pausing_freezes_the_world() {
    let editing = built();
    let mut session = Session::new();
    session.start(&editing);
    session.tick(0.016);

    session.toggle_pause();
    assert_eq!(session.state(), State::Paused);
    assert!(!session.tick(0.016), "une pause doit figer le monde");
    assert_eq!(session.steps, 1, "le compteur ne bouge plus");

    session.toggle_pause();
    assert!(session.tick(0.016), "reprendre doit relancer");
}

#[test]
fn pausing_while_editing_changes_nothing() {
    let mut session = Session::new();
    session.toggle_pause();

    assert_eq!(session.state(), State::Editing);
}

#[test]
fn a_paused_session_still_restores_on_stop() {
    let mut editing = built();
    let avant = contents(&editing);
    let mut session = Session::new();

    session.start(&editing);
    session.toggle_pause();
    for (_, prop) in editing.world.iter_mut::<Prop>() {
        prop.health = 99;
    }

    assert!(session.stop(&mut editing).is_some());
    assert_eq!(contents(&editing), avant);
}

#[test]
fn the_time_resets_between_sessions() {
    let mut editing = built();
    let mut session = Session::new();

    session.start(&editing);
    for _ in 0..10 {
        session.tick(0.016);
    }
    session.stop(&mut editing);

    session.start(&editing);
    assert_eq!(session.steps, 0);
    assert_eq!(session.elapsed, 0.0);
}

#[test]
fn a_world_becomes_a_scene_that_reloads() {
    let editing = built();
    let scene = play::to_scene(&editing.world, "niveau");

    assert_eq!(scene.len(), 3);

    // Et cette scene se relit dans un monde neuf.
    let mut registry = TypeRegistry::new();
    registry.register::<Prop>();
    let relu = raster_core::Scene::parse(&scene.to_toml_string(), &registry).unwrap();
    let mut world = raster_core::World::new();
    let poses = relu.spawn_into(&mut world, &registry).unwrap();

    assert_eq!(poses.len(), 3);
}

#[test]
fn a_scene_from_a_world_keeps_every_field() {
    let mut editing = editing();
    editing.world.spawn(Prop {
        position: Vec2::new(7.0, 9.0),
        health: 2,
        name: "unique".to_owned(),
        alive: true,
    });

    let scene = play::to_scene(&editing.world, "n");
    let toml = scene.to_toml_string();

    // Tous les champs, meme ceux qui valent leur defaut : l'editeur ne connait
    // pas les valeurs par defaut du type.
    assert!(toml.contains("name = \"unique\""), "{toml}");
    assert!(toml.contains("health = 2"), "{toml}");
    assert!(toml.contains("position = [7.0, 9.0]"), "{toml}");
}

#[test]
fn the_state_says_what_it_is() {
    assert!(State::Playing.running());
    assert!(!State::Paused.running());
    assert!(State::Paused.in_session());
    assert!(!State::Editing.in_session());
}

#[test]
fn every_actor_of_a_world_is_listed() {
    let editing = built();
    let ids = editing.world.actor_ids();

    assert_eq!(ids.len(), 3);
    // Aucun doublon : chaque acteur une fois.
    let mut uniques = ids.clone();
    uniques.sort();
    uniques.dedup();
    assert_eq!(uniques.len(), 3);
}

#[test]
fn an_unknown_type_is_dropped_rather_than_faked() {
    // Un acteur d'un type retire du registre entre-temps ne peut pas revenir.
    // Mieux vaut le perdre que poser un acteur different a sa place.
    let mut editing = built();
    let mut session = Session::new();
    session.start(&editing);

    editing.registry = TypeRegistry::new();
    let restored = session.stop(&mut editing).unwrap();

    assert_eq!(restored, 0);
    assert_eq!(editing.world.count::<Prop>(), 0);
}

#[test]
fn a_world_of_several_types_is_captured_whole() {
    let mut editing = editing();
    editing.world.spawn(Prop {
        name: "mur".to_owned(),
        ..Prop::default()
    });
    editing.world.spawn(Light {
        position: Vec2::new(10.0, 20.0),
        radius: 64.0,
    });
    editing.world.spawn(Light {
        position: Vec2::new(30.0, 40.0),
        radius: 32.0,
    });

    let mut session = Session::new();
    session.start(&editing);

    editing.world.clear();
    let restored = session.stop(&mut editing).unwrap();

    // Un seul pool parcouru en oublierait deux sur trois.
    assert_eq!(restored, 3, "tous les types doivent revenir");
    assert_eq!(editing.world.count::<Prop>(), 1);
    assert_eq!(editing.world.count::<Light>(), 2);

    let mut rayons: Vec<f32> = editing.world.values::<Light>().map(|l| l.radius).collect();
    rayons.sort_by(f32::total_cmp);
    assert_eq!(rayons, [32.0, 64.0]);
}

#[test]
fn actor_ids_covers_every_pool() {
    let mut editing = editing();
    editing.world.spawn(Prop::default());
    editing.world.spawn(Light::default());
    editing.world.spawn(Light::default());

    assert_eq!(
        editing.world.actor_ids().len(),
        3,
        "l'iteration doit traverser tous les pools"
    );
}
