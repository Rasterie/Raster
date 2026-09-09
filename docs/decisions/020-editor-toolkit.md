# 020 — The editor uses `raster-ui`

**Status:** Accepted
**Date:** 2026-09-10

## Context

`TODO.md` places a checkpoint at the end of M4: *is `raster-ui` good enough for
the editor? If not, decide here to use `egui` for editor chrome.*

`docs/architecture/editor.md` states the bet — the editor is a Raster
application, built with the same toolkit games use — and names the fallback:
`egui` for editor chrome if the editor needs things games never will.

M4 delivered: a built-in 5×7 font, a theme, constraint layout with anchors,
stable widget identity, event routing (hover, focus, capture, keyboard), and
eleven widgets. Keystone's three screens are built with it.

## What was measured

A probe built an editor-shaped screen — toolbar, asset tree, viewport,
inspector, console — with the toolkit as it stands.

**What already works.** Constraint layout produces the panel arrangement
directly: a 1280×800 shell splits into a 28px toolbar, a 240px tree, a 760px
viewport, a 280px inspector and a 120px console, in two `stack` calls. Event
routing handles an inspector's fields, including focus order and capture.

**What is missing**, in order of how structural it is:

| Missing | Kind | Cost |
| --- | --- | --- |
| Scissor clipping | Renderer change | ~30 lines in the batcher |
| Scroll area | Widget, needs clipping | Small, once clipping exists |
| Resizable splitter | Widget | Small |
| Mouse wheel routing | Input plumbing | Small |
| Drag and drop between panels | `Ui` state | Medium |
| Numeric drag field | Widget | Small |
| Dropdown menu, tooltip | Widgets, need clipping | Small |

Clipping is the only one that is not a widget. `wgpu` exposes
`set_scissor_rect` on a render pass, and the batcher already draws in runs
inside a single pass, so a scissor rectangle joins the sort key and one call is
added per run. It is a contained change, not a rewrite.

**Cost of the text approach.** Each lit pixel of a glyph is its own quad. An
editor screen costs about 13,400 quads against a game screen's 380 — 35×. CPU
layout for 30,600 quads measures **0.37 ms**, 2.2% of a frame at 60fps. That is
wasteful on the GPU and not a wall.

## Decision

**The editor uses `raster-ui`. `egui` is not adopted.**

Nothing found is a capability the toolkit cannot express. Every gap is either a
widget to write or a contained renderer change, and the two hardest parts —
layout and event routing — already work at editor scale.

Adopting `egui` would mean two toolkits, two visual identities, two sets of
conventions, and the thing `docs/architecture/editor.md` calls the point of the
bet — a UI toolkit whose author uses it daily on a demanding application — would
be lost. The fallback exists for a toolkit that *cannot* do the job. This one
can; it is unfinished, which is a different problem with a different fix.

## Consequences

- Scissor clipping lands in `raster-render` before M5's docking, because the
  scroll area, the dropdown and the tooltip all depend on it.
- A glyph atlas replaces pixel-per-quad when it shows up in a profile, not
  before. The measurement above is what to redo, and the API does not change.
- The editor's needs drive `raster-ui` from here, which is the intended
  direction: games got it to eleven widgets, the editor takes it further.

## Revisiting

The honest failure signal is not a missing widget — it is finding a *kind* of
thing the toolkit cannot express: an editor that needs text shaping, or a
hierarchy games never build. None appeared. If one does, this decision is the
one to reopen, with `egui` for editor chrome as the recorded fallback.
