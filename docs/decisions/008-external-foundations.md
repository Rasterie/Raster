# 008 — Rasterie and Resonance stay independent

**Status:** Accepted
**Date:** 2026-09-07

## Context

Two of Raster's domain tools already exist as working, independent projects:
Rasterie, a parametric pixel art generator, and Resonance, a synthesis and
sequencing library in Rust. Both predate this engine and were built for their own
sake.

They could be absorbed into the workspace, or consumed as dependencies.

## Decision

Both stay in their own repositories, with their own releases. Raster consumes
them as crates. Neither may contain Raster types.

## Alternatives

**Absorb them into the workspace.** Rejected. Both are useful outside a game
engine — Rasterie runs on the web today and has its own users; Resonance is a
synthesis library that has nothing to do with games. Absorbing them would make
that impossible.

**Fork them into the workspace and let the copies diverge.** Rejected outright.
Two copies of the same engine is a known failure mode, currently visible in the
Rasterie web app, and it produces silent drift.

## Consequences

**Easier:** each project keeps its own identity and release cadence; the
boundaries stay honest, because neither can quietly grow a dependency on engine
internals; and Rasterie's web app continues to exist unaffected.

**Harder:** a change needed in both places requires two repositories and a
version bump. This is the cost of the boundary and it is worth paying.

**The test that keeps it honest:** if a change to Raster requires a change to
`resonance-core` or `rasterie-engine`, the boundary is wrong and the fix belongs
in Raster, not in the foundation.

**Consequence for Rasterie specifically:** the engine needs a Rust port before it
can be embedded as the sprite editor. That port is a separate project, justified
by portability across web, editor and runtime — not by performance.
