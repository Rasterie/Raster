# Decisions

One file per decision that has lasting consequences, recording the reasoning and
the alternatives that were rejected.

The point is not the conclusion — it is the reasoning. A conclusion without its
reasoning gets reversed by whoever forgets why it was made, including its author
a year later.

When a decision is reversed, the file **stays** and gains a `Superseded` note
explaining what changed. Deleting it loses the more valuable half.

## Format

```markdown
# NNN — Title

**Status:** Accepted | Superseded by NNN | Revisit at <milestone>
**Date:** YYYY-MM-DD

## Context
What situation forced a choice.

## Decision
What was chosen, stated plainly.

## Alternatives
What else was considered, and why it lost.

## Consequences
What this makes easy, what it makes hard, what it forecloses.
```

## Index

| # | Decision | Status |
| --- | --- | --- |
| [001](001-actor-model.md) | Actor model rather than a scene tree or ECS | Accepted |
| [002](002-2d-only.md) | 2D only, permanently | Accepted |
| [003](003-monorepo.md) | Cargo workspace monorepo | Accepted |
| [004](004-rust.md) | Rust as the engine language | Accepted |
| [005](005-three-domains.md) | Three domains rather than N editors | Accepted |
| [006](006-scripting-order.md) | Rust first, scripting much later | Accepted |
| [007](007-no-visual-gameplay-scripting.md) | Graphs for data, text for logic | Accepted |
| [008](008-external-foundations.md) | Rasterie and Resonance stay independent | Accepted |
| [009](009-reflection-first.md) | Reflection in Wave 0 | Accepted |
| [010](010-testing-layout.md) | Per-crate `tests/` plus a conformance crate | Accepted |
| [011](011-reflection-mechanics.md) | Generated accessors, explicit registration | Accepted |
| [012](012-readonly-and-renames.md) | Readonly is editor-only; renames are the file name | Accepted |
| [013](013-actor-storage.md) | Actor storage: typed pools, decided by measurement | Accepted |
| [014](014-component-index.md) | The component index: a typed path beside reflection | Accepted |
