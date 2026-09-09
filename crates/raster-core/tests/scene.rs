use raster_core::reflect::{Reflect, TypeRegistry, Value};
use raster_core::{Scene, SceneError, World};
use raster_math::{IVec2, Rect, Vec2};

#[derive(Reflect, Default, Debug, PartialEq)]
struct Player {
    #[property(rename = "hp")]
    health: u32,
    #[property(readonly)]
    id: u32,
    #[property(skip)]
    cache: Vec<u32>,
    speed: f32,
    name: String,
    alive: bool,
    position: Vec2,
    tile: IVec2,
    bounds: Rect,
    tags: Vec<String>,
    nickname: Option<String>,
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Chest {
    open: bool,
    position: Vec2,
}

fn registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    registry.register::<Player>();
    registry.register::<Chest>();
    registry
}

fn sample() -> Player {
    Player {
        health: 42,
        id: 7,
        cache: vec![1, 2],
        speed: 220.5,
        name: "Nova".to_owned(),
        alive: true,
        position: Vec2::new(12.0, -3.5),
        tile: IVec2::new(4, 9),
        bounds: Rect::new(0.0, 1.0, 32.0, 48.0),
        tags: vec!["hero".to_owned(), "spawn".to_owned()],
        nickname: Some("Nov".to_owned()),
    }
}

#[test]
fn a_scene_survives_a_round_trip() {
    let mut scene = Scene::new("level_1");
    scene.add(&sample());
    scene.add(&Chest {
        open: true,
        position: Vec2::new(64.0, 0.0),
    });

    let text = scene.to_toml_string();
    let read = Scene::parse(&text, &registry()).unwrap();

    assert_eq!(read.name, "level_1");
    assert_eq!(read.len(), 2);
    assert_eq!(read.instances[0].type_name, "Player");
    assert_eq!(read.instances[1].type_name, "Chest");
}

#[test]
fn spawned_actors_carry_the_values_they_were_saved_with() {
    let mut scene = Scene::new("level_1");
    scene.add(&sample());

    let registry = registry();
    let read = Scene::parse(&scene.to_toml_string(), &registry).unwrap();

    let mut world = World::new();
    let ids = read.spawn_into(&mut world, &registry).unwrap();
    assert_eq!(ids.len(), 1);

    let player = world.get::<Player>(ids[0]).unwrap();
    assert_eq!(player.health, 42);
    assert_eq!(player.speed, 220.5);
    assert_eq!(player.name, "Nova");
    assert!(player.alive);
    assert_eq!(player.position, Vec2::new(12.0, -3.5));
    assert_eq!(player.tile, IVec2::new(4, 9));
    assert_eq!(player.bounds, Rect::new(0.0, 1.0, 32.0, 48.0));
    assert_eq!(player.tags, ["hero", "spawn"]);
    assert_eq!(player.nickname.as_deref(), Some("Nov"));
}

#[test]
fn a_readonly_field_still_loads() {
    let mut scene = Scene::new("level_1");
    scene.add(&sample());

    let registry = registry();
    let read = Scene::parse(&scene.to_toml_string(), &registry).unwrap();
    let mut world = World::new();
    let ids = read.spawn_into(&mut world, &registry).unwrap();

    assert_eq!(world.get::<Player>(ids[0]).unwrap().id, 7);
}

#[test]
fn a_skipped_field_is_never_written() {
    let mut scene = Scene::new("level_1");
    scene.add(&sample());

    assert!(!scene.to_toml_string().contains("cache"));
}

#[test]
fn only_the_fields_that_differ_from_the_defaults_are_written() {
    let mut scene = Scene::new("level_1");
    scene.add(&Chest {
        open: true,
        position: Vec2::ZERO,
    });

    let text = scene.to_toml_string();
    assert!(text.contains("open"));
    assert!(
        !text.contains("position"),
        "une valeur par defaut a ete ecrite : {text}"
    );
}

#[test]
fn an_actor_left_at_its_defaults_writes_no_field() {
    let mut scene = Scene::new("level_1");
    scene.add(&Chest::default());

    let registry = registry();
    let read = Scene::parse(&scene.to_toml_string(), &registry).unwrap();

    assert!(read.instances[0].fields.is_empty());

    let mut world = World::new();
    let ids = read.spawn_into(&mut world, &registry).unwrap();
    assert_eq!(world.get::<Chest>(ids[0]).unwrap(), &Chest::default());
}

#[test]
fn a_renamed_field_uses_its_serialized_name_on_disk() {
    let mut scene = Scene::new("level_1");
    scene.add(&sample());

    let text = scene.to_toml_string();
    assert!(text.contains("hp = 42"), "{text}");
    assert!(!text.contains("health"), "{text}");
}

#[test]
fn an_unknown_type_is_reported_by_name() {
    let toml = r#"
[scene]
name = "level_1"

[[actor]]
type = "Ghost"
"#;

    match Scene::parse(toml, &registry()) {
        Err(SceneError::UnknownType(name)) => assert_eq!(name, "Ghost"),
        other => panic!("attendu UnknownType, obtenu {other:?}"),
    }
}

#[test]
fn an_actor_without_a_type_is_malformed() {
    let toml = r#"
[[actor]]
speed = 1.0
"#;

    assert!(matches!(
        Scene::parse(toml, &registry()),
        Err(SceneError::Malformed(_))
    ));
}

#[test]
fn a_file_that_is_not_toml_is_a_parse_error() {
    assert!(matches!(
        Scene::parse("{ this is not toml", &registry()),
        Err(SceneError::Parse(_))
    ));
}

#[test]
fn an_empty_scene_loads_as_empty() {
    let scene = Scene::parse("[scene]\nname = \"vide\"\n", &registry()).unwrap();
    assert_eq!(scene.name, "vide");
    assert!(scene.is_empty());
}

#[test]
fn an_unknown_field_does_not_stop_the_load() {
    let toml = r#"
[[actor]]
type = "Chest"
open = true
sparkles = 3
"#;

    let registry = registry();
    let scene = Scene::parse(toml, &registry).unwrap();
    let mut world = World::new();
    let ids = scene.spawn_into(&mut world, &registry).unwrap();

    assert!(world.get::<Chest>(ids[0]).unwrap().open);
}

#[test]
fn a_number_written_without_a_decimal_point_still_reads_as_a_float() {
    let toml = r#"
[[actor]]
type = "Chest"
position = [3, 4]
"#;

    let registry = registry();
    let scene = Scene::parse(toml, &registry).unwrap();
    let mut world = World::new();
    let ids = scene.spawn_into(&mut world, &registry).unwrap();

    assert_eq!(
        world.get::<Chest>(ids[0]).unwrap().position,
        Vec2::new(3.0, 4.0)
    );
}

#[test]
fn a_scene_survives_a_trip_through_a_file() {
    let dir = std::env::temp_dir().join("raster-scene-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("level_1.scene.toml");

    let mut scene = Scene::new("level_1");
    scene.add(&sample());
    scene.save(&path).unwrap();

    let registry = registry();
    let read = Scene::load(&path, &registry).unwrap();
    let mut world = World::new();
    let ids = read.spawn_into(&mut world, &registry).unwrap();

    assert_eq!(
        world.get::<Player>(ids[0]).unwrap(),
        &sample_without_skipped()
    );

    std::fs::remove_file(&path).ok();
}

fn sample_without_skipped() -> Player {
    Player {
        cache: Vec::new(),
        ..sample()
    }
}

#[test]
fn a_missing_file_is_an_io_error() {
    assert!(matches!(
        Scene::load("/raster/does/not/exist.toml", &registry()),
        Err(SceneError::Io(_))
    ));
}

#[test]
fn the_registry_spawns_a_type_by_name() {
    let registry = registry();
    let mut world = World::new();

    let value = Value::Struct(
        [("open".to_owned(), Value::Bool(true))]
            .into_iter()
            .collect(),
    );

    let id = registry.spawn_into(&mut world, "Chest", &value).unwrap();
    assert!(world.get::<Chest>(id).unwrap().open);

    assert!(registry.spawn_into(&mut world, "Ghost", &value).is_none());
}
