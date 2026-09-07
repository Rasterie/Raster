# Domain: Game

Actors, the world, scenes, physics, input. This domain defines what an entity
*is*; the other two describe how it looks and sounds. Visual and Audio depend on
Game, never the reverse.

**Crates:** `raster-core`, `raster-physics`, `raster-input`, `raster-ed-scene`

## Actors and the world

The full model is in [architecture/actors.md](../architecture/actors.md). In
summary: an actor is a Rust type, its components are its fields, the world is a
flat collection, actors are addressed by `ActorId`, and parenting is optional.

## The frame

A defined order, because "whatever the code happens to do" becomes impossible to
debug once there are ten systems.

```
1. input        poll devices, update action state
2. fixed_tick   zero or more times, fixed timestep
     a. actor fixed_tick
     b. physics integration and collision
     c. collision callbacks
3. tick         once per frame, variable delta
4. messages     deliver everything queued during 1–3
5. animation    advance animation state
6. audio        update cues, buses, spatial parameters
7. render       cull, batch, draw
```

Fixed timestep with an accumulator, interpolated rendering. A platformer with
variable-timestep physics has jump heights that depend on frame rate, which is
the kind of bug that is discovered far too late.

Messages deliver at one point, never re-entrantly, so an actor cannot be mutated
while it is mid-`tick`.

## Time

```rust
pub struct Time {
    pub delta: f32,        // scaled
    pub raw_delta: f32,    // unscaled — for UI, which should ignore pauses
    pub elapsed: f64,
    pub scale: f32,        // 0.0 pauses; slow motion is a game mechanic
    pub fixed_delta: f32,
}
```

`raw_delta` exists because UI animation must keep running while the game is
paused. Forgetting this is a common and annoying bug.

## Physics

Deliberately not a rigid-body simulator. See
[non-goals.md](../non-goals.md#a-full-physics-engine).

What is provided is what 2D action games actually use:

- **AABB bodies** — static, kinematic and dynamic
- **Tile collision** — against the tilemap grid, the dominant case for a
  Terraria-like, and specialised rather than generalised
- **Swept collision** — continuous, so fast projectiles do not tunnel through
  walls
- **One-way platforms** — solid from above, passable from below
- **Slopes** — because a tile world without them feels wrong
- **Queries** — raycast, shape cast, overlap, with layer masks
- **Triggers** — overlap without response, for pickups and zones

Rotated colliders are not supported. Rotation is visual; collision stays
axis-aligned. This is what most 2D games do, and it makes tile collision an order
of magnitude simpler and faster.

```rust
pub struct Body {
    pub kind: BodyKind,
    pub shape: Rect,
    pub velocity: Vec2,
    pub gravity_scale: f32,
    pub layer: Layer,
    pub mask: LayerMask,
}
```

## Input

Actions rather than raw keys, so rebinding and multiple devices work without
gameplay code changing:

```rust
if ctx.input.pressed(Action::Jump) { … }
let dir = ctx.input.axis(Axis::Horizontal);
```

Bindings live in an asset, not in code. Keyboard, mouse and gamepad map into the
same action space.

Two features that gameplay code otherwise reimplements badly, provided here
instead: **buffering** (a jump pressed slightly before landing still registers)
and **coyote time** (a jump shortly after leaving a ledge still registers). Both
are the difference between controls that feel good and controls that feel
broken, and both belong in the engine.

## Scenes

A scene is a list of actor instances with their overridden properties, in a
readable text file. See [architecture/assets.md](../architecture/assets.md).

Loading a scene instantiates its actors into the world. Multiple scenes can be
loaded at once — a level plus a UI layer plus a persistent manager — and each
tracks which actors it owns so it can be unloaded independently.

## Save games

Called out because a Terraria-like makes it structural rather than an
afterthought: the world is generated, then mutated indefinitely, and must
survive a restart.

The world state is not the scene. Scenes are authored; saves are runtime state.
The two use the same reflection machinery and different files.

Open: how much of an actor's state is saved by default. Saving everything is
simple and fragile across versions; saving marked fields is more work and
survives updates.

## Open questions

- Actor storage — the three candidates in
  [actors.md](../architecture/actors.md#the-open-question-storage), to be
  decided by measurement
- Whether messages are typed at compile time or dynamic (scripting will need
  dynamic; a hybrid is likely)
- Determinism: worth guaranteeing for replays and networking, expensive to
  retrofit, and not currently a requirement
- Multiplayer: explicitly out of scope for now, but the actor model and message
  system should not make it impossible later
