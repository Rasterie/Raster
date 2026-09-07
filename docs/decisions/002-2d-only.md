# 002 — 2D only, permanently

**Status:** Accepted
**Date:** 2026-09-07

## Context

The initial motivation was "there is no real 2D engine — they are all 3D engines
with a 2D mode". That premise is not accurate and should be corrected in the
record: **Godot's 2D renderer is a genuinely separate pipeline**, with its own
node types, its own physics and real pixel-perfect support. It is a first-class
2D engine, and a very good one.

So "no 2D engine exists" is not the justification. The justification is narrower
and still holds: engines that support both make 2D the compromise. The scene
graph carries a Z axis nobody uses, the renderer sorts in 3D, physics is a 3D
solver constrained to a plane. Even Godot, which does better than most, carries
generality that a 2D-only engine would not.

The question is therefore not whether 2D-only is *possible* but whether it buys
enough to justify building an engine.

## Decision

Raster is 2D only. Not a reduced 3D engine, not 2.5D, not "2D now, 3D later".

## Alternatives

**2D with a 3D escape hatch.** Rejected. The escape hatch is what compromises the
2D design: as soon as the renderer must handle arbitrary transforms, the sprite
batcher becomes a general mesh renderer and the pixel-perfect guarantees weaken.

**Build on Godot instead.** Genuinely considered, and the honest answer is that
it would be faster. A framework layer over Godot could impose the actor model in
a few months rather than years, and inherit a mature renderer, physics and export
pipeline.

Rejected because the goal is not only the actor model. It is the domain-grouped
editors with Rasterie and Resonance living inside the engine, and that is not
something a layer on top of Godot can provide. The decision is also, explicitly,
a choice to build an engine — which is a legitimate goal in itself as long as it
is made knowingly.

## Consequences

**Easier:** the renderer is a sprite batcher; collision is axis-aligned AABB
against a grid; the camera is a rectangle; pixel-perfect rendering is a
guarantee rather than a setting to fight for.

**Harder:** nothing, for the target games.

**Foreclosed:** any project that later wants 3D. This is accepted. Adding 3D
would not extend Raster, it would delete the reason it exists.

**Also foreclosed:** rotated colliders. Rotation stays visual; collision stays
axis-aligned. This makes tile collision an order of magnitude simpler and is what
most 2D games do anyway.
