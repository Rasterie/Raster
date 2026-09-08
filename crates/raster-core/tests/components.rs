//! The renderer must reach every `Sprite` without knowing which actor holds it.
//!
//! Reflection can find those fields but only hands back an owned `Value` — an
//! allocation per actor per frame. These tests cover the fast path.

use raster_core::World;
use raster_core::actor::{Component, HasComponent};
use raster_core::reflect::Reflect;
use raster_math::Vec2;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Sprite {
    frame: u32,
}

impl Component for Sprite {}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Body {
    velocity: Vec2,
}

impl Component for Body {}

#[derive(Reflect, Default)]
struct Player {
    #[property(component)]
    sprite: Sprite,
    #[property(component)]
    body: Body,
    speed: f32,
}

/// An actor may hold two of the same component — a chest and its lid.
#[derive(Reflect, Default)]
struct Chest {
    #[property(component)]
    sprite: Sprite,
    #[property(component)]
    lid: Sprite,
}

/// An actor with no components at all must not break the iteration.
#[derive(Reflect, Default)]
struct Trigger {
    radius: f32,
}

fn world_with_actors() -> World {
    let mut world = World::new();
    world.register_component::<Player, Sprite>();
    world.register_component::<Player, Body>();
    world.register_component::<Chest, Sprite>();

    world.spawn(Player {
        sprite: Sprite { frame: 1 },
        body: Body {
            velocity: Vec2::new(1.0, 0.0),
        },
        speed: 90.0,
    });
    world.spawn(Player {
        sprite: Sprite { frame: 2 },
        body: Body {
            velocity: Vec2::new(0.0, 1.0),
        },
        speed: 60.0,
    });
    world.spawn(Chest {
        sprite: Sprite { frame: 10 },
        lid: Sprite { frame: 11 },
    });
    world.spawn(Trigger { radius: 5.0 });

    world
}

#[test]
fn every_component_is_reached_whatever_actor_holds_it() {
    let world = world_with_actors();

    let mut frames = Vec::new();
    world.each_component::<Sprite>(|_, sprite| frames.push(sprite.frame));
    frames.sort_unstable();

    assert_eq!(frames, vec![1, 2, 10, 11]);
}

/// The trait yields an iterator rather than one reference precisely so an
/// actor can hold several of the same component.
#[test]
fn an_actor_holding_two_of_a_component_yields_both() {
    let chest = Chest {
        sprite: Sprite { frame: 10 },
        lid: Sprite { frame: 11 },
    };

    let frames: Vec<u32> = HasComponent::<Sprite>::components(&chest)
        .map(|s| s.frame)
        .collect();
    assert_eq!(frames, vec![10, 11]);
}

#[test]
fn each_component_reports_the_owning_actor() {
    let mut world = World::new();
    world.register_component::<Player, Sprite>();

    let id = world.spawn(Player {
        sprite: Sprite { frame: 7 },
        ..Default::default()
    });

    let mut seen = Vec::new();
    world.each_component::<Sprite>(|owner, sprite| seen.push((owner, sprite.frame)));

    assert_eq!(seen, vec![(id, 7)]);
    assert!(
        world.get::<Player>(seen[0].0).is_some(),
        "the id did not resolve"
    );
}

/// Two component types must not be confused: asking for one must not yield the
/// other, even though both are erased to `&dyn Any` on the way through.
#[test]
fn component_types_do_not_leak_into_each_other() {
    let world = world_with_actors();

    let mut sprites = 0;
    world.each_component::<Sprite>(|_, _| sprites += 1);

    let mut bodies = 0;
    world.each_component::<Body>(|_, _| bodies += 1);

    assert_eq!(sprites, 4, "two players, a chest and its lid");
    assert_eq!(bodies, 2, "one per player");
}

#[test]
fn a_despawned_actor_stops_contributing_its_components() {
    let mut world = World::new();
    world.register_component::<Player, Sprite>();

    let first = world.spawn(Player {
        sprite: Sprite { frame: 1 },
        ..Default::default()
    });
    world.spawn(Player {
        sprite: Sprite { frame: 2 },
        ..Default::default()
    });

    world.despawn(first);

    let mut frames = Vec::new();
    world.each_component::<Sprite>(|_, s| frames.push(s.frame));

    assert_eq!(frames, vec![2]);
}

/// An actor type that holds no components of the requested kind is simply
/// skipped — it must not abort the whole iteration.
#[test]
fn an_actor_without_the_component_is_skipped() {
    let world = world_with_actors();

    let mut count = 0;
    world.each_component::<Body>(|_, _| count += 1);

    // Chest and Trigger hold no Body; only the two players do.
    assert_eq!(count, 2);
}

/// Registration is explicit, so forgetting it silently skips that actor type.
/// This test exists to make that behaviour deliberate rather than surprising.
#[test]
fn an_unregistered_actor_type_contributes_nothing() {
    let mut world = World::new();
    // Player is spawned but never registered for Sprite.
    world.spawn(Player {
        sprite: Sprite { frame: 1 },
        ..Default::default()
    });

    let mut count = 0;
    world.each_component::<Sprite>(|_, _| count += 1);

    assert_eq!(count, 0, "an unregistered type contributed components");
}

#[test]
fn registering_a_type_that_never_spawns_is_harmless() {
    let mut world = World::new();
    world.register_component::<Player, Sprite>();

    let mut count = 0;
    world.each_component::<Sprite>(|_, _| count += 1);

    assert_eq!(count, 0);
    assert_eq!(world.type_count(), 1, "registration should create the pool");
}

#[test]
fn asking_for_a_component_nobody_holds_yields_nothing() {
    #[derive(Reflect, Default)]
    struct Unused {
        value: u32,
    }
    impl Component for Unused {}

    let world = world_with_actors();

    let mut count = 0;
    world.each_component::<Unused>(|_, _| count += 1);
    assert_eq!(count, 0);
}

#[test]
fn clearing_the_world_leaves_no_components() {
    let mut world = world_with_actors();
    world.clear();

    let mut count = 0;
    world.each_component::<Sprite>(|_, _| count += 1);
    assert_eq!(count, 0);
}

/// The renderer will do this every frame, so it must stay correct at scale and
/// must not allocate per actor — the callback exists for that reason.
#[test]
fn iteration_holds_up_at_scale() {
    let mut world = World::new();
    world.register_component::<Player, Sprite>();
    world.register_component::<Chest, Sprite>();

    for i in 0..5_000 {
        world.spawn(Player {
            sprite: Sprite { frame: i },
            ..Default::default()
        });
        world.spawn(Chest {
            sprite: Sprite { frame: i },
            lid: Sprite { frame: i },
        });
    }

    let mut count = 0u32;
    let mut sum = 0u64;
    world.each_component::<Sprite>(|_, sprite| {
        count += 1;
        sum += u64::from(sprite.frame);
    });

    // 5 000 players with one sprite, 5 000 chests with two.
    assert_eq!(count, 15_000);
    assert_eq!(sum, 3 * (0..5_000u64).sum::<u64>());
}
