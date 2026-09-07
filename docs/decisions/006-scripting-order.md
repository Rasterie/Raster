# 006 — Rust first, scripting much later

**Status:** Accepted
**Date:** 2026-09-07

## Context

Rust is a demanding language for gameplay code, where iteration speed matters
more than raw performance. Engines solve this with a scripting layer — GDScript,
C#, Blueprint.

The instinct is to plan the scripting language early, since it shapes the whole
API. The question is whether to act on that instinct now.

## Decision

Four phases, in order:

1. **Rust only** — the engine and the first game
2. **Rhai**, to validate that the API is genuinely scriptable
3. **A purpose-built language**, once scripting's real requirements are known
4. **C#**, only if there is ever demand for it

Nothing in phases 2–4 is built during Waves 0–7. What *is* done from the first
commit is designing the engine so scripting is possible later.

## Alternatives

**Design the scripting language first.** Rejected, and this is the important
one. You cannot expose an API that does not exist. Building bindings against a
moving engine means exposing functions that get deleted and doing the work twice.
The scripting languages that work well were extracted from working engines, not
designed ahead of them.

**C# via .NET hosting, early.** Rejected for now. It buys a mature language and
existing developers; it costs a 100+ MB runtime dependency, intricate
marshalling, a moving GC next to Rust's ownership model, and painful
cross-boundary debugging. Engines that support it have spent years on that
integration and it remains among their most fragile areas. For a project with no
users, that cost buys nothing.

**No scripting, ever.** Rejected — but noted that Rust-side iteration can be
improved independently with a hot-reloaded game library, which is worth doing in
any case since Phase 1 is Rust-only.

## Consequences

**Required now, cheaply:** three disciplines that cost almost nothing today and
are extremely expensive to retrofit —

1. actors addressed by `ActorId`, never by Rust reference across an API
2. every capability reachable through `Ctx`, one enumerable command surface
3. actor fields reflectable

Hold those and Phase 2 is an adapter of a few thousand lines. Ignore them and it
is an engine rewrite.

**Deferred:** the language design itself, which is a year of part-time work plus
the surrounding comfort — diagnostics, a debugger, autocompletion — without which
nobody uses it, including its author.

**Precedent that makes Phase 3 plausible:** Resonance already contains a
hand-written language, `rnc`, with a tokenizer, parser, AST and evaluator in
about 900 lines. The skill exists; only the scale is larger.
