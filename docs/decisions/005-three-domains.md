# 005 — Three domains rather than N independent editors

**Status:** Accepted
**Date:** 2026-09-07

## Context

The editor model is taken from Unreal: each asset type gets a dedicated editor
rather than a generic property grid. A Material opens a graph editor, a Widget
Blueprint opens a designer, a Sound Cue opens an audio graph.

Enumerating what a 2D engine needs produces roughly eight editors: sprite,
palette, tileset, tilemap, animation, material, widget, cue, instrument,
sequencer, scene.

Eight editors is eight applications. That is what makes Unreal enormous, and it
is not affordable for a solo project.

Unreal's other property is that those editors are islands: the Material editor,
the Niagara editor and the Animation editor share a window frame and little
else. Each has its own timeline, its own preview, its own conventions.

## Decision

Group everything into three domains — Game, Visual, Audio. Editors within a
domain share their foundations by construction, not by convention.

Animation is not a domain; it belongs to Visual and shares Visual's frame model,
palette, canvas and preview. A material graph and a sound cue graph are the same
problem — a node graph over a dataflow — and share one implementation in
`raster-graph`.

## Alternatives

**Independent editors, Unreal-style.** Rejected on cost and on quality. Eight
independent editors means eight timelines and eight undo stacks, which produces
an inconsistent tool and multiplies the work.

**One generic editor, Godot-style.** Rejected — it is the third complaint that
motivated the project. A generic property grid edits a shader like a table of
numbers.

## Consequences

**Easier:** the second editor in a domain is much cheaper than the first; a
consistent interface follows from shared foundations rather than from discipline;
one undo stack, one selection model, one palette.

**Harder:** the shared foundations must be designed before the second editor in a
domain exists. Specifically `raster-graph` before the material graph, since the
cue editor needs the same thing, and Visual's canvas and palette model before the
tilemap editor.

**Risk:** a domain's shared foundation ends up shaped around its first editor and
fits the second badly. Mitigated by building the two graph editors together in
Wave 6 rather than sequentially.
