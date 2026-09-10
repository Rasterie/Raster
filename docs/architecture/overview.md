# Architecture overview

## The three layers

Raster separates three things that engines often blur together.

```
┌─────────────────────────────────────────────────────────┐
│  EDITOR            the tools, only present when editing  │
│  raster-editor, raster-ui, per-domain editors            │
├─────────────────────────────────────────────────────────┤
│  RUNTIME           what ships inside the game            │
│  raster-core, raster-render, raster-audio, raster-2d     │
├─────────────────────────────────────────────────────────┤
│  FOUNDATIONS       independent, useful without Raster    │
│  rasterie-engine, resonance-core, raster-math            │
└─────────────────────────────────────────────────────────┘
```

The rule that keeps this honest: **the runtime never depends on the editor**. A
shipped game links `raster-core` and its domain crates and nothing else. If a
runtime crate needs something from the editor, the design is wrong.

## Workspace layout

A single Cargo workspace. Each crate is a real boundary — its own `Cargo.toml`,
its own tests, its own public API — but they version together and refactor in
one pass.

```
raster/
├── Cargo.toml                  workspace manifest
├── crates/
│   ├── raster-math/            vectors, rects, transforms, curves
│   ├── raster-core/            actors, world, scenes, resources, reflection
│   ├── raster-render/          the 2D renderer (wgpu)
│   ├── raster-2d/              sprites, tilemaps, animation, cameras
│   ├── raster-audio/           audio runtime, wraps resonance-core
│   ├── raster-physics/         AABB, tile collision, spatial queries
│   ├── raster-input/           keyboard, mouse, gamepad
│   ├── raster-assets/          asset ids, loading, hot reload, import
│   ├── raster-graph/           the shared node-graph model
│   ├── raster-ui/              the retained-mode UI toolkit (game + editor)
│   ├── raster-script/          the scripting boundary (see scripting.md)
│   ├── raster-editor/          the editor shell: docking, undo, asset browser
│   ├── editors/
│   │   ├── raster-ed-visual/   Visual: what every Visual editor shares
│   │   ├── raster-ed-sprite/   Visual: the sprite editor (Rasterie)
│   │   ├── raster-ed-tilemap/  Visual: tileset and tilemap painting
│   │   ├── raster-ed-anim/     Visual: animation timeline
│   │   ├── raster-ed-widget/   Visual: UI designer
│   │   ├── raster-ed-material/ Visual: 2D material graph
│   │   ├── raster-ed-sound/    Audio: cue graph and synthesis (Resonance)
│   │   └── raster-ed-scene/    Game: the viewport and actor inspector
│   └── raster/                 the umbrella crate a game depends on
├── examples/
└── docs/
```

### Why a monorepo

Rasterie's web app and engine live in separate repositories, because they are
separate products with separate release cadences and a stable API between them.

Raster is the opposite case. The crates change together, a rename crosses ten of
them at once, and nobody consumes `raster-render` without `raster-core`. A Cargo
workspace is built for exactly this: one `cargo test`, one `cargo build`, atomic
refactors, no version matrix.

The two foundation crates — `rasterie-engine` and `resonance-core` — stay
outside, because they *are* independently useful.

## The three domains in crates

The domain model from [vision.md](../vision.md) maps onto crates like this:

**Game** — `raster-core`, `raster-physics`, `raster-input`, `raster-ed-scene`

Owns actors, the world, scene serialisation, collision and input. This is the
domain that defines what an entity *is*, so the other two depend on it and never
the reverse.

**Visual** — `raster-render`, `raster-2d`, `raster-ui`, and five editors

The largest domain by far. Everything that produces pixels: the renderer, the
sprite and tilemap data, animation, the UI toolkit, materials. Its editors share
a palette, a frame model, a canvas and a preview surface.

**Audio** — `raster-audio`, `raster-ed-sound`

Wraps `resonance-core` with the engine-facing concepts: cues, buses, spatial
attenuation, and the link between an actor and a sound.

## Dependency rules

These are the constraints that keep the layering real. They should be enforced
by a test, the way `rasterie-engine`'s autonomy is.

1. Nothing in `crates/` except `raster-editor` and `crates/editors/*` may
   reference the editor.
2. `raster-core` depends on `raster-math` and nothing else in the workspace. It
   is the root of the runtime.
3. Domains do not reach into each other's internals. Audio does not know what a
   sprite is; Visual does not know what a sound bus is. They meet in
   `raster-core`, through actors and assets.
4. No crate depends on a windowing or GPU library except `raster-render` and the
   editor. Headless tests must be possible for everything else.
5. `rasterie-engine` and `resonance-core` stay free of Raster types. If a change
   to Raster requires a change to one of them, that is a signal the boundary is
   wrong.

## Crate responsibilities

### raster-math
Vectors, rectangles, 2D transforms, easing curves, RNG. No dependencies. The
kind of crate that should be boring and stable within a month.

### raster-core
The heart. Defines actors, the world that holds them, actor identity, the
component model, the reflection system, scene serialisation, and the command API
that scripting will eventually target. Everything else is built on this.

Deliberately excludes: rendering, audio, physics, input. `raster-core` describes
*what exists*, not what it looks like or sounds like.

### raster-render
The 2D renderer over `wgpu`. Sprite batching, texture atlases, render layers,
2D lighting, materials as shaders, and the pixel-perfect pipeline (integer
scaling, snapping rules, filtering policy).

### raster-2d
The 2D content types that sit between core and the renderer: sprite components,
tilemaps and chunking, animation state, cameras, parallax. This is where the
structural needs of a large, mutable tile world live.

### raster-audio
Cues, buses, mixing, 2D spatial attenuation, and the actor↔sound link. Delegates
all synthesis and DSP to `resonance-core`.

### raster-physics
AABB collision, tile-grid collision, raycasts and spatial queries. Not a full
rigid-body simulation — see [non-goals.md](../non-goals.md).

### raster-assets
Asset identity, the manifest, loading, hot reload, and importing external files.
Owns the question "what is an asset and how do we refer to one" for every domain.

### raster-graph
The shared node-graph model: nodes, pins, typed connections, evaluation order,
cycle detection, serialisation. Used by material graphs and sound cues — the same
problem twice, solved once. Not used for gameplay logic.

### raster-ui
A retained-mode UI toolkit with a class-based, object-oriented widget model,
used both for in-game UI and for the editor itself. One toolkit means one visual
identity across everything. See [editor.md](editor.md).

### raster-script
The scripting boundary. Initially empty of any language: it defines the command
surface and the reflection contract that a scripting language will bind to.
See [scripting.md](scripting.md).

### raster-editor
The editor shell: docking, the asset browser, undo/redo, project management,
play-in-editor. Hosts the per-domain editors but knows nothing about their
internals beyond a common trait.

### raster
The umbrella crate. Re-exports what a game needs, so a `Cargo.toml` reads
`raster = "0.1"` and not a list of nine crates.

## What is not decided yet

Recorded here so they are not mistaken for settled:

- The actor storage model (arena, slotmap, or archetype) — see
  [actors.md](actors.md)
- Whether `raster-ui` is genuinely good enough for the editor, or whether the
  editor needs its own toolkit
- How scenes handle nesting and prefab-style overrides
- The tilemap chunk format and its streaming strategy
