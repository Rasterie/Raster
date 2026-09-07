# 007 — Graphs for data, text for logic

**Status:** Accepted
**Date:** 2026-09-07

## Context

Unreal has two graph systems that are often conflated. **Material graphs** and
**Sound Cues** describe dataflow: values transformed through a pipeline.
**Blueprints** describe control flow: gameplay logic with branches, loops and
state.

They look similar and behave very differently at scale.

## Decision

Node graphs are used for materials and audio cues. Gameplay logic is written in
text, in a real programming language.

## Alternatives

**Blueprint-style visual scripting for gameplay.** Rejected. Past a certain size
a logic graph becomes unreadable, unrefactorable and undiffable — the well-known
"Blueprint spaghetti". A solo project has no reason to reproduce it, and the
version control problem alone is disqualifying.

**No graphs at all.** Rejected. For a material or a cue, a graph is genuinely the
better representation: the data flows one way, the node count stays small, and a
live preview at each node is more useful than reading code. This is dataflow, and
graphs are good at dataflow.

## Consequences

**Easier:** one graph implementation serves both material and cue editors;
gameplay stays diffable, refactorable and greppable; no graph-vs-code split in
gameplay where logic could hide in either.

**Harder:** nothing for the intended user, who prefers writing code. Someone who
wanted visual scripting will not find it here.

**Boundary to hold:** the pressure to let graphs "just do a little logic" — a
branch here, a loop there — is how dataflow graphs turn into bad programming
languages. Conditional *selection* within a cue is fine; a state machine in a
graph is not.
