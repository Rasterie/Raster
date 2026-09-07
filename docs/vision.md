# Vision

## What Raster is

Raster is a game engine for 2D games, written in Rust, with an editor whose
tools are organised by domain rather than scattered across a generic inspector.

It is 2D-first in the strict sense: there is no 3D renderer with a 2D mode
layered on top. The scene is a plane, the camera is orthographic, the units are
pixels, and every subsystem is designed around that from the start.

## Why it exists

Three complaints, each specific, each with a design consequence.

### 1. An entity should have a boundary

Engines built on a scene tree dissolve an entity into a collection of nodes. A
sprite is not *part of* a player — it is a peer in the tree, with its own
transform, able to exist alone. A moderately complex entity becomes six nodes,
and its logic lives on whichever one holds a script. Reading the code does not
tell you what the entity is.

Unreal gets this right: an `Actor` is a class, its components are declared
inside it, and one file describes one entity. That is the model Raster wants,
applied to 2D.

**Consequence:** Raster uses an actor model. An actor is a Rust type. Its
components are fields. Its behaviour is its methods. See
[architecture/actors.md](architecture/actors.md).

### 2. Code should be the source of truth

When the scene file defines the entity, code becomes an attachment to it. You
cannot understand an entity by opening a file — you must open the editor. For
someone who works in code, that is backwards, and it makes diffs and version
control worse than they need to be.

**Consequence:** in Raster, code is the source of truth. A scene is a list of
instances and their overridden values, serialised as readable, diffable text. An
actor type exists whether or not any scene references it. See
[architecture/assets.md](architecture/assets.md).

### 3. Each asset deserves its own editor

A generic property grid edits a shader, a tileset and a sound the same way — as
a table of values. Unreal gets this right: a Material opens a graph editor, a
Widget Blueprint opens a designer with a canvas and a hierarchy, a Sound Cue
opens an audio graph. Each asset type gets a workspace built for it.

**Consequence:** in Raster, each asset type has a dedicated editor — but grouped
into three domains that share their foundations, rather than eight independent
applications that each reinvent a timeline and an undo stack. See
[architecture/editor.md](architecture/editor.md).

## The domain model

This is the structural idea that distinguishes Raster from Unreal's approach.

Unreal's specialised editors are each an island. The Material editor, the
Niagara editor and the Animation editor share a window frame and little else.
That is affordable at Epic's scale. It is not affordable here, and it is not
actually better.

Raster groups everything into three domains:

- **Game** — actors, the world, scenes, physics, input
- **Visual** — rendering, sprites, tilemaps, animation, UI, materials, palettes
- **Audio** — synthesis, cues, mixing, music

Animation is not a domain. It belongs to Visual, and it shares Visual's
foundations: the concept of a frame, the palette, the viewport, the preview.
A material graph and a sound cue graph are the same problem — a node graph over
a dataflow — and share one implementation.

A domain owns its data model, its runtime and its editors. Editors within a
domain share their infrastructure by construction, not by convention.

## Two tools that already exist

Raster starts with an unusual advantage: two of its domain tools already exist
as working, independent projects.

- **[Rasterie](https://github.com/Rasterie/engine)** — a parametric pixel art
  generator that encodes the rules of pixel art (cluster size, no orphan pixels,
  canonical slopes, OKLCH ramps with hue shifting). It becomes the foundation of
  the Visual domain's sprite editor.
- **Resonance** — a synthesis and sequencing library in Rust, with its own DSL,
  oscillators, envelopes, filters and effects. It becomes the foundation of the
  Audio domain.

Both remain independent crates with their own repositories and their own
releases. Raster consumes them. This is deliberate: they are useful outside a
game engine, and keeping them separate keeps their APIs honest.

The point is not that Raster generates content for you. It is that the tools
live **inside** the engine. No round trip to Aseprite and back, no exporting WAVs
from a DAW. You open the asset, you edit it, it is already in the game.

## Who it is for

For now: for me, to build 2D games with — a Terraria-like is the reference
target, because a large persistent tile world with inventory, lighting and
simulation stresses nearly every subsystem.

Later: open source, for anyone who wants an engine built for 2D from the ground
up, with an actor model where an entity is a type in a file.

The repository is private until there is something worth showing. But it is
written as open source from the first commit — documented decisions, honest
non-goals, a public licence, no shortcuts that assume a single user. Retrofitting
that later never works.

## What success looks like

In order, and none of these skip ahead:

1. A sprite drawn in Raster, moving on screen, driven by an actor written in Rust
2. A tile world you can walk around in
3. A game that is actually finished and playable
4. Someone other than me shipping something with it

Step 3 matters more than any engine feature. An engine that has never shipped a
game is a hypothesis, not a tool.
