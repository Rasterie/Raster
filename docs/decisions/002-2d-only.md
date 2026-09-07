# 002 — 2D only, permanently

**Status:** Accepted
**Date:** 2026-09-07

## Context

Most engines treat 2D as a mode within a 3D system. Where that happens, 2D is
the compromise: the scene graph carries a Z axis nobody uses, the renderer sorts
in 3D, physics is a 3D solver constrained to a plane, and pixel-perfect rendering
fights the pipeline.

Engines that do 2D well still carry generality a 2D-only engine would not — the
abstractions have to accommodate both cases.

The question is what a strictly 2D design buys.

## Decision

Raster is 2D only. Not a reduced 3D engine, not 2.5D, not "2D now, 3D later".

## Alternatives

**2D with a 3D escape hatch.** Rejected. The escape hatch is what compromises the
2D design: as soon as the renderer must handle arbitrary transforms, the sprite
batcher becomes a general mesh renderer and the pixel-perfect guarantees weaken.

**A framework layer on top of an existing engine.** Genuinely considered, and
the honest answer is that it would be faster: the actor model could be imposed in
a few months rather than years, inheriting a mature renderer, physics and export
pipeline.

Rejected because the goal is not only the actor model. It is the domain-grouped
editors with Rasterie and Resonance living inside the engine, and a layer on top
of someone else's editor cannot provide that. The decision is also, explicitly, a
choice to build an engine — a legitimate goal in itself, as long as it is made
knowingly.

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
