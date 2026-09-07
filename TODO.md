# TODO

Every task, grouped by version. This is the working plan; `docs/roadmap.md` is
the reasoning behind its ordering.

**Legend** — `[ ]` todo · `[~]` in progress · `[x]` done · `[?]` needs a decision
first · `[!]` blocks other work

**The rule that governs everything below: a game runs before the editor exists.**
Starting with the editor is how these projects die. Nothing in M4+ begins until
M0–M3 ship.

---

## Milestone overview

| Version | Name | Delivers | Blocks |
| --- | --- | --- | --- |
| **M0** | Foundations | Workspace, math, reflection | Everything |
| **M1** | First pixel | A sprite moving under keyboard control | M2 |
| **M2** | A world | Tiles, collision, animation, audio, scenes | M3 |
| **M3** | **MVP — a playable game** | A small complete game, no editor | M4 |
| **M4** | UI toolkit | `raster-ui`, HUD and menus | M5 |
| **M5** | Editor shell | Docking, inspector, viewport, play-in-editor | M6 |
| **M6** | Visual editors | Sprite, palette, tileset, tilemap, animation | M7 |
| **M7** | Graphs | Material graph + cue graph | — |
| **M8** | Remaining tools | UI designer, instrument, sequencer, lighting | — |
| **M9** | **v1.0 — public** | Docs, examples, licence, released | — |
| **M10** | Scripting | Rhai, then a purpose-built language | — |

M3 is the MVP. M9 is v1.0. Everything between is the tooling that makes the
engine worth using by someone other than its author.

---

# M0 — Foundations

Nothing visible on screen. The parts everything else assumes, including the one
piece that is expensive to retrofit.

## M0.1 Repository and tooling

- [x] Create the repository
- [x] Cargo workspace manifest
- [x] `.gitignore`, `.gitattributes`
- [x] Design documentation in `docs/`
- [x] `CLAUDE.md` with conventions
- [x] `rust-toolchain.toml` pinning the toolchain
- [x] `rustfmt.toml` and `clippy.toml`
- [x] CI: build, test, clippy with `-D warnings`, fmt check
- [x] Release workflow: macOS (Intel + Apple Silicon), Linux, Windows
- [x] `dev` as the default branch, one branch per change
- [x] Branch protection on `main` and `dev`
- [ ] `[?]` Choose the licence — MIT OR Apache-2.0 is the Rust convention
- [ ] `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` in `docs/`
- [x] Issue and PR templates
- [ ] `deny.toml` (`cargo-deny`) for licence and advisory checks

## M0.2 raster-math

Boring and stable within a month. No dependencies.

- [x] `Vec2` — arithmetic, dot, cross, length, normalise, rotate, lerp
- [x] `Rect` — intersection, containment, union, expansion, corners
- [x] `IVec2` for tile and pixel coordinates
- [ ] `Transform2D` — translation, rotation, scale, composition, inverse
- [ ] `Mat3` for transform composition
- [ ] `IRect` for integer rectangles
- [ ] Angle helpers — wrapping, shortest delta, degree/radian conversion
- [ ] Easing curves — the standard set, plus a custom curve type
- [ ] Deterministic RNG (PCG or xorshift) — seedable, reproducible
- [ ] `tests/` covering every operation, including degenerate cases
- [ ] Property tests where they apply (inverse of inverse is identity, etc.)

## M0.3 Reflection `[!]`

Blocks the inspector, scene serialisation, scripting and hot reload. This is why
it is in M0 and not M5.

- [ ] `[?]` Field access: generated accessors vs raw offsets — leaning accessors
  (no `unsafe` in the foundation crate, and the perf difference is irrelevant at
  inspector rates)
- [ ] `Value` enum — the dynamic currency
- [ ] `ValueKind` — the type descriptor
- [ ] `TypeInfo` and `FieldInfo`
- [ ] `Reflect` trait — `type_info`, `get_field`, `set_field`, `fields`
- [ ] `#[derive(Reflect)]` proc macro
- [ ] `#[property(...)]` attributes — skip, min, max, readonly, rename, tooltip
- [ ] Primitives, engine types, nested structs, enums with payloads, `Vec<T>`,
  `Option<T>`
- [ ] Compile error (not silent skip) on a field whose type is not reflectable
- [ ] `[?]` Type registry: `inventory`-style linker collection vs explicit
  registration — explicit is uglier and far more predictable
- [ ] Name-to-constructor map for instantiating from a scene file
- [ ] `tests/` — round-trip every supported type, rename survival, error cases

## M0.4 Core skeleton

- [ ] `ActorId` — generational index, `Debug`, `Hash`, niche-optimised `Option`
- [ ] `Actor` trait
- [ ] `Behaviour` trait with all lifecycle hooks defaulted empty
- [ ] `World` — spawn, despawn, get, get_mut, iter
- [ ] `[?] [!]` **Storage model — decide by measurement, not argument.**
  Benchmark all three at 1k / 10k / 50k actors before committing:
  - A. `Vec<Option<Box<dyn Actor>>>` in a generational arena
  - B. Per-type typed pools with a type tag in `ActorId` *(current inclination)*
  - C. Archetype storage behind an actor-shaped API
- [ ] Generation allocator with free-list reuse
- [ ] Stale-id resolution returns `None` — with a test that proves it
- [ ] Component index: component type → `(ActorId, accessor)`
- [ ] `tests/` — spawn/despawn/reuse, stale ids, generation overflow

## M0.5 Window and GPU surface

- [ ] `winit` window creation and the event loop
- [ ] `wgpu` instance, adapter, device, queue
- [ ] Surface configuration, resize handling, present modes
- [ ] Clear to a colour — the "hello world" of a renderer
- [ ] Graceful failure when no compatible adapter exists

**M0 done when:** a window opens, an actor can be spawned, and its fields can be
listed and modified by name at runtime.

---

# M1 — First pixel

The milestone that proves the actor model on real code.

## M1.1 Renderer core

- [ ] Shader module loading and the render pipeline
- [ ] Vertex and index buffers, dynamic sizing
- [ ] Uniform buffers and bind groups
- [ ] Texture upload, sampler with nearest filtering
- [ ] Sprite batcher — group by (texture, material, layer)
- [ ] Draw call submission and a per-frame stats counter
- [ ] Texture atlas: runtime packing, then import-time packing

## M1.2 Pixel-perfect pipeline `[!]`

The constraint that defines the renderer. Getting the camera/sprite snapping pair
wrong produces either jitter or blur, and it is the most common failure in 2D
engines.

- [ ] Fixed low-resolution render target
- [ ] Integer upscale to the window
- [ ] Letterboxing when aspect ratios differ
- [ ] Nearest-neighbour everywhere by default
- [ ] Sprite positions snap to the pixel grid
- [ ] **Camera interpolates in sub-pixels** — this is the half people get wrong
- [ ] Rotation policy: off by default, explicit opt-in
- [ ] Visual test scene that makes jitter and blur obvious at a glance

## M1.3 Sprites

- [ ] `Sprite` component — texture, frame, tint, flip, pivot, layer
- [ ] Layer ordering with within-layer sorting
- [ ] `Texture` asset type, PNG loading
- [ ] Load a Rasterie-authored sprite from disk
- [ ] Placeholder texture (magenta checker) for unresolved handles

## M1.4 Camera

- [ ] Orthographic 2D camera
- [ ] Viewport and zoom (integer zoom levels only)
- [ ] Follow behaviour with smoothing and dead zone
- [ ] Bounds clamping
- [ ] Multiple cameras with per-camera layer masks

## M1.5 Input

- [ ] Keyboard and mouse state
- [ ] Gamepad via `gilrs`
- [ ] Action mapping — `Action::Jump` rather than a raw key
- [ ] Axis mapping with dead zones
- [ ] Bindings as an asset, not in code
- [ ] **Input buffering** — a jump pressed just before landing still registers
- [ ] **Coyote time** — a jump just after leaving a ledge still registers
- [ ] `tests/` for buffering and coyote windows

## M1.6 The frame loop

- [ ] Fixed timestep accumulator
- [ ] Interpolated rendering between fixed steps
- [ ] `Time` — delta, raw_delta, elapsed, scale, fixed_delta
- [ ] `raw_delta` for UI, so menus animate while the game is paused
- [ ] Spiral-of-death guard (cap the fixed steps per frame)
- [ ] The documented frame order from `docs/domains/game.md`

## M1.7 Actor lifecycle

- [ ] `on_spawn`, `tick`, `on_despawn` wired into the loop
- [ ] `fixed_tick`
- [ ] `Ctx` — the single command surface
- [ ] Deferred spawn/despawn (no mutation of the world mid-iteration)

**M1 done when:** a sprite drawn in Rasterie moves under keyboard control, in a
window, driven by an actor written in Rust — with no editor.

**Decide here:** the storage model, by benchmark.

---

# M2 — A world

Enough engine for a real, if small, game.

## M2.1 Tilemaps

- [ ] Chunked storage — fixed-size chunks of compact arrays
- [ ] Tile id, and per-tile data (collision, one-way, material)
- [ ] Multiple layers — background, main, foreground
- [ ] Chunk mesh generation
- [ ] Rebuild only the affected chunk on edit
- [ ] Culling — draw only visible chunks
- [ ] Streaming — load and evict chunks around the camera
- [ ] `Tileset` asset — tiles defined from a sprite
- [ ] Autotile: 47-variant bitmask, computed on edit and cached
- [ ] Runtime mutation API (place, break, query) — cheap, because a Terraria-like
  does it constantly
- [ ] `tests/` — chunk boundaries, autotile neighbourhoods, streaming

## M2.2 Physics

- [ ] `Body` component — static, kinematic, dynamic
- [ ] AABB overlap and resolution
- [ ] **Tile collision** — specialised against the grid, the dominant case
- [ ] Swept collision so fast projectiles do not tunnel
- [ ] Slopes
- [ ] One-way platforms
- [ ] Layers and masks
- [ ] Triggers (overlap without response)
- [ ] Queries — raycast, shape cast, overlap
- [ ] `on_collide` dispatch
- [ ] `tests/` — tunnelling, corner cases, slope transitions, one-way edges

## M2.3 Animation

- [ ] `Animation` asset — frames with per-frame duration
- [ ] `Animator` component — play, loop, speed, stop
- [ ] **Events on frames** — a footstep on frame 3, a hitbox on frame 5
- [ ] State machine — states, transitions, conditions
- [ ] Crossfade between states
- [ ] Property animation over reflected fields, using curves

## M2.4 Audio

- [ ] `raster-audio` crate wrapping `resonance-core`
- [ ] Audio thread, lock-free command queue
- [ ] **No allocation, no locks, no I/O on the audio thread** — an underrun is an
  audible click
- [ ] `Cue` asset (runtime side; the graph editor is M7)
- [ ] Voice management and a stealing policy when voices run out
- [ ] Buses — Master, Music, SFX, UI, Ambience
- [ ] Volume, ducking, snapshots
- [ ] 2D spatialisation — distance attenuation, stereo pan, low-pass by distance
- [ ] Listener attached to an actor
- [ ] WAV and OGG loading
- [ ] `[?]` Per-sample vs per-block evaluation — per-block is far more efficient
- [ ] `tests/` — headless, no device required

## M2.5 Assets

- [ ] `AssetId<T>` — typed, uuid-backed
- [ ] `.meta` sidecar files
- [ ] Project manifest, uuid → path
- [ ] `Handle<T>` — reference counted, placeholder while loading
- [ ] Async loading on a worker pool
- [ ] **Hot reload** — watch, reload, swap behind live handles
- [ ] Import pipeline — PNG → Sprite, WAV → Cue
- [ ] `asset!` macro resolving paths at compile time
- [ ] Dependency tracking (a tileset depends on its sprite)

## M2.6 Scenes

- [ ] Text format (TOML)
- [ ] Save — write only fields that differ from type defaults
- [ ] Load — instantiate via reflection
- [ ] Multiple scenes loaded at once, each tracking its own actors
- [ ] Unload a scene independently
- [ ] `[?]` Nesting and overrides — the prefab problem; deferring is fine,
  ignoring forever is not
- [ ] `tests/` — round-trip fidelity, unknown field tolerance, renamed fields

## M2.7 Attachment

- [ ] `attach` / `detach` with named anchors
- [ ] Transform resolution through the attachment chain
- [ ] Explicit lifetime policy when a parent is destroyed

## M2.8 Messages

- [ ] Typed message queue
- [ ] Delivery at a defined point in the frame, never re-entrant
- [ ] `on_message` dispatch
- [ ] `[?]` Dynamic messages for scripting later

**M2 done when:** a character runs and jumps through a tile world, with
animation and sound, loaded from a scene file.

---

# M3 — MVP: a playable game

**The most important milestone in this document.** An engine that has never
shipped a game is a hypothesis, not a tool.

No editor exists yet. The game is built by writing Rust and hand-editing scene
files, and that is deliberate — it proves the runtime stands on its own.

- [ ] `[?]` Pick the game. Small and finishable. A single-screen platformer or a
  tiny mining/building loop — **not** the full Terraria-like
- [ ] Player controller that feels good (this is where buffering and coyote time
  earn their place)
- [ ] Two or three enemy types with distinct behaviour
- [ ] Collision, damage, death, respawn
- [ ] A basic inventory
- [ ] Tile placing and breaking
- [ ] Save and load of world state (distinct from scenes — runtime state)
- [ ] Sound effects and music
- [ ] A title screen and a pause menu (immediate-mode stopgap; `raster-ui` is M4)
- [ ] Win or loss condition
- [ ] Ship a build for macOS and one other platform
- [ ] **Write down every friction encountered.** This list is the input to M4–M8

**M3 done when:** someone who is not you can download it, play it, and finish it.

---

# M4 — UI toolkit

- [ ] `Widget` trait — layout, paint, event
- [ ] Widget tree with stable identity and persistent state
- [ ] Constraint-based layout (flex-like) plus anchors for HUDs
- [ ] Painter API over the 2D renderer
- [ ] Event routing — hover, focus, capture, keyboard navigation
- [ ] Bitmap font rendering (a pixel art engine needs pixel fonts first)
- [ ] `[?]` Vector font support later, or never
- [ ] Theme tokens — colour, spacing, radius, motion — centralised in one place
- [ ] Core widgets: text, button, slider, checkbox, radio, text input, list,
  scroll area, panel, split, tabs, tree, menu, tooltip, modal
- [ ] Animation and transitions using `raw_delta`
- [ ] `.widget` asset loading (the designer is M8)
- [ ] Rebuild the M3 game's HUD and menus with it

**M4 done when:** the MVP's UI is built with `raster-ui`.

**Checkpoint `[?]`:** is it good enough for the editor? If not, decide here to
use `egui` for editor chrome — and record the decision in `docs/decisions/`.

---

# M5 — Editor shell

The frame that holds the editors. Still no dedicated editors.

- [ ] Editor binary, separate from the runtime
- [ ] Docking — split, tab, float, resize
- [ ] Layout persistence per project
- [ ] Asset browser — tree, search, filter, drag and drop
- [ ] **Global undo/redo** — one stack for every editor, not one per panel
- [ ] Command pattern with coalescing (a drag is one undo entry)
- [ ] **Reflection-driven inspector** — works on any `Reflect` type, honours
  `#[property]` attributes
- [ ] Scene viewport — pan, zoom, grid, selection, box select
- [ ] Gizmos — move, and scale where it applies
- [ ] Snapping to grid and to pixels
- [ ] Console — logs, warnings, errors, filtering
- [ ] **Play in editor** — run the game in a panel, with live asset reload
- [ ] Project creation and settings
- [ ] Editor preferences

**M5 done when:** the MVP's level can be built by placing actors in the viewport
instead of by editing text.

---

# M6 — Visual editors

Where Raster starts being distinctive. Sprite first — the tileset editor consumes
sprites, and the shared canvas and palette model gets established here.

## M6.1 Shared Visual foundations `[!]`

Designed before the second editor exists, or the third forces a rewrite of the
first two.

- [ ] Canvas — pan, zoom, pixel grid, checkerboard, rulers
- [ ] Selection — rectangle, lasso, magic wand, by colour
- [ ] Palette model shared across every Visual editor
- [ ] Frame model shared between sprite and animation
- [ ] Preview surface
- [ ] Tool framework — brush, eraser, fill, picker, shapes

## M6.2 Rasterie port to Rust `[!]`

- [ ] Port `rasterie-engine` from TypeScript to Rust
- [ ] Keep the web version working — this is a port, not a migration
- [ ] Colour: OKLCH, ramps, hue shifting
- [ ] Shape grammar
- [ ] Pixel art rules: cluster size, orphan pixels, canonical slopes
- [ ] Dithering
- [ ] Validation and audit
- [ ] Verify parity against the TypeScript test suite

## M6.3 Sprite editor

The flagship.

- [ ] Manual pixel drawing — the full tool set
- [ ] Layers with blend modes and opacity
- [ ] Parametric generation via Rasterie
- [ ] Live parameter tweaking
- [ ] Pixel art rule checking as guidance, not enforcement
- [ ] Onion skinning
- [ ] Symmetry modes
- [ ] Import a PNG, and be honest that it carries no parameters
- [ ] Export PNG and spritesheet

## M6.4 Palette editor

- [ ] Ramp authoring with OKLCH hue shifting
- [ ] Palette swapping across every sprite built on it
- [ ] Import `.gpl`, `.pal`, `.hex`
- [ ] Contrast and readability checking

## M6.5 Tileset editor

- [ ] Define tiles from a sprite
- [ ] Per-tile collision shape
- [ ] Autotile rule authoring, with a live 47-variant preview
- [ ] Tile properties and metadata
- [ ] Animated tiles

## M6.6 Tilemap painter

- [ ] Paint, erase, fill, rectangle, line
- [ ] Layer switching
- [ ] Stamps and brushes from multi-tile selections
- [ ] Autotile applied live while painting
- [ ] Large-map performance (this is where naive implementations die)

## M6.7 Animation timeline

- [ ] Frame timeline with drag-to-reorder
- [ ] Per-frame duration
- [ ] Onion skinning, shared with the sprite editor
- [ ] Event markers on frames
- [ ] State machine graph editor
- [ ] Property curve editor
- [ ] Live preview

**M6 done when:** a sprite can be created, turned into a tileset, painted into a
world and animated — without leaving Raster.

---

# M7 — Graphs

Two editors, one foundation. Built together deliberately: they are the same
problem, and building one alone produces a foundation shaped around a single
case.

## M7.1 raster-graph `[!]`

- [ ] Node and pin model, typed connections
- [ ] Type checking on connect
- [ ] Cycle detection
- [ ] Topological evaluation order
- [ ] Serialisation
- [ ] Graph editor UI — pan, zoom, box select, wire routing
- [ ] Node search and creation menu
- [ ] Comment boxes and reroute nodes
- [ ] Undo integration

## M7.2 Material graph

- [ ] Node set: texture sample, colour, maths, UV, time, palette lookup
- [ ] Shader codegen from the graph (WGSL)
- [ ] Live preview on a sprite
- [ ] Built-in effects: outline, hit flash, dissolve, water distortion, palette
  cycling, scrolling
- [ ] Material parameters exposed to actors

## M7.3 Cue graph

- [ ] Node set: sample source, Resonance instrument, noise
- [ ] Selection: random, sequential, weighted, by parameter
- [ ] Modulation: pitch, volume, filter, randomised ranges
- [ ] Envelopes and effects, exposed from Resonance
- [ ] Routing and spatialisation nodes
- [ ] Live audition in the editor

**M7 done when:** a hit flash is authored as a material and a footstep as a cue,
both in graphs, both live in the running game.

---

# M8 — Remaining tools

## M8.1 UI designer

The most expensive editor in this document.

- [ ] Canvas with widget hierarchy
- [ ] Drag to place, resize, anchor
- [ ] Property editing per widget
- [ ] Layout preview at multiple resolutions
- [ ] Theme editing
- [ ] `.widget` asset output
- [ ] Data binding to actor properties

## M8.2 Instrument editor

- [ ] Oscillator, envelope, filter, effect chains from Resonance
- [ ] Live keyboard preview
- [ ] Waveform and spectrum display
- [ ] Preset management
- [ ] `[?]` `rnc` as a text authoring path alongside the visual editor

## M8.3 Sequencer

- [ ] Track and pattern editing
- [ ] Piano roll
- [ ] Tempo, time signature, swing
- [ ] Layered tracks with intensity fading
- [ ] Transitions on musical boundaries
- [ ] Export to WAV

## M8.4 2D lighting

- [ ] Point, spot and directional 2D lights
- [ ] Normal map support on sprites
- [ ] Shadow casting from tiles
- [ ] Ambient and per-layer lighting
- [ ] Day/night cycle support
- [ ] Optional — many pixel art games want flat unlit rendering

## M8.5 Particles

- [ ] Particle system with its own tight storage — **not actors**
- [ ] Emitter shapes, burst and continuous
- [ ] Curves over lifetime for size, colour, velocity
- [ ] Sprite and animated particles
- [ ] Particle editor with live preview

## M8.6 Build pipeline

- [ ] Asset packing for release builds
- [ ] Build configuration per target
- [ ] Cross-compilation: Windows, macOS, Linux
- [ ] Asset optimisation and compression
- [ ] `[?]` Web export via WASM

---

# M9 — v1.0: public release

- [ ] User documentation — not these design notes
- [ ] Getting-started tutorial
- [ ] API reference (`cargo doc`, curated)
- [ ] 5–10 example projects, from trivial to complete
- [ ] A starter template repository
- [ ] `cargo generate` template
- [ ] Website with documentation hosting
- [ ] Choose and apply the licence across every file
- [ ] Publish crates to crates.io
- [ ] Semantic versioning policy and a stability statement
- [ ] Migration guide policy for breaking changes
- [ ] Repository goes public
- [ ] Announcement

---

# M10 — Scripting

Only once the API has been proven by a real game.

- [ ] Audit the `Ctx` surface for scriptability
- [ ] Bind Rhai — not to ship, but to find what the API gets wrong
- [ ] Actor properties via reflection from script
- [ ] Script lifecycle hooks
- [ ] Hot reload of scripts
- [ ] **Fix everything the binding reveals**
- [ ] `[?]` Then, and only then: a purpose-built language
  - [ ] Syntax design informed by what Rhai made awkward
  - [ ] Lexer, parser, AST *(the `rnc` experience applies directly)*
  - [ ] Type system
  - [ ] Bytecode VM
  - [ ] Diagnostics people can act on
  - [ ] Debugger
  - [ ] Editor integration and autocompletion
- [ ] `[?]` C# — an adoption decision, not a technical one. May never happen

---

# Continuous

Not milestones. Ongoing throughout.

## Testing

Structure, per `docs/decisions/010-testing-layout.md`:

- `crates/<crate>/tests/` — integration tests against the public API. The bulk.
- `#[cfg(test)]` in-file — only for what is private and unreachable from outside.

- [ ] Architecture rules enforced in CI with `cargo-deny` and a dependency
  check, rather than as a test crate:
  - [ ] The runtime never depends on the editor
  - [ ] `raster-core` depends only on `raster-math`
  - [ ] Domains do not reach into each other
  - [ ] Only `raster-render` and the editor touch GPU or windowing
- [ ] Golden-image rendering tests
- [ ] Benchmarks: actor iteration, sprite batching, tile updates, audio mixing
- [ ] Fuzzing for asset and scene parsers
- [ ] Coverage reporting in CI

## Performance

- [ ] Frame-time budget and a visible overlay
- [ ] Profiling hooks
- [ ] Benchmark suite run in CI with regression alerts
- [ ] Memory tracking
- [ ] Target: 60fps with 1,000 active actors and a streamed tile world

## Documentation

- [ ] Every public item documented as it is written, not afterwards
- [ ] Design docs updated when the code contradicts them — a stale doc is a bug
- [ ] A new ADR for every decision with lasting consequences

## The game

- [ ] Keep building a real game against the engine, continuously
- [ ] Every friction becomes an issue
- [ ] The Terraria-like remains the long-term target that justifies the design

---

# Decisions still open

Collected from throughout. Each blocks work downstream.

| # | Decision | Needed by | Current leaning |
| --- | --- | --- | --- |
| 1 | Actor storage model | M0.4 | Typed pools — decide by benchmark |
| 2 | Reflection field access | M0.3 | Generated accessors, no `unsafe` |
| 3 | Type registry mechanism | M0.3 | Explicit registration |
| 4 | Licence | M0.1 | MIT OR Apache-2.0 |
| 5 | Which game is the MVP | M3 | Small platformer, not the Terraria-like |
| 6 | `raster-ui` for the editor | M4 | Try it; `egui` is the fallback |
| 7 | Scene nesting and overrides | M2.6 | Defer, do not ignore |
| 8 | Audio per-sample vs per-block | M2.4 | Per-block |
| 9 | Tilemap: asset or save data | M2.1 | Save data for generated worlds |
| 10 | Web export | M8.6 | Plausible, not a priority |

---

# How this fails

Named so they are recognisable while they are happening:

1. **Building the editor first.** The one reliable way to never ship.
2. **Skipping M3.** If the MVP is not finished, none of the tooling is justified.
3. **Generalising early.** A plugin system before two plugins exist.
4. **Perfecting M0.** Reflection can be revised. It cannot be skipped, and it
   does not need to be beautiful before M1 starts.
5. **Scope creep into 3D.** See `docs/non-goals.md`.
6. **Starting the Terraria-like as the MVP.** It is the long-term target, not
   the first game.
