# 017 — Audio output: `cpal` directly, Resonance unlinked

**Status:** Accepted
**Date:** 2026-09-09

## Context

`raster-audio` mixes into a slice. Nothing sends that slice to a sound card yet.

`docs/domains/audio.md` and `docs/decisions/008` describe Resonance as a
foundation Raster builds on, and Resonance already wraps `cpal` in its
`output::realtime` module. Reusing it would avoid a dependency.

Three facts settle this differently than the roadmap assumed:

- **Resonance lives in another organisation** — `Dreamwave-Interactive`, not
  `Rasterie` — and the repository is **private**. Raster is public.
- Its local working copy is ahead of what was pushed, and it is under active
  development as a composition application.
- It *generates* sound; it does not read it. Its `play_realtime(source, seconds)`
  is a composition tool's API, not a game loop's.

## Decision

**Depend on `cpal` directly. Do not link Resonance at all for now.**

A git dependency on a private repository would break the CI, which has no
credentials for it, and break the build for anyone cloning a public repository.
A local path dependency would build on exactly one machine. Neither is a
dependency; both are a way to make Raster unbuildable.

Going through Resonance would also invert the boundary `docs/decisions/008`
draws. A game's output loop pulls blocks on a callback with a fixed budget;
Resonance's plays a source for a number of seconds. Fitting one to the other
means changing Resonance for Raster's sake — which that decision names as the
signal the boundary is wrong.

`cpal` is what Resonance itself uses, so this adds no new class of dependency to
the tree, only a direct edge instead of a transitive one.

## Consequences

- `raster-audio` owns the output stream: device selection, the callback, and the
  ring buffer that feeds it.
- The mix stays testable without a device — `render` fills a slice, and the
  output layer is the only part that needs hardware.
- Resonance stays free to move, be published, or stay private, without Raster's
  build depending on the answer.

## Revisiting

When Resonance is reachable from a public build — published to crates.io, or
moved into `Rasterie` and made public — synthesised sources become available to
cues as `Source` implementations. That is an addition on top of what exists, not
a replacement: the mix, the buses and the output stream stay where they are.
