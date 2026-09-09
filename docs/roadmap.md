# Roadmap

## The ordering principle

**A game runs before the editor exists.**

The way solo engine projects die is by starting with the editor, because that is
the visible part and the part that feels like an engine. Two years later there
is a docking system, an inspector and no game.

Raster inverts it: the runtime first, then the tools, then polish. Every wave
ends with something playable, and the actor model gets validated by use before
any editor is built on top of it.

The waves are ordered, not scheduled. Estimates on a solo part-time project are
fiction; the sequence is what matters.

## Wave 0 — Foundations

Nothing visible. The parts everything else assumes.

- Workspace, CI, licence, contribution rules
- `raster-math` — vectors, rects, transforms, curves
- `raster-core` skeleton — `ActorId`, the world, spawn/despawn/get
- **Reflection** — the derive macro, `TypeInfo`, `Value`, dynamic get/set
- Windowing and an empty `wgpu` surface

Reflection is here rather than later because the inspector, scene loading,
scripting and hot reload all reduce to it. Retrofitting it is the expensive
mistake.

**Done when:** a window opens, an actor can be spawned, and its fields can be
listed and modified by name at runtime.

## Wave 1 — A sprite that moves

The first milestone that proves the model.

- Sprite rendering — batching, atlases, layers
- Pixel-perfect pipeline — integer scaling, snapping, nearest filtering
- Camera — orthographic, follow, sub-pixel interpolation
- Input — actions, bindings, buffering, coyote time
- Frame loop — fixed timestep, interpolated rendering
- Actor lifecycle — `tick`, `on_spawn`, `on_despawn`
- Load a Rasterie sprite from disk

**Done when:** a sprite drawn in Rasterie moves under keyboard control, in a
window, driven by an actor written in Rust — with no editor.

**Decide here, by measurement:** the actor storage model. Benchmark the three
candidates on a realistic actor count rather than arguing about them.

## Wave 2 — A world to walk in

Enough engine for a real, if small, game.

- Tilemaps — chunked storage, layers, rendering
- Tile collision — AABB against the grid, slopes, one-way platforms
- Physics — bodies, swept collision, queries, triggers
- Animation — frames, timing, events, state machine
- Audio — `raster-audio` over Resonance, cues, buses, 2D spatialisation
- Scenes — text format, load and save, reflection-driven
- Asset system — ids, manifest, loading, hot reload

**Done when:** a character runs and jumps through a tile world, with animation
and sound, loaded from a scene file.

## Wave 3 — The UI toolkit

The prerequisite for the editor, and useful to games on its own.

- `raster-ui` — widget tree, layout, painting, events
- Theming — centralised tokens
- Bitmap font rendering
- The core widget set — text, button, slider, list, panel, scroll

**Done when:** a game HUD and a pause menu are built with it, in the Wave 2 game.

**Checkpoint:** if the toolkit is not good enough for the editor, decide here
whether to use `egui` for editor chrome. Record the decision either way.

## Wave 4 — The editor shell

Still no dedicated editors — the frame that holds them.

- Docking and layout persistence
- Asset browser
- Global undo/redo
- Reflection-driven inspector
- Scene viewport — place, select, move, gizmos
- Console
- Play in editor

**Done when:** the Wave 2 game's level can be built by placing actors in the
viewport instead of by editing text.

## Wave 5 — The Visual editors

Where Raster starts being distinctive.

- **Sprite editor** — Rasterie embedded: parametric generation, palettes, OKLCH
  ramps, pixel art rules as guidance
- **Palette editor** — ramps, hue shifting, palette swapping
- **Tileset and tilemap** — tile definition, collision, 47-variant autotile,
  painting
- **Animation timeline** — frames, events, curves, state machine

Sprite before tilemap: the tileset editor consumes sprites, and the shared canvas
and palette model gets established by the sprite editor first.

**Done when:** a sprite can be created, turned into a tileset, painted into a
world and animated — without leaving Raster.

## Wave 6 — Graphs

Two editors, one foundation.

- `raster-graph` — nodes, typed pins, evaluation, cycle detection, serialisation
- **Material graph** — 2D shaders: outlines, flashes, dissolves, distortion
- **Cue graph** — sound assembly, the Sound Cue analogue

Built together deliberately: they are the same problem, and building one without
the other produces a foundation shaped around a single case.

**Done when:** a hit flash is authored as a material and a footstep as a cue,
both in graphs, both live in the running game.

## Wave 7 — The rest of the tools

- **UI designer** — the `.widget` authoring canvas, the UMG analogue
- **Instrument editor** — Resonance's synthesis surface
- **Sequencer** — music tracks and patterns
- 2D lighting — normal maps, coloured lights, tile shadows
- Particles — their own system, not actors

## Wave 8 — Scripting

Only once the API has been proven by a real game.

- Audit the `Ctx` surface for scriptability
- Bind Rhai — not to ship, but to find what the API gets wrong
- Fix what that reveals
- Then, and only then, consider a purpose-built language

See [architecture/scripting.md](architecture/scripting.md) for why this is last.

## Wave 9 — Open source

- Documentation that is not these design notes
- Examples and a starter template
- A published, versioned release
- The repository goes public

## Alongside everything: the game

A real game is built continuously against the engine, not at the end. It is the
only honest test of whether any of this works.

Reference target: a Terraria-like — chosen as a yardstick, not as the engine's
purpose. A large persistent tile world with inventory, lighting, simulation and
save files stresses nearly every subsystem at once, which is what makes it
useful for sizing. The engine stays general: if a subsystem only makes sense for
that one game, it does not belong in the engine.

## What would make this fail

Named here so they are recognisable when they start happening:

1. **Building the editor first.** The one reliable way to never ship.
2. **Generalising early.** A plugin system before two plugins exist; an
   abstraction before the second case appears.
3. **Never finishing a game.** An engine with no game is a hypothesis.
4. **Scope creep into 3D.** See [non-goals.md](non-goals.md).
5. **Perfecting Wave 0.** Reflection can be revised. It cannot be skipped, and it
   also does not need to be beautiful before Wave 1 starts.
