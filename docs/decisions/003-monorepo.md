# 003 — Cargo workspace monorepo

**Status:** Accepted
**Date:** 2026-09-07

## Context

Raster is many subsystems, each substantial enough to be its own project: a
renderer, an audio runtime, a UI toolkit, a physics layer, seven editors. They
need to be separated somehow.

There is relevant history here. An earlier proposal to put the Rasterie web app
and its engine in a monorepo was rejected, for good reasons: the folder became
unwieldy and a monorepo is unpleasant to manage alone.

This case is different, and the difference is worth stating so the earlier
decision does not get misapplied.

## Decision

A single Cargo workspace containing every Raster crate. The two foundation
libraries — `rasterie-engine` and `resonance-core` — stay outside, in their own
repositories.

## Alternatives

**One repository per crate.** Rejected. The crates change together; a rename
crosses ten of them at once; nobody consumes `raster-render` without
`raster-core`. Separate repositories would mean a version matrix and coordinated
releases for what is really one artefact.

**One crate for everything.** Rejected. Compile times, no enforced boundaries,
and no way to keep the editor out of a shipped game — which is the layering rule
the whole architecture depends on.

## Consequences

**Easier:** one `cargo test` and one `cargo build`; atomic cross-crate refactors;
no version matrix; boundaries enforced by the compiler rather than by discipline.

**Harder:** the repository is large, which was the objection last time.
Mitigated by the fact that a Cargo workspace is genuinely designed for this —
unlike a JS monorepo, it needs no extra tooling, no workspace manager, no
hoisting rules.

**Why the Rasterie decision does not apply here:** Rasterie's web app and engine
are two products with separate release cadences and a stable API between them.
Raster's crates are one product with internal seams. The earlier decision was
right for its case and would be wrong for this one.

**Kept outside:** `rasterie-engine` and `resonance-core` are independently
useful — one is a pixel art generator that runs on the web, the other a synthesis
library usable without a game engine. Keeping them separate keeps their APIs
honest, since neither can quietly grow a dependency on Raster types.
