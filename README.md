# Raster

A 2D game engine in Rust, where an entity is a type in a file and every asset
gets an editor built for it.

> **Status: design phase.** There is no code yet — only the documents in
> [`docs/`](docs/) that decide what gets built and in what order. The repository
> is private until there is something worth showing.

## What it is

Raster is 2D-first in the strict sense: no 3D renderer with a 2D mode layered on
top. The scene is a plane, the camera is orthographic, the units are pixels, and
every subsystem is designed around that.

Three ideas define it.

**An actor is a Rust type.** Its components are its fields, its behaviour is its
methods. One file describes one entity — no tree of nodes to navigate, no scene
file that has to be opened to find out what a player is.

```rust
#[derive(Actor)]
pub struct Player {
    sprite: Sprite,
    body: Body,
    speed: f32,
}

impl Behaviour for Player {
    fn tick(&mut self, ctx: &mut Ctx, dt: f32) {
        self.body.velocity.x = ctx.input.axis(Axis::Horizontal) * self.speed;
    }
}
```

**Code is the source of truth.** A scene is a list of instances and the values
they override, in readable diffable text. Delete every scene and you lose level
layouts, not the definition of what an entity is.

**Every asset type gets its own editor**, grouped into three domains — Game,
Visual, Audio — that share their foundations. A sprite opens a pixel art editor,
a cue opens an audio graph, a material opens a shader graph. Not one generic
property grid for all of them.

## The tools live inside the engine

Raster starts with two working tools that become domain editors:

- **[Rasterie](https://github.com/Rasterie/engine)** — a parametric pixel art
  generator that encodes the rules of pixel art: cluster sizes, orphan pixels,
  canonical slopes, OKLCH ramps with hue shifting. Becomes the sprite editor.
- **Resonance** — a synthesis and sequencing library with oscillators,
  envelopes, filters, effects and its own DSL. Becomes the audio domain.

The point is not that the engine generates content for you. It is that there is
no round trip: no exporting PNGs from Aseprite, no exporting WAVs from a DAW.
You edit the asset where you use it, and the running game updates live.

## Documentation

Everything is in [`docs/`](docs/). Start with:

- [Vision](docs/vision.md) — why this exists and who it is for
- [Architecture overview](docs/architecture/overview.md) — domains and crates
- [The actor model](docs/architecture/actors.md) — the core abstraction
- [Roadmap](docs/roadmap.md) — the order of work
- [Non-goals](docs/non-goals.md) — what it will not do, and why
- [Decisions](docs/decisions/) — the reasoning behind the choices


## Non-goals

No 3D, ever. No full physics engine. No visual scripting for gameplay. No
networking. No code editor. See [non-goals.md](docs/non-goals.md) for the
reasoning on each.

## Licence

MIT. See [LICENSE](LICENSE).
