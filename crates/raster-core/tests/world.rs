use raster_core::reflect::Reflect;
use raster_core::{ActorId, Behaviour, World};
use raster_math::Vec2;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Player {
    position: Vec2,
    health: u32,
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Enemy {
    position: Vec2,
    target: u32,
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Ticker {
    ticks: u32,
    fixed_ticks: u32,
    spawned: bool,
    despawned: bool,
}

impl Behaviour for Ticker {
    fn tick(&mut self, _dt: f32) {
        self.ticks += 1;
    }
    fn fixed_tick(&mut self, _dt: f32) {
        self.fixed_ticks += 1;
    }
    fn on_spawn(&mut self) {
        self.spawned = true;
    }
    fn on_despawn(&mut self) {
        self.despawned = true;
    }
}

#[test]
fn a_spawned_actor_can_be_read_back() {
    let mut world = World::new();
    let id = world.spawn(Player {
        health: 100,
        ..Default::default()
    });

    assert_eq!(world.get::<Player>(id).unwrap().health, 100);
    assert!(world.contains(id));
    assert_eq!(world.len(), 1);
}

#[test]
fn an_actor_can_be_modified_in_place() {
    let mut world = World::new();
    let id = world.spawn(Player::default());

    world.get_mut::<Player>(id).unwrap().health = 50;
    assert_eq!(world.get::<Player>(id).unwrap().health, 50);
}

#[test]
fn a_despawned_actor_is_gone() {
    let mut world = World::new();
    let id = world.spawn(Player::default());

    assert!(world.despawn(id));
    assert!(world.get::<Player>(id).is_none());
    assert!(!world.contains(id));
    assert_eq!(world.len(), 0);
}

/// Two systems deciding to destroy the same enemy in one frame is normal, so a
/// second despawn reports failure rather than panicking.
#[test]
fn despawning_twice_is_a_no_op() {
    let mut world = World::new();
    let id = world.spawn(Player::default());

    assert!(world.despawn(id));
    assert!(!world.despawn(id));
}

/// The whole point of the generation. Without it, an enemy holding the id of a
/// dead player would silently start reading whichever actor took its slot.
#[test]
fn a_stale_id_does_not_resolve_to_the_actor_that_replaced_it() {
    let mut world = World::new();

    let first = world.spawn(Player {
        health: 100,
        ..Default::default()
    });
    world.despawn(first);

    // The new actor almost certainly reuses the freed slot.
    let second = world.spawn(Player {
        health: 50,
        ..Default::default()
    });
    assert_eq!(first.index(), second.index(), "the slot was not reused");

    assert!(
        world.get::<Player>(first).is_none(),
        "the stale id resolved"
    );
    assert!(!world.contains(first));
    assert_eq!(world.get::<Player>(second).unwrap().health, 50);
}

#[test]
fn a_reused_slot_gets_a_new_generation() {
    let mut world = World::new();

    let first = world.spawn(Player::default());
    world.despawn(first);
    let second = world.spawn(Player::default());

    assert_eq!(first.index(), second.index());
    assert_ne!(first.generation(), second.generation());
}

#[test]
fn different_types_live_in_different_pools() {
    let mut world = World::new();

    let player = world.spawn(Player::default());
    let enemy = world.spawn(Enemy::default());

    assert_eq!(world.type_count(), 2);
    assert_ne!(player.type_tag(), enemy.type_tag());

    assert_eq!(world.count::<Player>(), 1);
    assert_eq!(world.count::<Enemy>(), 1);
    assert_eq!(world.len(), 2);
}

/// A `Player` id must not be readable as an `Enemy`, even if the indices line
/// up — which they will, since each pool numbers its slots from zero.
#[test]
fn an_id_from_one_pool_does_not_resolve_in_another() {
    let mut world = World::new();

    let player = world.spawn(Player::default());
    let enemy = world.spawn(Enemy::default());
    assert_eq!(
        player.index(),
        enemy.index(),
        "the pools did not both start at zero"
    );

    assert!(world.get::<Enemy>(player).is_none());
    assert!(world.get::<Player>(enemy).is_none());
}

#[test]
fn despawning_reaches_the_right_pool_without_knowing_the_type() {
    let mut world = World::new();

    let player = world.spawn(Player::default());
    let enemy = world.spawn(Enemy::default());

    assert!(world.despawn(enemy));
    assert_eq!(world.count::<Enemy>(), 0);
    assert_eq!(world.count::<Player>(), 1, "the wrong pool was touched");
    assert!(world.get::<Player>(player).is_some());
}

#[test]
fn iteration_yields_every_actor_of_a_type_with_its_id() {
    let mut world = World::new();
    let ids: Vec<ActorId> = (0..5)
        .map(|i| {
            world.spawn(Player {
                health: i,
                ..Default::default()
            })
        })
        .collect();

    let mut seen: Vec<(ActorId, u32)> = world
        .iter::<Player>()
        .map(|(id, a)| (id, a.health))
        .collect();
    seen.sort_by_key(|(id, _)| id.index());

    assert_eq!(seen.len(), 5);
    for (i, (id, health)) in seen.iter().enumerate() {
        assert_eq!(*id, ids[i]);
        assert_eq!(*health, i as u32);
    }
}

#[test]
fn iteration_skips_despawned_actors() {
    let mut world = World::new();
    let ids: Vec<_> = (0..5).map(|_| world.spawn(Player::default())).collect();

    world.despawn(ids[1]);
    world.despawn(ids[3]);

    assert_eq!(world.iter::<Player>().count(), 3);
    assert_eq!(world.count::<Player>(), 3);
}

#[test]
fn mutable_iteration_reaches_every_actor() {
    let mut world = World::new();
    for _ in 0..5 {
        world.spawn(Player::default());
    }

    for (_, player) in world.iter_mut::<Player>() {
        player.health = 42;
    }

    assert!(world.values::<Player>().all(|p| p.health == 42));
}

/// The ids handed out by a mutable iteration must be usable afterwards, or the
/// iteration is only half useful.
#[test]
fn ids_from_mutable_iteration_stay_valid() {
    let mut world = World::new();
    for _ in 0..3 {
        world.spawn(Player::default());
    }

    let ids: Vec<_> = world.iter_mut::<Player>().map(|(id, _)| id).collect();
    for id in ids {
        assert!(world.get::<Player>(id).is_some(), "{id:?} did not resolve");
    }
}

#[test]
fn iterating_a_type_that_was_never_spawned_yields_nothing() {
    let world = World::new();

    assert_eq!(world.iter::<Player>().count(), 0);
    assert_eq!(world.count::<Player>(), 0);
    assert!(world.is_empty());
}

#[test]
fn tick_runs_on_every_actor_of_the_type() {
    let mut world = World::new();
    for _ in 0..3 {
        world.spawn(Ticker::default());
    }

    world.tick::<Ticker>(0.016);
    world.tick::<Ticker>(0.016);
    world.fixed_tick::<Ticker>(0.016);

    for ticker in world.values::<Ticker>() {
        assert_eq!(ticker.ticks, 2);
        assert_eq!(ticker.fixed_ticks, 1);
    }
}

#[test]
fn the_spawn_hook_runs_when_asked_for() {
    let mut world = World::new();

    let plain = world.spawn(Ticker::default());
    assert!(
        !world.get::<Ticker>(plain).unwrap().spawned,
        "the hook ran unbidden"
    );

    let hooked = world.spawn_with_hook(Ticker::default());
    assert!(world.get::<Ticker>(hooked).unwrap().spawned);
}

#[test]
fn the_despawn_hook_runs_before_the_actor_is_dropped() {
    let mut world = World::new();
    let id = world.spawn(Ticker::default());

    assert!(world.despawn_with_hook::<Ticker>(id));
    assert!(world.get::<Ticker>(id).is_none());
    assert!(
        !world.despawn_with_hook::<Ticker>(id),
        "a second despawn succeeded"
    );
}

#[test]
fn clearing_removes_every_actor() {
    let mut world = World::new();
    let id = world.spawn(Player::default());
    world.spawn(Enemy::default());

    world.clear();

    assert!(world.is_empty());
    assert!(world.get::<Player>(id).is_none());
    assert_eq!(world.type_count(), 2, "the pools were dropped, not cleared");
}

/// Clearing must not reset the generation counters, or an id kept from before
/// a level change would resolve to whichever actor now holds its slot.
#[test]
fn ids_from_before_a_clear_do_not_resolve_after_it() {
    let mut world = World::new();
    let old = world.spawn(Player {
        health: 100,
        ..Default::default()
    });

    world.clear();
    let new = world.spawn(Player {
        health: 1,
        ..Default::default()
    });

    assert_eq!(old.index(), new.index(), "the slot was not reused");
    assert_ne!(
        old.generation(),
        new.generation(),
        "the generation was reset"
    );

    assert!(
        world.get::<Player>(old).is_none(),
        "an id from before the clear resolved"
    );
    assert_eq!(world.get::<Player>(new).unwrap().health, 1);
}

/// The same, across several clears, since a level may be reloaded repeatedly.
#[test]
fn generations_keep_advancing_across_repeated_clears() {
    let mut world = World::new();
    let mut seen = Vec::new();

    for _ in 0..10 {
        seen.push(world.spawn(Player::default()));
        world.clear();
    }

    // Every id ever handed out must now be dead.
    for id in &seen {
        assert!(
            world.get::<Player>(*id).is_none(),
            "{id:?} survived a clear"
        );
    }

    // And they must all be distinct.
    let generations: std::collections::BTreeSet<_> =
        seen.iter().map(|id| id.generation()).collect();
    assert_eq!(generations.len(), seen.len(), "a generation was reused");
}

#[test]
fn slots_are_reused_rather_than_growing_without_bound() {
    let mut world = World::new();

    for _ in 0..100 {
        let id = world.spawn(Player::default());
        world.despawn(id);
    }

    assert_eq!(world.len(), 0);

    // Every spawn reused the same freed slot.
    let id = world.spawn(Player::default());
    assert_eq!(id.index(), 0, "slots were not reused");
}

#[test]
fn actors_can_reference_each_other_by_id() {
    let mut world = World::new();

    let player = world.spawn(Player {
        health: 100,
        ..Default::default()
    });
    let enemy = world.spawn(Enemy {
        target: player.index(),
        ..Default::default()
    });

    let target_index = world.get::<Enemy>(enemy).unwrap().target;
    assert_eq!(target_index, player.index());
}

/// `Option<ActorId>` costing no more than `ActorId` is why the generation is
/// non-zero — an actor holding an optional target pays nothing for it.
#[test]
fn an_optional_id_is_no_larger_than_an_id() {
    assert_eq!(size_of::<Option<ActorId>>(), size_of::<ActorId>());
    assert_eq!(size_of::<ActorId>(), 12);
}

#[test]
fn many_actors_of_many_types_coexist() {
    let mut world = World::new();

    for i in 0..1_000 {
        world.spawn(Player {
            health: i,
            ..Default::default()
        });
        world.spawn(Enemy {
            target: i,
            ..Default::default()
        });
    }

    assert_eq!(world.count::<Player>(), 1_000);
    assert_eq!(world.count::<Enemy>(), 1_000);
    assert_eq!(world.len(), 2_000);

    let sum: u32 = world.values::<Player>().map(|p| p.health).sum();
    assert_eq!(sum, (0..1_000).sum::<u32>());
}

#[test]
fn spawning_after_a_clear_works() {
    // Changer de niveau, c'est vider puis reposer. L'allocateur garde ses
    // emplacements en circulation, donc les vider entierement le ferait
    // diverger du pool.
    let mut world = World::new();
    for _ in 0..5 {
        world.spawn(Player::default());
    }

    world.clear();
    assert_eq!(world.count::<Player>(), 0);

    let id = world.spawn(Player::default());
    assert!(
        world.contains(id),
        "l'acteur pose apres un vidage a disparu"
    );
    assert_eq!(world.count::<Player>(), 1);
}

#[test]
fn ids_from_before_a_clear_never_come_back() {
    let mut world = World::new();
    let avant: Vec<_> = (0..3).map(|_| world.spawn(Player::default())).collect();

    world.clear();
    let apres: Vec<_> = (0..3).map(|_| world.spawn(Player::default())).collect();

    for ancien in &avant {
        assert!(
            !apres.contains(ancien),
            "un identifiant d'avant le vidage a ete rendu a nouveau"
        );
        assert!(!world.contains(*ancien), "un ancien identifiant est vivant");
    }
}

#[test]
fn a_world_survives_many_clear_and_spawn_cycles() {
    // Ce que fait un editeur qui lance et arrete une partie en boucle.
    let mut world = World::new();

    for tour in 0..20 {
        for _ in 0..10 {
            world.spawn(Player::default());
        }
        assert_eq!(world.count::<Player>(), 10, "tour {tour}");
        world.clear();
    }

    assert_eq!(world.count::<Player>(), 0);
}
