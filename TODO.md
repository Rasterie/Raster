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
- [x] Licence: MIT
- [x] `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` in `docs/`
- [x] Issue and PR templates
- [x] `deny.toml` (`cargo-deny`) for licence and advisory checks

## M0.2 raster-math

Boring and stable within a month. No dependencies.

- [x] `Vec2` — arithmetic, dot, cross, length, normalise, rotate, lerp
- [x] `Rect` — intersection, containment, union, expansion, corners
- [x] `IVec2` for tile and pixel coordinates
- [x] `Transform2D` — translation, rotation, scale, composition, inverse
- [x] `Mat3` for transform composition
- [x] `IRect` for integer rectangles
- [x] Angle helpers — wrapping, shortest delta, degree/radian conversion
- [x] Easing curves — the standard set
- [x] Deterministic RNG (PCG32) — seedable, reproducible, unbiased
- [x] `tests/` covering every operation, including degenerate cases — 126 tests
- [ ] A custom curve type, if the built-in easings prove insufficient

## M0.3 Reflection `[!]`

Blocks the inspector, scene serialisation, scripting and hot reload. This is why
it is in M0 and not M5.

- [x] Field access: generated accessors — no `unsafe`, see decision 011
- [x] `Value` enum — the dynamic currency
- [x] `ValueKind` — the type descriptor
- [x] `TypeInfo` and `FieldInfo`
- [x] `Reflect` trait — `type_info`, `get_field`, `set_field`, `apply`
- [x] `#[derive(Reflect)]` proc macro
- [x] `#[property(...)]` attributes — skip, min, max, readonly, rename, tooltip
- [x] Primitives, engine types, `Vec<T>`, `Option<T>`
- [x] Type registry: explicit registration — see decision 011
- [x] Name-to-constructor map for instantiating from a scene file
- [x] `tests/` — round-trip every supported type, rename survival, error cases
- [x] Nested reflected structs (a component inside an actor)
- [ ] Enums with payloads
- [x] Compile error (not silent skip) on a field whose type is not reflectable

## M0.4 Core skeleton

- [x] `ActorId` — generational index, `Debug`, `Hash`, niche-optimised `Option`
- [x] `Actor` trait
- [x] `Behaviour` trait with all lifecycle hooks defaulted empty
- [x] `World` — spawn, despawn, get, get_mut, iter
- [x] Storage model — typed pools, decided by measurement (decision 013)
- [x] Generation allocator with free-list reuse
- [x] Stale-id resolution returns `None` — with a test that proves it
- [x] `tests/` — spawn/despawn/reuse, stale ids, clear does not reset generations
- [x] Component index: `Component`, `HasComponent<C>`, `World::each_component`
  — a typed path beside reflection, measured at zero allocations per frame.
  See decision 014.
- [ ] `each_component_mut` — animation will need it; the borrow rules are less
  obvious and inventing them before a caller exists would be guessing.
- [ ] `on_collide` and `on_message` hooks — they need physics and a message
  queue, so they arrive with M2

## M0.5 Window and GPU surface

- [x] `winit` window creation and the event loop
- [x] `wgpu` instance, adapter, device, queue
- [x] Surface configuration, resize handling, present modes
- [x] Clear to a colour — the "hello world" of a renderer
- [x] Graceful failure when no compatible adapter exists
- [x] `AutoVsync` not capping on macOS is now harmless: the fixed timestep
  paces the simulation, so a high frame rate only means more frames drawn.

**M0 done when:** a window opens, an actor can be spawned, and its fields can be
listed and modified by name at runtime.

---

# M1 — First pixel

The milestone that proves the actor model on real code.

## M1.1 Renderer core

- [x] Shader module loading and the render pipeline
- [x] Instance buffer with doubling growth
- [x] Uniform buffers and bind groups
- [x] Texture upload, sampler with nearest filtering
- [x] Sprite batcher — group by (layer, texture)
- [x] Draw call submission and a per-frame stats counter
- [ ] Texture atlas: runtime packing, then import-time packing

## M1.2 Pixel-perfect pipeline `[!]`

The constraint that defines the renderer. Getting the camera/sprite snapping pair
wrong produces either jitter or blur, and it is the most common failure in 2D
engines.

- [x] Integer upscale computed from the window (`Camera::window_scale`)
- [x] Letterboxing when aspect ratios differ (`Camera::viewport`)
- [x] Nearest-neighbour everywhere by default
- [x] Sprite positions snap to the pixel grid
- [x] **Camera interpolates in sub-pixels** — this is the half people get wrong
- [x] Visual test scene that makes jitter and blur obvious at a glance
- [x] Fixed low-resolution render target — sprites draw into an offscreen
  image at the game's resolution, scaled up by a whole number with letterboxing
- [ ] Rotation policy: off by default, explicit opt-in

## M1.3 Sprites

- [x] `SpriteDraw` — position, size, source rect, tint, flip, layer
- [x] Layer ordering, then grouping by texture within a layer
- [x] `Texture` from RGBA pixels
- [x] Placeholder texture (magenta checker) for a missing texture
- [x] PNG loading — RGBA, RGB, greyscale, 8 and 16 bit
- [x] Load a sprite from disk, with a placeholder when the file is missing
- [ ] Load a Rasterie-authored sprite, parameters included
- [ ] `Sprite` as an actor component, wired to the component index

## M1.4 Camera

- [x] Orthographic 2D camera
- [x] Viewport and zoom (integer zoom levels only)
- [x] Follow behaviour with smoothing, frame-rate independent
- [x] Screen-to-world conversion, letterboxing accounted for
- [ ] Dead zone in the follow behaviour
- [ ] Bounds clamping
- [ ] Multiple cameras with per-camera layer masks

## M1.5 Input

- [x] Keyboard and mouse state
- [x] Action mapping — `Action::JUMP` rather than a raw key
- [x] Axis mapping, with diagonal normalisation
- [x] **Input buffering** — a jump pressed just before landing still registers
- [x] **Coyote time** — `Grace`, a jump just after leaving a ledge still counts
- [x] Focus loss releases every held key
- [x] `tests/` for buffering and coyote windows
- [ ] Gamepad via `gilrs` — the buttons are defined, nothing feeds them yet
- [ ] Analogue sticks with dead zones
- [ ] Bindings as an asset, not in code

## M1.6 The frame loop

- [x] Fixed timestep accumulator
- [x] `Time` — delta, raw_delta, elapsed, scale, fixed_delta, alpha
- [x] `raw_delta` for UI, so menus animate while the game is paused
- [x] Spiral-of-death guard (cap the fixed steps per frame)
- [x] The documented frame order: fixed steps, then update, then render
- [ ] Interpolated rendering — `Time::alpha` is computed but nothing reads it
  yet; sprites will need a previous position to interpolate from

## M1.7 Actor lifecycle

- [x] `fixed_update` and `update` on the `App` trait, driven by the frame loop
- [ ] `on_spawn`, `tick`, `on_despawn` driven from the loop rather than called
  by the game — needs `Ctx`, so it lands with the command surface
- [ ] `Ctx` — the single command surface
- [ ] Deferred spawn/despawn (no mutation of the world mid-iteration)

**M1 done when:** a sprite drawn in Rasterie moves under keyboard control, in a
window, driven by an actor written in Rust — with no editor.

**Decide here:** the storage model, by benchmark.

---

# M2 — A world

Enough engine for a real, if small, game.

## M2.1 Tilemaps

- [x] Chunked storage — 32x32 chunks of compact arrays, 2 KB each
- [x] Tile id in two bytes, with per-kind data in the tileset
- [x] Rebuild only the affected chunk on edit, neighbours included at borders
- [x] Culling — `chunks_in` yields only what a view touches
- [x] Autotile: the 47-variant bitmask, verified to be exactly 47
- [x] Runtime mutation API (place, break, query, fill, prune)
- [x] `tests/` — chunk boundaries, negative coordinates, autotile neighbourhoods
- [x] Tile drawing — reuses the sprite batcher rather than a cached mesh: a
  320x180 view holds ~286 tiles, 13 KB of instance data, so a cache would
  optimise nothing and add invalidation to get wrong
- [ ] Multiple layers — background, main, foreground
- [ ] Streaming — load and evict chunks around the camera
- [ ] `Tileset` as an asset rather than built in code

## M2.2 Physics

- [x] `Body` — an AABB with a velocity
- [x] AABB overlap and resolution, axis by axis
- [x] **Tile collision** — specialised against the grid
- [x] Sub-stepping so fast bodies do not tunnel
- [x] One-way platforms, with drop-through
- [x] `Contacts` — grounded, on-wall, per-side
- [x] `tests/` — tunnelling, one-way edges, narrow corridors, no sinking
- [ ] Slopes
- [ ] Layers and masks
- [ ] Triggers (overlap without response)
- [ ] Queries — raycast, shape cast
- [ ] `on_collide` dispatch — needs `Ctx`

## M2.3 Animation

- [x] `Animation` — frames with per-frame duration, loop / once / ping-pong
- [x] `Animator` — play, speed, restart, catches up on a long step
- [x] **Events on frames** — a footstep on frame 3, a hitbox on frame 5
- [x] State machine — states, transitions, conditions, deferred transitions
- [ ] Crossfade between states
- [ ] Property animation over reflected fields, using curves
- [ ] `Animation` as an asset rather than built in code

## M2.4 Audio

- [x] `raster-audio` crate — mix, voices, buses (Resonance stays the synthesis side)
- [x] Audio thread, lock-free ring buffer (decision 017)
- [x] **No allocation, no locks, no I/O on the audio thread** — an underrun is an
  audible click
- [ ] `Cue` asset (runtime side; the graph editor is M7)
- [x] Voice management and a stealing policy when voices run out
- [x] Buses — Master, Music, SFX, UI, Ambience
- [x] Volume per bus and master
- [ ] Ducking, snapshots
- [x] 2D spatialisation — distance attenuation, stereo pan
- [ ] Low-pass by distance
- [ ] Listener attached to an actor
- [x] WAV loading
- [ ] OGG loading
- [ ] `[?]` Per-sample vs per-block evaluation — per-block is far more efficient
- [x] `tests/` — headless, no device required

## M2.5 Assets

- [x] `AssetId` — the path is the identity (decision 015)
- [ ] `.meta` sidecar files — not needed until an asset carries import settings
- [x] `Project` — the root assets are named relative to, found by its marker
- [x] `AssetStore<T>` / `Handle<T>` — loaded once, handles stay valid
- [x] Placeholder for a missing texture, reported once
- [ ] Async loading on a worker pool
- [x] **Hot reload** — watch, reload, swap behind live handles (decision 016)
- [x] PNG → texture, through `Textures`
- [ ] Import pipeline — WAV → Cue
- [ ] `asset!` macro resolving paths at compile time
- [ ] Dependency tracking (a tileset depends on its sprite)

## M2.6 Scenes

- [x] Text format (TOML)
- [x] Save — write only fields that differ from type defaults
- [x] Load — instantiate via reflection
- [ ] Multiple scenes loaded at once, each tracking its own actors
- [ ] Unload a scene independently
- [ ] `[?]` Nesting and overrides — the prefab problem; deferring is fine,
  ignoring forever is not
- [x] `tests/` — round-trip fidelity, unknown field tolerance, renamed fields

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

- [x] Pick the game — a single-screen platformer (decision 018)
- [x] Player controller — run, jump, wall and platform collision
- [x] Three enemy types: walker, spike, flyer
- [x] Collision, damage, death, respawn
- [ ] A basic inventory — dropped from M3 with the mining loop (decision 018)
- [ ] Tile placing and breaking — same
- [x] Save and load of progress (distinct from scenes — runtime state)
- [x] Sound effects
- [ ] Music
- [x] A title screen and a pause menu (sprites, no text until M4)
- [x] Win and loss conditions
- [x] Ship a build for macOS, Linux and Windows — release v0.1.0
- [x] **Write down every friction encountered** — `docs/friction.md`

**M3 done when:** someone who is not you can download it, play it, and finish it.

---

# M4 — UI toolkit

- [x] Widgets — layout, paint, event through `Ui::interact`
- [x] Stable identity and persistent state (`Id`, `Ui::remember`)
- [x] Constraint-based layout (flex-like) plus anchors for HUDs
- [x] Painter API over the 2D renderer
- [x] Event routing — hover, focus, capture, keyboard navigation
- [x] Bitmap font rendering — one 5x7 font built in (decision 019)
- [ ] Loadable fonts as an asset type
- [ ] `[?]` Vector font support later, or never
- [x] Theme tokens — colour and spacing, centralised in one place
- [x] Widgets: text, button, checkbox, slider, panel, progress, radio, text
  input, list, tabs, modal, scroll area, splitter, drag value, dropdown, tooltip
- [x] Scissor clipping in `raster-render`, which the scroll area needs
- [x] Mouse wheel and drag-and-drop routing
- [ ] Remaining: tree widget
- [ ] Animation and transitions using `raw_delta`
- [ ] `.widget` asset loading (the designer is M8)
- [x] Rebuild the M3 game's HUD and menus with it

**M4 done when:** the MVP's UI is built with `raster-ui`.

**Checkpoint:** answered — the editor uses `raster-ui` (decision 020). Scissor
clipping lands in `raster-render` first: the scroll area, the dropdown and the
tooltip all need it.

---

# M5 — Editor shell

The frame that holds the editors. Still no dedicated editors.

- [x] Editor binary, separate from the runtime
- [x] Docking — split and tab; float comes with a second window
- [ ] Layout persistence per project
- [x] Asset browser — tree, search, filter
- [ ] Drag and drop from the browser into the viewport
- [x] **Global undo/redo** — one stack for every editor, not one per panel
- [x] Command pattern with coalescing (a drag is one undo entry)
- [x] **Reflection-driven inspector** — works on any `Reflect` type, honours
  `#[property]` attributes
- [x] Scene viewport — zoom, grid, selection, box select
- [x] Gizmos — move handles, constrained per axis
- [x] Snapping to grid and to pixels
- [x] Console — project state
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
- [ ] The Terraria-like remains the long-term target the engine is *sized*
  against — a yardstick for whether it holds up, never a reason to specialise
  the engine towards one game

---

# Decisions still open

Collected from throughout. Each blocks work downstream.

| # | Decision | Needed by | Current leaning |
| --- | --- | --- | --- |
| ~~1~~ | ~~Actor storage model~~ | — | **Typed pools — decided by benchmark** |
| ~~2~~ | ~~Reflection field access~~ | — | **Generated accessors — decided** |
| ~~3~~ | ~~Type registry mechanism~~ | — | **Explicit registration — decided** |
| ~~4~~ | ~~Licence~~ | — | **MIT — decided** |
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
