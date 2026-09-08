# 011 — Generated accessors and explicit registration

**Status:** Accepted
**Date:** 2026-09-08

## Context

Two mechanisms had to be chosen before writing the reflection layer, and both
were left open in `docs/decisions/009-reflection-first.md`.

**How a reflected field is reached.** Either the derive macro generates a match
over field names calling ordinary field access, or `FieldInfo` stores a byte
offset and access goes through raw pointer arithmetic.

**How a type becomes known by name.** Loading a scene turns the string
`"Player"` into a `Player`, which needs a name-to-constructor map the engine
cannot populate on its own.

## Decision

**Generated accessors**, not offsets. `#[derive(Reflect)]` emits a `match` on
the field name whose arms are plain field reads and writes.

**Explicit registration.** The game calls `registry.register::<Player>()`.
Nothing is collected automatically.

## Alternatives

**Raw offsets.** Faster in principle: `FieldInfo` holds a `usize` and access is
pointer arithmetic. Rejected on two counts. It requires `unsafe` in the most
foundational crate of the engine — where a mistake is hardest to trace — and it
requires `#[repr(C)]` on every reflected type to keep the layout predictable,
which is a constraint imposed on every game type for the benefit of tooling. The
speed difference is irrelevant: reflection runs at inspector rates and at scene
load, never in the frame loop.

**Linker-collected registration** (`inventory` or `linkme`). Types register
themselves by placing entries in a linker section, so a game never writes a
registration list. Genuinely more pleasant when it works — and it silently
registers nothing when the code is built as a static library, and behaves
unpredictably under WebAssembly. A registry that is quietly empty produces a
scene that loads zero actors with no error to explain it. Explicit registration
is more typing and fails at a point where the message is obvious.

## Consequences

**No `unsafe` anywhere in reflection**, so `raster-core` keeps
`#![forbid(unsafe_code)]`.

**No layout constraints** on game types: no `#[repr(C)]`, no restriction on
field ordering.

**A registration list to maintain.** A game that forgets to register a type
learns at scene load, by name. That is a worse developer experience than
automatic collection and a much better one than silence.

**Revisit if** profiling ever shows reflected access in a hot path. That would
mean reflection is being used somewhere it should not be, and the fix is
probably to stop doing that rather than to make it faster.
