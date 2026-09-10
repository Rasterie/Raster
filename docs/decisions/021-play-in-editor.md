# 021 — Play in editor: the world ticks, the editor draws

**Status:** Accepted
**Date:** 2026-09-10

## Context

M5 asks for play-in-editor: run the game in a panel, with live asset reload.
`docs/architecture/editor.md` says the same and no more.

Three facts constrain the answer:

- `raster_render::run` owns the event loop and the window. Two of them cannot
  coexist in one process.
- `Gpu` holds a surface tied to that window. There is exactly one.
- `App` is a plain trait — `init`, `update`, `fixed_update`, `render` — and
  `RenderTarget` already draws off-screen.

## Options

**A. A separate process.** The editor spawns the game binary. Complete
isolation: a crash in the game cannot take the editor down, and the game runs
exactly as a player would run it. In exchange, nothing is shared — no live
inspection of a running actor, no editing while it runs, and the panel would
show a separate window rather than embedded content.

**B. The game runs inside the editor's loop.** The editor owns the frame, and
calls the game's `update` and `render` into a `RenderTarget` it then draws in a
panel. Everything is shared: the world is right there, the inspector keeps
working on live actors, hot reload applies to both.

**C. A game-agnostic simulation.** No game binary at all: the editor ticks the
`World` it is already editing, using the runtime's own systems.

## Decision

**C, growing into B.**

The editor already holds a `World` full of actors. Playing is not a separate
program — it is the same world, advanced by the runtime instead of frozen for
editing. Pressing play copies the world aside, runs it, and pressing stop puts
the untouched copy back. That is what an editor's play mode *is*: a scratch
run over a saved state.

This needs no process, no second window, no second GPU. It is the option that
matches what the editor already has.

Option B — a real game crate's `App` driven by the editor — is the same
machinery with a `Box<dyn App>` in place of the built-in tick. Reaching it
means giving `App` a way to be driven without owning the loop, which is worth
doing when a game other than the editor's own actors needs to run.

Option A stays the right answer for *testing a release build*, which is not
what play-in-editor is for. A game a player would run is launched, not embedded.

## Consequences

- Pressing play snapshots the world; stopping restores it. Nothing a play
  session does can be saved by accident.
- The undo history is untouched while playing, and cleared of nothing when
  stopping — the edit before play is still the last undoable thing.
- Hot reload already works, because assets are shared with the editor.
- The panel needs no new rendering path: the same batcher, a `RenderTarget`,
  drawn where the viewport was.

## Revisiting

The signal to move to B is a game whose behaviour lives in its own crate rather
than in reflected actors — which is every real game. The scratch-run design
above does not change then; only what advances the world does.
