# The editor

## The principle

**One asset type, one dedicated editor — grouped into three domains that share
their foundations.**

Unreal gets the first half right: a Material opens a graph editor, a Widget
Blueprint opens a designer, a Sound Cue opens an audio graph. Each asset gets a
workspace built for what it actually is.

What Unreal does not do is make those editors share anything. The Material
editor, the Niagara editor and the Animation editor are islands, each with its
own timeline, its own preview, its own conventions. That is affordable for Epic.
It is not affordable here, and it produces an inconsistent tool.

Raster groups editors into the three domains, and editors within a domain share
their infrastructure by construction:

- **Visual** editors share the canvas, the palette, the frame model, the pixel
  grid and the preview surface. The sprite editor, the tilemap painter and the
  animation timeline are three views over related concepts, not three programs.
- **Audio** editors share the transport, the waveform display, the synthesis
  chain and the live preview.
- **Game** editors share the viewport, the selection model and the inspector.

## The shell

`raster-editor` provides what every editor needs and none of them should
implement:

- **Docking** — panels arranged and rearranged, layouts saved per project
- **Asset browser** — the project tree, the entry point to every editor
- **Undo/redo** — one global stack, shared across every editor
- **Inspector** — driven by [reflection](reflection.md), works on any type
- **Play in editor** — run the game in a panel, with hot reload live
- **Console** — logs, warnings, errors

Undo being global is a deliberate constraint. Per-editor undo stacks are easier
to build and produce an unpredictable tool: undoing after switching panels does
something surprising. One stack, one history, one mental model.

## The editor is a Raster application

The editor is built with `raster-ui`, the same retained-mode toolkit games use.

This is a deliberate bet, and it cuts both ways.

**In favour:** it makes `raster-ui` genuinely good, because it is used daily on a
demanding application. A UI toolkit whose author does not use it for anything
serious stays mediocre. It also gives the editor and games one visual identity,
which is your requirement of a shared design language.

**Against:** it couples the editor's progress to the toolkit's, and a
retained-mode object-oriented toolkit is a substantial project in itself. If
`raster-ui` is late, the editor is late.

The mitigation is ordering: `raster-ui` is Wave 2, before any real editor work,
and the first editors are simple enough to be built on a toolkit that is still
young. If the bet fails — if the editor needs things games never will — the
fallback is `egui` for editor chrome while `raster-ui` stays the game toolkit.
That fallback should be a decision recorded in `decisions/`, not a quiet drift.

## The UI toolkit

Object-oriented and retained-mode, per the design requirement of a common base
and a consistent visual identity.

```rust
pub struct Button {
    label: String,
    style: StyleRef,
    on_click: Option<Callback>,
}

impl Widget for Button {
    fn layout(&mut self, constraints: Constraints) -> Size;
    fn paint(&self, painter: &mut Painter);
    fn event(&mut self, event: &Event) -> Response;
}
```

Widgets are objects with identity and state, arranged in a tree, laid out by
constraint propagation, and painted through the same 2D renderer the game uses.

Immediate-mode (`egui`-style) was considered and rejected for the main toolkit:
it is excellent for tools and awkward for game UI, where you want persistent
widgets with animation state and a designer that edits a real hierarchy. Since
the toolkit must serve both, retained mode is the one that can.

Styling is centralised — a theme of tokens, not per-widget constants — so the
design identity is one file rather than a thousand call sites.

## Per-domain editors

### Visual

**Sprite editor** — the Rasterie pixel art tool, embedded. Parametric
generation, palettes, OKLCH ramps with hue shifting, and the pixel art rules
(cluster size, orphan pixels, canonical slopes) enforced as guidance. This is
the flagship: the reason someone would choose Raster over Godot.

**Tileset and tilemap** — define tiles from a sprite, set collision and autotile
rules, then paint. Autotiling with 47-variant bitmasks, because a Terraria-like
is unusable without it.

**Animation timeline** — frames, timing, events on frames, curve-based property
animation. Shares the frame model with the sprite editor rather than redefining
it.

**UI designer** — a canvas plus a widget hierarchy, producing `.widget` assets.
The direct analogue of UMG, and the most expensive of these to build.

**Material graph** — a node graph producing a 2D shader. Built on `raster-graph`,
shared with the sound cue editor.

### Audio

**Cue graph** — assemble sounds: sources, mixing, randomisation, envelopes,
effects. Node-graph based, same `raster-graph` foundation. The direct analogue
of Sound Cue.

**Instrument editor** — Resonance's synthesis surface: oscillators, envelopes,
filters, effects, with live preview.

**Sequencer** — Resonance's music sequencing, tracks and patterns.

### Game

**Scene viewport** — place actors, move them, see the world. Zoom, pan, grid
snapping, selection, gizmos.

**Actor inspector** — reflection-driven property editing on the selected actor.

## What the editor is not

- **Not a level editor that owns entities.** It edits scene files, which
  reference actor types defined in code. It never generates gameplay code.
- **Not required to run a game.** A game links the runtime only. The editor is a
  separate binary.
- **Not a replacement for a text editor.** Raster does not include a code editor.
  You write Rust in whatever you already use.

## Order of construction

Editors arrive in waves — see [roadmap.md](../roadmap.md) — but the shared
foundations must be designed before the second editor of a domain exists, or the
third forces a rewrite of the first two.

Specifically: `raster-graph` before the material graph, because the sound cue
editor needs the same thing. And the Visual domain's canvas and palette model
before the tilemap editor, because the sprite editor will have established them
informally by then.
