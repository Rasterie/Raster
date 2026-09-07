# 010 — Test layout: per-crate `tests/`, plus one conformance crate

**Status:** Accepted
**Date:** 2026-09-07

## Context

Rust's most visible testing convention puts unit tests in a `#[cfg(test)] mod
tests` block at the bottom of the file they test. Coming from ecosystems where
tests live in a separate directory, this reads as though Rust does not support a
test folder at all.

It does. Any file under a crate's `tests/` directory is compiled as its own
crate that consumes the library from outside — exactly as a user would — and
`cargo test` runs it. Both mechanisms exist and serve different purposes.

The question is which to use where, and whether a single workspace-wide test
crate would be simpler than scattering tests across eighteen crates.

## Decision

Two tiers:

1. **`crates/<crate>/tests/`** — integration tests against the public API. The
   bulk of the test suite.
2. **`#[cfg(test)]` in-file** — reserved for what is private and unreachable from
   outside.

Architecture rules — the runtime not depending on the editor, `raster-core`
depending only on `raster-math` — are enforced in CI by inspecting the dependency
graph, not by a test crate. A dedicated conformance crate was considered and set
aside as more machinery than the rule needs.

## Alternatives

**A single `raster-tests` crate holding every test.** Rejected, for four
concrete reasons:

1. **It can only reach public items.** An external crate cannot see private ones.
   In an engine, the delicate logic is often internal — the component index, the
   generation allocator, autotile neighbourhood computation. Those would become
   untestable, or would have to be made public purely to test them, which is
   worse.
2. **It serialises compilation.** A single test crate depending on all eighteen
   relinks everything when one test changes. Per-crate test targets rebuild only
   what changed.
3. **It loses parallelism.** Cargo compiles and runs test targets across crates
   in parallel. One crate is one unit.
4. **It loses locality.** A physics test living next to physics gets updated when
   physics changes. A test living three directories away silently rots.

**Everything in `#[cfg(test)]` blocks, the default convention.** Rejected as the
primary location. Tests written from inside the crate can reach private state,
which makes it easy to test implementation rather than behaviour — and those
tests break on every refactor without catching real regressions. Testing from
outside keeps the public API honest, because writing the test is the first time
anyone uses it as a consumer would.

## Consequences

**A clean test directory per crate**, which was the original motivation:

```
crates/raster-core/
├── src/
│   └── lib.rs
└── tests/
    ├── world.rs
    ├── reflection.rs
    └── scenes.rs
```

**Architecture rules move to CI.** Checking that the runtime does not depend on
the editor is a question about the dependency graph, which `cargo metadata`
answers directly. A shell step in CI is enough, and it needs no crate.

The precedent is `tests/e2e/engine-autonomy.test.ts` in `rasterie-engine`, which
froze the engine's boundary against regressions. The rule was worth having; here
it lives in CI rather than in a test target.

**A discipline that follows:** if something can only be tested from inside the
crate, that is worth a moment's thought. Sometimes the answer is a legitimate
`#[cfg(test)]` block. Sometimes it means a module is doing something that
deserves to be its own crate with its own public API.
