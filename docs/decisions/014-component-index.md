# 014 — The component index: a typed path beside reflection

**Status:** Accepted
**Date:** 2026-09-08

## Context

The renderer must draw every `Sprite` without knowing that `Player` exists.
`docs/architecture/actors.md` said reflection would provide this, and that was
half right.

Reflection *finds* the fields: a `FieldInfo` whose kind is
`ValueKind::Struct("Sprite")` says which actor types hold one. But reading a
field back yields an owned `Value` — for a struct, an allocated `BTreeMap`. Per
actor, per frame. That is correct for an inspector and unusable for a render
loop.

## Decision

A second, typed path alongside reflection.

- **`Component`** — a marker trait on the component type.
- **`HasComponent<C>`** — implemented by `#[derive(Reflect)]` for each field
  marked `#[property(component)]`, yielding borrows rather than values.
- **`World::register_component::<T, C>()`** — declares that actors of type `T`
  hold components of type `C`.
- **`World::each_component::<C>(f)`** — applies `f` to every `C` in the world,
  whatever actor holds it.

## Why the shape is what it is

**An attribute rather than inference.** A derive macro sees tokens, not types:
it cannot know whether `Sprite` implements `Component`. Marking the field is
also more legible than a rule the reader has to remember.

**An iterator rather than one reference.** An actor may hold two of the same
component — a chest and its lid. One `HasComponent<Sprite>` implementation
gathering every `Sprite` field avoids the conflicting-impl problem that a
per-field trait would create.

**A callback rather than a returned iterator.** The components live in
different pools of different concrete types. Any iterator spanning them would
have to box or collect, allocating once per frame. The callback does neither —
measured at zero allocations, below.

**Explicit registration**, for the same reason as type registration in decision
011. Forgetting it silently skips that actor type, which a test covers so the
behaviour is deliberate rather than surprising.

## Measurements

Release build, LTO, mixed actor types, counting allocations with a wrapping
global allocator.

| Actors | Sprites | Per iteration | Allocations |
| ---: | ---: | ---: | ---: |
| 1 000 | 1 500 | 7.5 µs | 0 |
| 10 000 | 15 000 | 58.3 µs | 0 |
| 50 000 | 75 000 | 152.6 µs | 0 |

152 µs for 75 000 sprites is 0.9% of a 16 700 µs frame — and, more importantly,
nothing is allocated on the path the renderer walks every frame.

## Consequences

**Two ways to reach a field**, which is a real cost in API surface: reflection
for tools, `HasComponent` for subsystems. They are not interchangeable, and the
docs must keep saying which is which.

**Registration is a chore** the game must remember. The alternative — linker
collection — was rejected in decision 011 for reasons that apply here too.

**Not yet done:** mutable iteration (`each_component_mut`). Animation will need
it; the borrow rules are less obvious there, and inventing them before something
needs them would be guessing.
