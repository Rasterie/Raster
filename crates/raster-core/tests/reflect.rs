use raster_core::reflect::{Reflect, ReflectError, TypeRegistry, Value, ValueKind};
use raster_math::{IVec2, Rect, Vec2};
use std::collections::BTreeMap;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Player {
    #[property(min = 0.0, max = 500.0, tooltip = "Pixels per second")]
    speed: f32,

    #[property(rename = "hp")]
    health: u32,

    #[property(readonly)]
    id: u32,

    #[property(skip)]
    cache: Vec<u32>,

    name: String,
    alive: bool,
    position: Vec2,
    tile: IVec2,
    bounds: Rect,
    tags: Vec<String>,
    nickname: Option<String>,
}

#[test]
fn type_info_lists_every_field_but_the_skipped_one() {
    let info = Player::type_info();
    assert_eq!(info.name, "Player");

    let names: Vec<_> = info.fields.iter().map(|f| f.name).collect();
    assert_eq!(
        names,
        [
            "speed", "health", "id", "name", "alive", "position", "tile", "bounds", "tags",
            "nickname"
        ]
    );
    assert!(!names.contains(&"cache"), "skipped field was reflected");
}

#[test]
fn field_kinds_match_their_rust_types() {
    let info = Player::type_info();

    assert_eq!(info.field("speed").unwrap().kind, ValueKind::Float);
    assert_eq!(info.field("health").unwrap().kind, ValueKind::Int);
    assert_eq!(info.field("name").unwrap().kind, ValueKind::Str);
    assert_eq!(info.field("alive").unwrap().kind, ValueKind::Bool);
    assert_eq!(info.field("position").unwrap().kind, ValueKind::Vec2);
    assert_eq!(info.field("tile").unwrap().kind, ValueKind::IVec2);
    assert_eq!(info.field("bounds").unwrap().kind, ValueKind::Rect);
    assert_eq!(
        info.field("tags").unwrap().kind,
        ValueKind::List(&ValueKind::Str)
    );
    assert_eq!(
        info.field("nickname").unwrap().kind,
        ValueKind::Option(&ValueKind::Str)
    );
}

#[test]
fn attributes_reach_the_field_info() {
    let info = Player::type_info();

    let speed = info.field("speed").unwrap();
    assert_eq!(speed.attrs.min, Some(0.0));
    assert_eq!(speed.attrs.max, Some(500.0));
    assert_eq!(speed.attrs.tooltip, Some("Pixels per second"));

    assert!(info.field("id").unwrap().attrs.readonly);
    assert!(!info.field("speed").unwrap().attrs.readonly);
}

#[test]
fn every_field_round_trips() {
    let mut p = Player::default();

    p.set_field("speed", Value::Float(90.0)).unwrap();
    p.set_field("health", Value::Int(100)).unwrap();
    p.set_field("name", Value::Str("Hero".into())).unwrap();
    p.set_field("alive", Value::Bool(true)).unwrap();
    p.set_field("position", Value::Vec2(Vec2::new(3.0, 4.0)))
        .unwrap();
    p.set_field("tile", Value::IVec2(IVec2::new(1, 2))).unwrap();
    p.set_field("bounds", Value::Rect(Rect::new(0.0, 0.0, 8.0, 8.0)))
        .unwrap();

    assert_eq!(p.get_field("speed"), Some(Value::Float(90.0)));
    assert_eq!(p.get_field("health"), Some(Value::Int(100)));
    assert_eq!(p.get_field("name"), Some(Value::Str("Hero".into())));
    assert_eq!(p.get_field("alive"), Some(Value::Bool(true)));
    assert_eq!(
        p.get_field("position"),
        Some(Value::Vec2(Vec2::new(3.0, 4.0)))
    );
    assert_eq!(p.get_field("tile"), Some(Value::IVec2(IVec2::new(1, 2))));
    assert_eq!(
        p.get_field("bounds"),
        Some(Value::Rect(Rect::new(0.0, 0.0, 8.0, 8.0)))
    );
}

#[test]
fn lists_and_options_round_trip() {
    let mut p = Player::default();

    p.set_field(
        "tags",
        Value::List(vec![Value::Str("a".into()), Value::Str("b".into())]),
    )
    .unwrap();
    assert_eq!(p.tags, vec!["a".to_string(), "b".to_string()]);

    p.set_field("nickname", Value::Str("Ace".into())).unwrap();
    assert_eq!(p.nickname.as_deref(), Some("Ace"));

    p.set_field("nickname", Value::None).unwrap();
    assert_eq!(p.nickname, None);
    assert_eq!(p.get_field("nickname"), Some(Value::None));
}

/// An inspector slider dragged past its end should stop at the limit, not
/// refuse the edit.
#[test]
fn bounds_clamp_rather_than_reject() {
    let mut p = Player::default();

    p.set_field("speed", Value::Float(9000.0)).unwrap();
    assert_eq!(p.speed, 500.0);

    p.set_field("speed", Value::Float(-100.0)).unwrap();
    assert_eq!(p.speed, 0.0);
}

#[test]
fn readonly_fields_refuse_writes() {
    let mut p = Player::default();
    let err = p.set_field("id", Value::Int(7)).unwrap_err();

    assert!(matches!(err, ReflectError::Readonly { .. }), "got {err:?}");
    assert_eq!(p.id, 0);
}

#[test]
fn a_skipped_field_is_invisible_to_reflection() {
    let mut p = Player::default();

    assert_eq!(p.get_field("cache"), None);
    assert!(matches!(
        p.set_field("cache", Value::List(vec![])),
        Err(ReflectError::UnknownField { .. })
    ));
}

#[test]
fn unknown_fields_are_reported_by_name() {
    let mut p = Player::default();

    match p.set_field("nonexistent", Value::Int(1)) {
        Err(ReflectError::UnknownField { type_name, field }) => {
            assert_eq!(type_name, "Player");
            assert_eq!(field, "nonexistent");
        }
        other => panic!("expected UnknownField, got {other:?}"),
    }
}

#[test]
fn a_wrong_type_is_refused() {
    let mut p = Player::default();
    let err = p.set_field("speed", Value::Str("fast".into())).unwrap_err();

    assert!(
        matches!(err, ReflectError::TypeMismatch { .. }),
        "got {err:?}"
    );
    assert_eq!(p.speed, 0.0, "the field was modified despite the error");
}

/// A scene file writes `speed = 90` for a whole number; refusing to read that
/// back into a float would make hand-edited files fragile.
#[test]
fn an_integer_is_accepted_for_a_float_field() {
    let mut p = Player::default();
    p.set_field("speed", Value::Int(90)).unwrap();
    assert_eq!(p.speed, 90.0);
}

/// Writing 300 into a u8 must fail loudly rather than wrap around to 44.
#[test]
fn an_out_of_range_integer_is_refused() {
    #[derive(Reflect, Default)]
    struct Small {
        tiny: u8,
    }

    let mut s = Small::default();
    assert!(s.set_field("tiny", Value::Int(300)).is_err());
    assert!(s.set_field("tiny", Value::Int(-1)).is_err());
    assert_eq!(s.tiny, 0);

    s.set_field("tiny", Value::Int(255)).unwrap();
    assert_eq!(s.tiny, 255);
}

/// A NaN position propagates through every calculation it touches and is
/// painful to trace back to the file that introduced it.
#[test]
fn nan_and_infinity_are_refused() {
    let mut p = Player::default();

    assert!(p.set_field("speed", Value::Float(f64::NAN)).is_err());
    assert!(p.set_field("speed", Value::Float(f64::INFINITY)).is_err());
    assert_eq!(p.speed, 0.0);
}

/// The rename is what lets a field be renamed in code without invalidating
/// every scene file that references it.
#[test]
fn renamed_fields_serialise_under_their_new_name() {
    let info = Player::type_info();
    let health = info.field("health").unwrap();

    assert_eq!(health.name, "health");
    assert_eq!(health.serialized_name, "hp");
    assert_eq!(info.field_by_serialized_name("hp").unwrap().name, "health");

    let p = Player {
        health: 42,
        ..Default::default()
    };
    let value = p.to_value();
    let fields = value.as_struct().unwrap();

    assert!(
        fields.contains_key("hp"),
        "serialised under the Rust name instead"
    );
    assert!(!fields.contains_key("health"));
}

/// `to_value` writes under the serialised name, so `apply` must read under it
/// too. Getting this wrong silently drops every renamed field when a scene
/// loads — the value is written, then never read back.
#[test]
fn a_renamed_field_survives_the_round_trip() {
    let original = Player {
        health: 100,
        ..Default::default()
    };

    let mut restored = Player::default();
    restored.apply(&original.to_value()).unwrap();

    assert_eq!(restored.health, 100);
}

/// `readonly` means "the inspector must not offer this for editing", not
/// "this value cannot be restored" — otherwise a saved value would be lost on
/// every load.
#[test]
fn a_readonly_field_still_survives_the_round_trip() {
    let original = Player {
        id: 7,
        ..Default::default()
    };

    let mut restored = Player::default();
    restored.apply(&original.to_value()).unwrap();

    assert_eq!(restored.id, 7);
    // But a direct write is still refused.
    assert!(restored.set_field("id", Value::Int(9)).is_err());
}

#[test]
fn to_value_captures_the_whole_struct() {
    let p = Player {
        speed: 90.0,
        name: "Hero".into(),
        ..Default::default()
    };
    let fields = p.to_value().as_struct().unwrap().clone();

    assert_eq!(fields.get("speed"), Some(&Value::Float(90.0)));
    assert_eq!(fields.get("name"), Some(&Value::Str("Hero".into())));
    assert!(
        !fields.contains_key("cache"),
        "skipped field was serialised"
    );
}

/// A scene written by a newer version must still load in an older one, minus
/// what it cannot understand — the alternative is refusing the file entirely.
#[test]
fn apply_ignores_fields_it_does_not_know() {
    let mut p = Player::default();

    let mut fields = BTreeMap::new();
    fields.insert("speed".to_string(), Value::Float(120.0));
    fields.insert("from_a_future_version".to_string(), Value::Int(1));

    p.apply(&Value::Struct(fields)).unwrap();
    assert_eq!(p.speed, 120.0);
}

#[test]
fn apply_still_reports_a_real_type_error() {
    let mut p = Player::default();

    let mut fields = BTreeMap::new();
    fields.insert("speed".to_string(), Value::Str("fast".into()));

    assert!(p.apply(&Value::Struct(fields)).is_err());
}

#[test]
fn apply_refuses_a_non_struct() {
    let mut p = Player::default();
    assert!(p.apply(&Value::Int(3)).is_err());
}

#[test]
fn a_value_survives_a_full_round_trip() {
    let original = Player {
        speed: 90.0,
        health: 100,
        name: "Hero".into(),
        alive: true,
        position: Vec2::new(3.0, 4.0),
        tags: vec!["fast".into()],
        nickname: Some("Ace".into()),
        ..Default::default()
    };

    let mut restored = Player::default();
    restored.apply(&original.to_value()).unwrap();

    // `cache` is skipped, so it is not expected to survive.
    assert_eq!(restored.speed, original.speed);
    assert_eq!(restored.health, original.health);
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.alive, original.alive);
    assert_eq!(restored.position, original.position);
    assert_eq!(restored.tags, original.tags);
    assert_eq!(restored.nickname, original.nickname);
}

/// Serialising the same value twice must produce identical output, or scene
/// files would churn in version control for no reason.
#[test]
fn serialisation_order_is_stable() {
    let p = Player::default();
    let first = p.to_value().to_string();

    for _ in 0..20 {
        assert_eq!(Player::default().to_value().to_string(), first);
    }
}

#[test]
fn nested_values_are_reachable_by_path() {
    let mut outer = BTreeMap::new();
    let mut inner = BTreeMap::new();
    inner.insert("x".to_string(), Value::Float(3.0));
    outer.insert("body".to_string(), Value::Struct(inner));
    outer.insert(
        "tags".to_string(),
        Value::List(vec![Value::Str("a".into())]),
    );

    let value = Value::Struct(outer);

    assert_eq!(value.path("body.x"), Some(&Value::Float(3.0)));
    assert_eq!(value.path("tags.0"), Some(&Value::Str("a".into())));
    assert_eq!(value.path("body.missing"), None);
    assert_eq!(value.path("tags.9"), None);
}

// --- Registry ---------------------------------------------------------------

#[derive(Reflect, Default, Debug, PartialEq)]
struct Chest {
    contents: Vec<String>,
    locked: bool,
}

#[test]
fn a_registered_type_can_be_built_from_its_name() {
    let mut registry = TypeRegistry::new();
    registry.register::<Player>();
    registry.register::<Chest>();

    assert_eq!(registry.len(), 2);
    assert!(registry.contains("Player"));
    assert_eq!(registry.info("Chest").unwrap().name, "Chest");

    let object = registry.construct("Player").expect("registered");
    assert_eq!(object.type_info().name, "Player");
}

#[test]
fn an_unregistered_type_yields_none() {
    let registry = TypeRegistry::new();
    assert!(registry.construct("Player").is_none());
    assert!(registry.info("Player").is_none());
    assert!(!registry.contains("Player"));
}

/// This is what a scene loader does for every actor it reads.
#[test]
fn construct_from_builds_and_fills_in_one_step() {
    let mut registry = TypeRegistry::new();
    registry.register::<Player>();

    let mut fields = BTreeMap::new();
    fields.insert("speed".to_string(), Value::Float(120.0));
    fields.insert("name".to_string(), Value::Str("Loaded".into()));

    let object = registry
        .construct_from("Player", &Value::Struct(fields))
        .unwrap();
    let player = object.as_any().downcast_ref::<Player>().expect("downcast");

    assert_eq!(player.speed, 120.0);
    assert_eq!(player.name, "Loaded");
}

#[test]
fn construct_from_reports_an_unregistered_type() {
    let registry = TypeRegistry::new();
    assert!(
        registry
            .construct_from("Ghost", &Value::Struct(BTreeMap::new()))
            .is_err()
    );
}

#[test]
fn registered_names_come_back_in_a_stable_order() {
    let mut registry = TypeRegistry::new();
    registry.register::<Player>();
    registry.register::<Chest>();

    assert_eq!(registry.names().collect::<Vec<_>>(), ["Chest", "Player"]);
}

#[test]
fn registering_twice_replaces_rather_than_duplicates() {
    let mut registry = TypeRegistry::new();
    registry.register::<Player>();
    registry.register::<Player>();
    assert_eq!(registry.len(), 1);
}

#[test]
fn a_constructed_object_is_mutable_through_reflection() {
    let mut registry = TypeRegistry::new();
    registry.register::<Chest>();

    let mut object = registry.construct("Chest").unwrap();
    object.set_field("locked", Value::Bool(true)).unwrap();

    assert_eq!(object.get_field("locked"), Some(Value::Bool(true)));
    assert!(object.as_any_mut().downcast_mut::<Chest>().unwrap().locked);
}
