# 012 — Readonly is an editor concept, and renames are the serialised name

**Status:** Accepted
**Date:** 2026-09-08

## Context

Two attributes turned out to have a subtlety that only appeared once round-trip
tests were written, and both had produced a real bug.

`#[property(readonly)]` marks a field the inspector must not offer for editing.
The first implementation refused *every* reflected write, including the one
scene loading performs — so any type with a readonly field could not be
restored from a file.

`#[property(rename = "hp")]` lets a field be renamed in code without
invalidating existing scenes. The first implementation wrote under the new name
and read back under the Rust name, so every renamed field silently reverted to
its default on load. Nothing errored; the value simply vanished.

## Decision

**Readonly constrains the editor, not persistence.** `set_field` honours it;
`set_field_unchecked` and `apply` do not.

**The serialised name is what files carry.** `to_value` writes it, and
`apply` — via `set_field_by_serialized_name` — reads it. The Rust field name
addresses a field from code; the serialised name addresses it from a file.

## Alternatives

**Readonly refuses every write.** Rejected: it makes the attribute unusable,
since any field carrying it stops persisting.

**Readonly is advisory, checked only by the inspector.** Tempting, and rejected
because it puts the rule somewhere it can be forgotten. `set_field` is also the
path scripting will take, and a script should not write a field the inspector
refuses.

**Accept both names when loading.** Rejected. It hides the moment a rename takes
effect and makes the file format ambiguous — two spellings for one field, with
no rule about which wins.

## Consequences

**Three write paths**, each with a stated purpose: `set_field` for the editor
and scripting, `set_field_unchecked` for tools that must bypass readonly, and
`set_field_by_serialized_name` for loading files.

**Renaming a field stays safe** as long as the old serialised name is kept in
the attribute — which is the entire point of having it.

**Both rules are covered by round-trip tests**, because both bugs were invisible
to any test that only checked a single direction.
