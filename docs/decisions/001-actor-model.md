# 001 — Actor model rather than a scene tree or ECS

**Status:** Accepted
**Date:** 2026-09-07

## Context

The engine needs a way to represent entities. Three established models exist in
the engines that matter, and they produce very different code.

The trigger for this project is that in a scene-tree engine, **an entity has no
boundary**: a sprite is not part of a player, it is a peer in the tree with its
own transform, able to exist alone. A moderately complex entity becomes six
nodes, and the logic lives on whichever one holds a script. Reading the code does
not tell you what the entity is.

Unreal's model does not have this problem. `AMyCharacter` is a class: one file,
its components declared inside it.

It is worth being accurate, because the difference is smaller than it first
appears — Unreal's components also form an attached tree, rooted at a
`RootComponent`. The real distinctions are:

1. A component is *attached to* an Actor that remains the identity, rather than
   being a peer in a tree.
2. The script lives on the entity as a class, in one place.
3. The entity has a name and a file, rather than being diffuse.

## Decision

An actor is a Rust type. Its components are its fields. Its behaviour is its
methods. The world is a flat collection. Actors are addressed by `ActorId`.
Parenting exists but is optional.

## Alternatives

**A scene-tree model.** Rejected — it is the reason this project exists. It also
forces every entity into a hierarchy it does not need; most entities in a 2D game
have no meaningful parent.

**ECS (Bevy-style archetypes).** Rejected as the *surface* model, though not
necessarily as the storage. An ECS is excellent for simulating a hundred thousand
entities and poor for expressing "a player is this specific thing". Queries and
systems scatter one entity's behaviour across the codebase, which is the same
legibility problem as the node tree arriving from the other direction.

**A dynamic component list.** Rejected — a `get_component::<T>()` returning an
`Option` you unwrap is a runtime check for something the compiler could know. If a
`Player` has a `Sprite`, that should be a field.

## Consequences

**Easier:** reading an entity definition; the compiler knowing which components
exist; serialisation and inspection via reflection; a scripting layer, because
id-based access is already the norm.

**Harder:** iterating one component type across all actors, since data is not
laid out contiguously by component. This is the central performance question and
it is why the storage model is still open — see `architecture/actors.md`.

**Foreclosed:** the ECS performance ceiling for very large homogeneous entity
counts. Mitigated by not using actors for those cases at all: particles,
projectiles and tiles get their own tight storage. Tiles in particular are a
chunked grid, never actors.

**Required downstream:** `ActorId` rather than references, a single `Ctx`
command surface, and reflectable fields. Those three are what make the inspector
and later scripting possible, and they are cheap now and expensive to retrofit.
