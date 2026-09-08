//! Nested reflected structs: a component held inside an actor.
//!
//! This is the shape every actor takes — `Player { sprite: Sprite, body: Body }`
//! — so it has to work before the actor model can be built on top.

use raster_core::reflect::{Reflect, Value, ValueKind};
use raster_math::Vec2;
use std::collections::BTreeMap;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Sprite {
    frame: u32,
    tint: String,
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Body {
    velocity: Vec2,
    #[property(min = 0.0)]
    gravity_scale: f32,
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Player {
    sprite: Sprite,
    body: Body,
    speed: f32,
}

#[test]
fn a_nested_struct_is_described_by_name() {
    let info = Player::type_info();

    assert_eq!(
        info.field("sprite").unwrap().kind,
        ValueKind::Struct("Sprite")
    );
    assert_eq!(info.field("body").unwrap().kind, ValueKind::Struct("Body"));
    assert_eq!(info.field("speed").unwrap().kind, ValueKind::Float);
}

#[test]
fn reading_a_component_yields_its_whole_value() {
    let player = Player {
        sprite: Sprite {
            frame: 3,
            tint: "red".into(),
        },
        ..Default::default()
    };

    let value = player.get_field("sprite").expect("sprite is reflected");
    let fields = value.as_struct().expect("a struct");

    assert_eq!(fields.get("frame"), Some(&Value::Int(3)));
    assert_eq!(fields.get("tint"), Some(&Value::Str("red".into())));
}

#[test]
fn writing_a_component_replaces_it() {
    let mut player = Player::default();

    let mut sprite = BTreeMap::new();
    sprite.insert("frame".to_string(), Value::Int(7));
    sprite.insert("tint".to_string(), Value::Str("blue".into()));

    player.set_field("sprite", Value::Struct(sprite)).unwrap();

    assert_eq!(player.sprite.frame, 7);
    assert_eq!(player.sprite.tint, "blue");
}

/// A scene file rarely spells out every field of every component; the ones it
/// omits should keep their defaults rather than failing the load.
#[test]
fn a_partial_component_keeps_its_defaults() {
    let mut player = Player::default();

    let mut sprite = BTreeMap::new();
    sprite.insert("frame".to_string(), Value::Int(5));

    player.set_field("sprite", Value::Struct(sprite)).unwrap();

    assert_eq!(player.sprite.frame, 5);
    assert_eq!(player.sprite.tint, "", "the omitted field lost its default");
}

#[test]
fn a_whole_actor_survives_a_round_trip() {
    let original = Player {
        sprite: Sprite {
            frame: 3,
            tint: "red".into(),
        },
        body: Body {
            velocity: Vec2::new(1.0, -2.0),
            gravity_scale: 0.5,
        },
        speed: 90.0,
    };

    let mut restored = Player::default();
    restored.apply(&original.to_value()).unwrap();

    assert_eq!(restored, original);
}

/// The constraint belongs to the component that declares it, wherever that
/// component is used.
#[test]
fn bounds_inside_a_component_still_apply() {
    let mut player = Player::default();

    let mut body = BTreeMap::new();
    body.insert("gravity_scale".to_string(), Value::Float(-5.0));

    player.set_field("body", Value::Struct(body)).unwrap();
    assert_eq!(
        player.body.gravity_scale, 0.0,
        "the minimum was not enforced"
    );
}

#[test]
fn a_wrong_type_for_a_component_is_refused() {
    let mut player = Player::default();
    assert!(player.set_field("sprite", Value::Int(3)).is_err());
    assert_eq!(player.sprite, Sprite::default());
}

#[test]
fn an_error_inside_a_component_is_reported() {
    let mut player = Player::default();

    let mut sprite = BTreeMap::new();
    sprite.insert("frame".to_string(), Value::Str("not a number".into()));

    assert!(player.set_field("sprite", Value::Struct(sprite)).is_err());
}

#[test]
fn nested_values_are_reachable_by_path() {
    let player = Player {
        sprite: Sprite {
            frame: 3,
            tint: "red".into(),
        },
        body: Body {
            velocity: Vec2::new(1.0, -2.0),
            gravity_scale: 0.5,
        },
        speed: 90.0,
    };

    let value = player.to_value();

    assert_eq!(value.path("sprite.frame"), Some(&Value::Int(3)));
    assert_eq!(value.path("body.gravity_scale"), Some(&Value::Float(0.5)));
    assert_eq!(value.path("sprite.missing"), None);
}

/// Two levels of nesting, since an actor may hold a component that itself holds
/// one.
#[test]
fn nesting_works_more_than_one_level_deep() {
    #[derive(Reflect, Default, Debug, PartialEq)]
    struct Inner {
        value: i32,
    }

    #[derive(Reflect, Default, Debug, PartialEq)]
    struct Middle {
        inner: Inner,
    }

    #[derive(Reflect, Default, Debug, PartialEq)]
    struct Outer {
        middle: Middle,
    }

    let original = Outer {
        middle: Middle {
            inner: Inner { value: 42 },
        },
    };

    let mut restored = Outer::default();
    restored.apply(&original.to_value()).unwrap();

    assert_eq!(restored, original);
    assert_eq!(
        original.to_value().path("middle.inner.value"),
        Some(&Value::Int(42))
    );
}

/// A component inside a list — an inventory, a list of waypoints.
#[test]
fn a_list_of_components_round_trips() {
    #[derive(Reflect, Default, Debug, PartialEq)]
    struct Item {
        name: String,
        count: u32,
    }

    #[derive(Reflect, Default, Debug, PartialEq)]
    struct Inventory {
        items: Vec<Item>,
    }

    let original = Inventory {
        items: vec![
            Item {
                name: "torch".into(),
                count: 3,
            },
            Item {
                name: "rope".into(),
                count: 1,
            },
        ],
    };

    let mut restored = Inventory::default();
    restored.apply(&original.to_value()).unwrap();

    assert_eq!(restored, original);
}
