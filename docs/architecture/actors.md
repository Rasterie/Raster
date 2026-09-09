# The actor model

This is the core abstraction of Raster. Everything else is built to serve it.

## The principle

**An actor is a Rust type. Its components are its fields. Its behaviour is its
methods. One file describes one entity.**

```rust
#[derive(Actor)]
pub struct Player {
    sprite: Sprite,
    body: Body,
    health: Health,

    speed: f32,
    jumps_left: u8,
}

impl Player {
    fn spawn() -> Self {
        Self {
            sprite: Sprite::new(asset!("actors/player.sprite")),
            body: Body::dynamic(Rect::new(0.0, 0.0, 12.0, 24.0)),
            health: Health::new(100),
            speed: 90.0,
            jumps_left: 2,
        }
    }
}

impl Behaviour for Player {
    fn tick(&mut self, ctx: &mut Ctx, dt: f32) {
        let dir = ctx.input.axis(Axis::Horizontal);
        self.body.velocity.x = dir * self.speed;

        if ctx.input.pressed(Action::Jump) && self.jumps_left > 0 {
            self.body.velocity.y = -220.0;
            self.jumps_left -= 1;
            ctx.audio.play(asset!("sfx/jump.cue"));
        }
    }

    fn on_land(&mut self, _ctx: &mut Ctx) {
        self.jumps_left = 2;
    }
}
```

Read that file and you know what a player is: what it is made of, what it does,
what it responds to. No tree to navigate, no scene file to open, no guessing
which of six nodes holds the script.

## What this is not

**Not an ECS.** Bevy, and most modern Rust engines, store components in
archetype tables and run systems over queries. That is excellent for simulating
a hundred thousand entities and poor for expressing "a player is this specific
thing". Raster chooses legibility over that particular kind of throughput, and
accepts the cost.

The cost is real and worth naming: iterating 50,000 actors of mixed types is
slower here than in an ECS, because their data is not laid out contiguously by
component. See "Where this model breaks" below for how a game avoids paying it
where it matters.

**Not a node tree.** A `Sprite` is a component, not an entity. It has no
independent existence, no transform of its own, and cannot be placed in a scene
by itself. It belongs to its actor.

**Not deep inheritance.** There is no `Actor` base class to extend through five
generations. Shared behaviour comes from composition — a component — or from a
trait, not from an inheritance chain.

## Components

A component is a plain struct that provides one capability. It holds data, and
may hold behaviour that operates only on itself.

```rust
pub struct Sprite {
    pub texture: AssetId<Texture>,
    pub frame: u32,
    pub tint: Color,
    pub flip_x: bool,
    pub layer: Layer,
}
```

Components are just fields. There is no registration, no dynamic map of
`TypeId -> Box<dyn Any>`, no `get_component::<Sprite>()` returning an `Option`
you must unwrap. If a `Player` has a `Sprite`, `self.sprite` is right there and
the compiler knows it.

The engine discovers components through [reflection](reflection.md), which is
also what powers the inspector and, later, scripting.

### How subsystems find components

The renderer must draw every `Sprite` without knowing about `Player`. Reflection
provides this: at registration time, each actor type reports which of its fields
are components of which type, and the world maintains an index from component
type to `(ActorId, field offset)`.

This is the central performance question of the design, and the point where the
storage model matters most. It is not settled — see below.

## Identity

Actors are referenced by `ActorId`, never by Rust reference.

```rust
pub struct ActorId {
    index: u32,
    generation: u32,
}
```

A generational index: the generation invalidates ids when an actor is destroyed,
so a stale id resolves to `None` rather than to whatever was allocated in its
place. This is a well-understood pattern and the right default.

Three reasons this matters, and only one is about the borrow checker:

1. **Actors reference each other freely.** An enemy targeting a player holds an
   `ActorId`, not a `&Player`. No lifetime problems, no `Rc<RefCell<>>`.
2. **Scripting requires it.** A script cannot hold a Rust reference across a
   call boundary. If the engine's own API is id-based from day one, adding
   scripting is an adapter rather than a rewrite. See
   [scripting.md](scripting.md).
3. **Serialisation requires it.** A scene file stores ids, not pointers.

## The world

The world owns every actor and is the only path to mutate one.

```rust
impl World {
    pub fn spawn<A: Actor>(&mut self, actor: A) -> ActorId;
    pub fn despawn(&mut self, id: ActorId);
    pub fn get<A: Actor>(&self, id: ActorId) -> Option<&A>;
    pub fn get_mut<A: Actor>(&mut self, id: ActorId) -> Option<&mut A>;
    pub fn iter<A: Actor>(&self) -> impl Iterator<Item = (ActorId, &A)>;
}
```

Note the absence of a `root` or a `scene tree`. The world is a flat collection.

## Parenting is optional

Actors are not in a hierarchy by default. Most entities in a 2D game — an enemy,
a projectile, a chest — have no parent and need none.

Attachment exists when it is genuinely needed: a weapon held in a hand, a health
bar following a monster, a camera tracking a player.

```rust
ctx.attach(sword_id, to: player_id, at: Anchor::Named("hand_r"));
ctx.detach(sword_id);
```

Attachment affects transform resolution and lifetime (detaching or destroying a
parent is an explicit policy, not an implicit cascade). It is a relation between
actors, not the structure that gives them existence.

This is the direct answer to "everything must be in a scene": in Raster, an actor
exists in the world; a tree is something you opt into.

## Lifecycle

The hooks an actor may implement. All are optional, all have empty defaults.

| Hook | When |
| --- | --- |
| `on_spawn` | Added to the world, after fields are initialised |
| `tick` | Once per frame, with delta time |
| `fixed_tick` | Fixed timestep, for physics-coupled logic |
| `on_despawn` | Removed from the world, before storage is freed |
| `on_collide` | Physics reports a collision |
| `on_message` | Another actor sends it a typed message |

Deliberately absent: `on_render`. Actors do not draw. The renderer reads
`Sprite` components. An actor that wants custom visuals configures a material or
a custom sprite, it does not get a draw callback. This keeps rendering batched
and keeps the renderer independent of gameplay.

## Messages

Actors communicate through typed messages rather than direct method calls,
because a direct call would require a mutable reference to another actor while
the caller is itself borrowed.

```rust
ctx.send(target, Damage { amount: 10, source: self_id });
```

Messages are queued and delivered at a defined point in the frame, never
re-entrantly. Delivery order and timing are specified rather than incidental —
that specification is a task for the first implementation, not a detail to leave
to whatever the code happens to do.

## Where this model breaks, and what we do about it

Being honest about this now is cheaper than discovering it in year two.

**Many small identical entities.** Ten thousand particles, or bullets, as actors
would be slow — each is a heap-ish allocation with a virtual `tick`. The answer
is not to make the actor model faster; it is to not use actors for those.
Particles are a Visual-domain system with their own tight storage. Same for
projectiles, if a game needs thousands.

**Tiles.** A large tile world holds millions of cells. Tiles are emphatically
not actors: they are a chunked grid in `raster-2d`, stored as compact arrays.
Only tiles with behaviour — a chest, a machine — get an associated actor.

**Wide component iteration.** "Update every `Body`" touches memory scattered
across actor types. If profiling shows this dominating, the fix is to store hot
components in dedicated arrays that the actor field aliases into. That is a
significant design change, which is why the storage model is still open.

## The open question: storage

Three candidates, none yet chosen. This should be decided by measurement during
Wave 1, on a realistic actor count, not by argument now.

**A. Boxed trait objects in a generational arena.** `Vec<Option<Box<dyn Actor>>>`.
Simplest, closest to Unreal, straightforward reflection. Costs a pointer chase
per actor and scatters component data.

**B. Per-type pools.** One typed `Vec<Player>`, one `Vec<Enemy>`, and so on, with
`ActorId` carrying a type tag. Actors of a type are contiguous, iteration by
type is fast, no boxing. Costs complexity in heterogeneous iteration and in
`get::<A>` dispatch.

**C. Archetype storage with an actor-shaped API.** ECS underneath, actor model
on the surface. Best iteration performance, hardest to reconcile with "the actor
owns its fields", and risks leaking ECS semantics through the abstraction — which
would defeat the purpose of the whole design.

Current inclination: **B**, because it preserves the model's clarity while making
the common case (iterate all actors of one type) fast, and because it makes the
component index cheap. This is an inclination, not a decision.

## Consequences to hold on to

Whatever the storage, three rules must survive, because everything downstream
depends on them:

1. Actors are addressed by `ActorId`, never by Rust reference across an API.
2. Every engine capability is reachable through `Ctx` — a single, enumerable
   command surface — rather than through free functions and globals.
3. Actor fields are reflectable: listable, readable and writable dynamically.

These three are what make the inspector possible, and what will make scripting an
adapter instead of a rewrite. They cost almost nothing to hold from the start and
are extremely expensive to retrofit.
