# 016 — Hot reload: polling, not filesystem events

**Status:** Accepted
**Date:** 2026-09-09

## Context

Editing a sprite and seeing it change in the running game — without restarting —
is what makes an engine pleasant to work in. `docs/decisions/015` made an asset
identifiable by its path, and `AssetStore` already replaces an asset in place,
leaving live handles pointing at the new version. What remains is noticing that
a file changed.

Two ways to notice:

- **A. Filesystem events** — the `notify` crate, which wraps FSEvents, inotify
  and `ReadDirectoryChangesW`. Near-instant, and the usual answer.
- **B. Polling modification times** — walk the loaded assets, `stat` each one,
  compare with what was recorded at load.

## Measurement

Release build, Apple Silicon, warm filesystem cache, one `stat` per loaded
asset:

| Loaded assets | Time per sweep |
| ---: | ---: |
| 10 | 0.018 ms |
| 100 | 0.162 ms |
| 500 | 0.629 ms |

A sweep of 500 assets costs 3.8% of one frame at 60 fps — and a sweep runs a few
times per second, not every frame. At four sweeps per second that is 0.25% of a
CPU second.

## Decision

**Polling**, at an interval the game sets, defaulting to four times per second.

Latency is the only thing events would win, and it is the one thing that does
not matter here: nothing distinguishes 100 ms from 16 ms when the trigger is a
human saving a file in another application. Against that, `notify` brings a
dependency tree, a background thread, three platform backends with three sets of
quirks, and event coalescing to write anyway — editors save by writing a
temporary file and renaming it, so a single save arrives as several events.

Polling only watches what is already loaded, which is the set that can actually
be reloaded. Watching directories would report files no one asked for.

## Consequences

- Reloading is explicit and synchronous: the game calls it, at a point of its
  choosing, and knows nothing is being mutated behind its back.
- Only loaded assets are watched — a new file on disk is not an event, it
  becomes one when something loads it.
- A file being written when the sweep runs is caught on the next sweep, so a
  half-written PNG fails to decode once and reloads correctly right after.
- Nothing prevents adding an event-driven watcher later behind the same API.

## Revisiting

If a project ever loads assets in the thousands, or if a sweep shows up in a
frame profile, the measurement above is the thing to redo — not the argument.
