# 018 — The MVP game: a single-screen platformer

**Status:** Accepted
**Date:** 2026-09-09

## Context

M3 asks for a playable game, and marks the choice of *which* game as open. The
milestone is the one the roadmap calls the most important, and `TODO.md` names
skipping it as the second way this project fails.

The long-term target is a Terraria-like. `docs/roadmap.md` is explicit that this
is a yardstick, not the first game, and "starting the Terraria-like as the MVP"
is listed as failure mode 6.

What already runs: an actor world with typed pools and reflection, tilemaps with
autotiling, AABB collision with sub-stepping, sprite batching, a fixed timestep,
input with buffering and coyote time, animation state machines, scenes in TOML,
assets with hot reload, and audio with buses and spatialisation.

What does not exist: a UI toolkit (M4), an editor (M6+), and text rendering.

## Options

**A. A tiny mining/building loop.** Closest to the long-term target. Needs
world generation, an inventory, tile placing and breaking, and world state that
persists — most of which is new systems rather than proof that the old ones
work.

**B. A single-screen platformer with a goal.** Rooms of hand-authored tiles, a
few enemies, damage and respawn, a key and a door. Almost every system already
exists; what is missing is the game on top of them.

## Decision

**A single-screen platformer**, a handful of rooms, finishable in a few minutes.

The point of M3 is not to build the target game. It is to find out what hurts
when someone actually builds *a* game — and that only works if the game gets
finished. Option A spends its budget on systems that do not exist yet, which is
how a milestone meant to expose friction becomes a milestone that generates it.

The platformer exercises the runtime end to end: actors, tiles, collision,
animation, input feel, scenes, assets, audio, and save state. It leaves the
Terraria-like intact as the target, reached later with tools that M3 will have
justified.

`TODO.md`'s M3 list keeps a basic inventory and tile placing. Those belong to
option A and are dropped here — building them into a platformer with no use for
them would be scope, not proof. They return with the mining loop, once the tools
exist.

## Consequences

- Rooms are authored as scene files, which is the first real use of the scene
  format by something other than a test.
- No UI toolkit, so the title screen and the pause menu are drawn with sprites.
  That constraint is data for M4, not a problem to solve now.
- Save state is a single small file: which rooms are cleared, where the player
  is. Distinct from scenes, which describe the world as authored.
- Every friction point gets written down as it appears. That list is the real
  output of this milestone.

## Revisiting

If the platformer is finished and the friction list is thin, the mining loop is
the natural second game — not a replacement for this decision, a follow-on.
