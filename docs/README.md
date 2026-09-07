# Raster — documentation

Raster is a 2D-first game engine written in Rust. This folder holds the design
documents: what the engine is, how it is structured, and — just as important —
what it deliberately will not do.

These documents are written before the code. They will be wrong in places, and
they are meant to be revised as reality pushes back. A document that no longer
matches the code is a bug in the document.

## Start here

| Document | What it answers |
| --- | --- |
| [vision.md](vision.md) | Why this engine exists, and who it is for |
| [architecture/overview.md](architecture/overview.md) | The three domains and how the crates are laid out |
| [architecture/actors.md](architecture/actors.md) | The actor model — the core abstraction |
| [architecture/assets.md](architecture/assets.md) | How assets are identified, loaded and edited |
| [architecture/editor.md](architecture/editor.md) | The editor shell and the per-asset editors |
| [architecture/scripting.md](architecture/scripting.md) | How gameplay is written, now and later |
| [architecture/reflection.md](architecture/reflection.md) | The mechanism the inspector and scripting both need |
| [roadmap.md](roadmap.md) | The order of work, in waves |
| [../TODO.md](../TODO.md) | Every task, grouped by milestone |
| [non-goals.md](non-goals.md) | What Raster will not do, and why |
| [glossary.md](glossary.md) | Terms used consistently across these documents |

## Domains

Raster is organised into three domains rather than a flat list of subsystems.
Each has its own document:

- [domains/game.md](domains/game.md) — actors, world, scenes, physics
- [domains/visual.md](domains/visual.md) — rendering, sprites, tilemaps, animation, UI, materials
- [domains/audio.md](domains/audio.md) — synthesis, cues, mixing, music

## Decisions

Design decisions with lasting consequences are recorded in
[decisions/](decisions/), one file each, with the reasoning and the alternatives
that were rejected. When a decision is reversed, the file stays and gains a note
saying why — the reasoning is worth more than the conclusion.
