# 013 — Actor storage: typed pools

**Status:** Accepted
**Date:** 2026-09-08

## Context

`docs/decisions/001-actor-model.md` settled the *surface* model — an actor is a
Rust type, its components are its fields — but left the storage open, with an
instruction to decide by measurement rather than argument.

Three candidates:

- **A. Boxed arena** — `Vec<Option<Box<dyn Actor>>>`. Simplest, closest to
  Unreal, one pointer chase per actor.
- **B. Typed pools** — one `Vec<Player>`, one `Vec<Enemy>`, with a type tag in
  `ActorId`. Contiguous per type, no boxing.
- **C. Archetypes** — component columns, ECS-style, behind an actor-shaped API.

## Measurements

Release build, `lto = true`, `codegen-units = 1`, Apple Silicon. Mixed
population of three actor types. Times are per operation, in microseconds.

| Operation | Actors | A. boxed | B. pools | C. archetypes |
| --- | ---: | ---: | ---: | ---: |
| tick all | 1 000 | 2.8 | **1.3** | 2.1 |
| tick all | 10 000 | 13.7 | **6.1** | 8.5 |
| tick all | 50 000 | 40.0 | **20.4** | 31.0 |
| collect sprites | 10 000 | 12.8 | 6.5 | **6.1** |
| collect sprites | 50 000 | 53.8 | 29.7 | **28.2** |
| random access ×1000 | 10 000 | 2.5 | 2.6 | **1.5** |
| random access ×1000 | 50 000 | 3.0 | 2.3 | **1.7** |
| iterate one type | 5 000 | 17.2 | **1.5** | n/a |
| iterate one type | 25 000 | 82.2 | **8.7** | n/a |
| spawn+despawn 10 000 | — | 196.0 | **39.4** | 216.8 |

Archetypes have no single-type iteration row: the model has no notion of an
actor type to iterate, which is precisely the legibility problem decision 001
rejected.

## Decision

**Typed pools.**

## Why the numbers say this

**The frame budget makes the scale legible.** At 60fps a frame is 16 700 µs.
Ticking 50 000 actors costs 20 µs with pools and 40 µs boxed — both negligible.
The measurements matter not because any single one is dangerous, but because the
gaps are consistent and compound across a real frame.

**Single-type iteration is the decisive row.** 82 µs against 8.7 µs at 25 000
actors — 9.4×. This is what a system does every frame ("update every enemy"),
and it is the operation the engine will perform most often. The boxed arena
pays a pointer chase plus a failed downcast for every actor of the wrong type.

**Spawn and despawn is the second gap.** 39 µs against 196 µs for 10 000
actors, 5×. A bullet-heavy game creates and destroys constantly, and the boxed
arena allocates on every spawn.

**Pools lose only on random access, and only slightly.** 2.3 µs against 1.7 µs
per thousand lookups — the type tag costs a branch. At 0.6 µs per thousand this
is not worth a design decision.

**Archetypes win two rows and lose the model.** Sprite collection is marginally
faster (28.2 against 29.7 µs — 5%), and random access is genuinely better. But
ticking is 50% slower than pools, spawn/despawn is the worst of the three, and
the model cannot express "iterate every Enemy" at all. Paying that to win 5% on
one row would trade the whole design for a rounding error.

## Consequences

**`ActorId` carries a type tag**, so a lookup can find the right pool. Three
fields — index, generation, type tag — and the id stays 8 bytes.

**Heterogeneous iteration branches on the tag.** "Every actor, whatever its
type" costs a match. The measurements above already include that cost.

**Adding an actor type adds a pool.** The world cannot be a fixed struct of
named pools as in this benchmark; it needs a map from type to a type-erased
pool, with downcasting at the boundary. That indirection is not in these
numbers and must be kept thin — it is the main risk of this choice.

**Revisit if** heterogeneous iteration becomes the dominant cost in a real
profile, or if the type-erased pool map turns out to cost more than the boxed
arena it replaced. Neither is likely; both are measurable.

## Follow-up: the type-erased indirection, measured

The consequences above flagged the `TypeId -> pool` map as the risk of this
choice, and as the one thing the benchmark did not cover. It has now been
measured on the real implementation.

| Operation | Through `World` | Benchmark equivalent |
| --- | ---: | ---: |
| tick, 50 000 actors | 28.6 µs | 20.4 µs |
| iterate one type, 50 000 | 22.9 µs | 8.7 µs |
| spawn 10 000 | 176 µs | — |
| spawn + despawn 10 000 | 225 µs | 39.4 µs |
| spawn 10 000 into a bare `Vec` | 18 µs | — |

**The indirection is real and it is on the spawn path.** Spawning through the
world costs about 10× a bare `Vec` push, because every call hashes a `TypeId`
and downcasts.

**One fix was worth making.** `despawn` originally hashed twice — tag to
`TypeId`, then `TypeId` to pool. Since tags are handed out sequentially from
zero, a `Vec` indexed by tag replaces the first map entirely. That took the
spawn+despawn cycle from 373 µs to 225 µs, and despawn alone from 170 µs to
73 µs.

**The rest is not worth fixing yet.** 176 µs for 10 000 spawns is 0.018 µs per
actor. A game spawning 100 actors in a frame pays 1.8 µs against a 16 700 µs
budget — 0.01%. The relative slowdown is dramatic; the absolute cost is not
measurable in a frame.

Caching the last-used pool would remove most of the remaining hash, and is the
obvious optimisation if a profile ever justifies it. Doing it now would add a
stale-cache failure mode to buy microseconds nobody can perceive.

## What was still not measured

Cache behaviour under a realistic access pattern — the benchmarks walk actors in
allocation order, which is friendlier than a real game where actors are created
and destroyed over time. Fragmentation would hurt all three models, and pools
least, since each type stays dense.
