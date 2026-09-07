# 009 — Reflection in Wave 0

**Status:** Accepted
**Date:** 2026-09-07

## Context

Reflection — listing a type's fields, reading and writing them by name at
runtime — is unglamorous infrastructure with no visible payoff. The natural
instinct is to add it later, when the inspector needs it.

## Decision

Reflection is built in Wave 0, before rendering, before the first sprite appears
on screen.

## Alternatives

**Add it when the inspector needs it (Wave 4).** Rejected. By then, scene
serialisation exists and will have been written by hand. Retrofitting reflection
means rewriting it.

**Never build it; hand-write serialisation with serde and hand-build inspector
panels.** Rejected. It works for ten actor types and collapses at a hundred.
Every new actor would need a hand-written inspector panel, which guarantees that
some actors simply never get one.

## Consequences

**What it unlocks — four features that all reduce to the same mechanism:**

| Feature | Needs |
| --- | --- |
| Inspector | List a selected actor's fields and edit them |
| Scene serialisation | Write every field to text, read it back |
| Scripting | Read and write properties from another language |
| Hot reload | Preserve state across a reload, by name |

Build it once, get four. Skip it, implement each ad hoc and incompatibly. This is
the classic engine mistake.

**Cost:** a derive macro and a `Value` enum before anything is visible on screen.
Perhaps a week of work with no demo at the end, which is the real reason it gets
skipped.

**Explicitly not required:** that it be beautiful. Reflection can be revised in
Wave 4 when the inspector reveals what it got wrong. It cannot be *absent*, and
it also does not need to be perfect before Wave 1 starts.
