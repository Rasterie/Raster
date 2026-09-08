# Contributing

Raster is in its design phase and moving fast. The API breaks often, and will
keep breaking until 1.0 — see [non-goals.md](non-goals.md).

That makes this a poor moment for large contributions, and a good one for
issues: a bug report, a question about a design decision, or a case the
documents do not cover is more useful right now than a patch.

## Before writing code

Open an issue first, or comment on an existing one. Not a formality — the
design documents in this folder record decisions that a patch may unknowingly
contradict, and it is easier to say so before the work than after.

Read [decisions/](decisions/) for anything touching architecture. If a change
conflicts with a recorded decision, that decision can be revisited — but it
happens by writing a new decision record, not by quietly diverging.

## Working on a change

Branch from `dev`. Name the branch for what it does: `feat/tilemap-chunks`,
`fix/collision-tunnelling`, `docs/clarify-actors`.

**One commit per change.** A commit that renames a field and fixes a bug is two
commits. Subject lines are one line, imperative, 72 characters at most.

**No trailers.** No `Co-Authored-By`, no tool attribution, no session links.

## Before opening a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must pass. CI runs them on macOS, Linux and Windows, so a change that
only builds on one platform will be caught — but catching it locally is faster.

Every commit in a pull request should compile and pass its tests on its own. A
`git bisect` that lands on a broken commit wastes more time than splitting the
work carefully saves.

## Code

Written in English — code, comments, identifiers, documentation. Issues and pull
requests can be in French or English.

**Comments explain why, never what.** If code needs a comment to say what it
does, change the code. The comments worth writing are the ones recording a
non-obvious choice: an invariant, a measured constraint, a reason not to do the
simpler thing.

**No `unsafe` without a `// SAFETY:` comment** justifying the invariant it
relies on. Most crates here forbid it outright.

**Tests go in `tests/`**, against the public API. Use `#[cfg(test)]` only for
what is private and unreachable from outside — see
[decisions/010-testing-layout.md](decisions/010-testing-layout.md).

Write the test that would have caught the bug. A fix without one invites the
bug back.

## Performance claims

Decisions about performance are made by measurement, not argument — see
[decisions/013-actor-storage.md](decisions/013-actor-storage.md) for what that
looks like in practice. A pull request that claims a speed-up should carry the
numbers, including what was not measured.
