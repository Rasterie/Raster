# 004 — Rust as the engine language

**Status:** Accepted
**Date:** 2026-09-07

## Context

The engine needs an implementation language, and the choice is close to
irreversible.

Two existing assets weigh on it: **Resonance** is already written in Rust —
about 2,000 lines of working synthesis and sequencing with a clean `Source`
trait — and **Rasterie** is TypeScript that was already a candidate for a Rust
rewrite.

## Decision

Rust, for the engine and every crate in the workspace.

## Alternatives

**C++.** The conventional choice, the one Unreal and most engines make. Better
tooling for graphics debugging, more existing libraries, no borrow checker to
fight. Rejected primarily because Resonance already exists in Rust and would have
to be rewritten, and because the safety guarantees matter more on a solo project
where there is nobody to catch a use-after-free in review.

**Zig.** Attractive — simpler than Rust, excellent C interop, good for engines.
Rejected as too young: the language is still changing, the ecosystem is thin, and
`wgpu` has no equivalent.

**C#.** Rejected as the *engine* language. It is a reasonable scripting language
later (see 006), but a GC pause in the render loop is not acceptable and the
audio thread would be worse.

## Consequences

**Easier:** Resonance integrates as a dependency with no changes; `wgpu` gives
Vulkan, Metal, DX12 and WebGPU from one backend; fearless concurrency matters for
the audio thread specifically; and WASM keeps the browser reachable, which
matters since Rasterie already runs there.

**Harder:** Rust is a demanding language, and the actor model has to be designed
around the borrow checker rather than against it. This is precisely why actors
are addressed by `ActorId` rather than by reference — a constraint that turns out
to also be what makes scripting possible later.

**Consequence for Rasterie:** the engine will need a Rust port to be embedded as
the sprite editor. That is real work, and it is not urgent — the web app keeps
its TypeScript version, and the port happens when Wave 5 arrives. The port's
justification is portability across three targets, not performance.
