# 015 — Asset identity: the path, not a uuid

**Status:** Accepted
**Date:** 2026-09-09

## Context

An actor field needs to name an asset — a sprite, a sound, a tileset — in a way
that survives being written to a scene file and read back. `ActorId` cannot
serve: it identifies a live actor in a world, not a file on disk.

`docs/roadmap.md` M2.5 said "uuid-backed", written before scenes existed. Now
that scene files are TOML meant to be read and edited by hand
(`docs/decisions/012` and the scene format), that choice deserves a second look.

## Options

**A. A uuid, with a `.meta` sidecar per asset.** Unity's model. Moving or
renaming a file breaks nothing, because the identity lives beside the file
rather than in its path.

**B. The path, relative to the project root.** Godot's `res://` model. The
identity is what the developer already types.

## Decision

**The path.**

The deciding cost is asymmetric. With uuids, every scene file reads

```toml
texture = "8f3a91c4-...-2b7e"
```

which is unreadable, undiffable, and unwritable by hand — permanently, on every
file, for every developer. That undoes what the scene format was for.

With paths, the cost appears only when an asset is renamed or moved, which is
rare, and which an editor can fix by rewriting the references it already knows
about. A rename without the editor breaks a reference loudly, at load time, with
the missing path in the message.

## Consequences

- `AssetId` wraps a normalised relative path. Cheap to clone (`Arc<str>`),
  comparable, hashable, and readable in a file.
- Assets resolve against a project root, so a scene never contains an absolute
  path and a project stays movable between machines.
- Paths are normalised on construction — separators, `./`, casing on the
  platforms that need it — so two spellings of one asset are one asset.
- No `.meta` files for now, and none needed until an asset carries import
  settings of its own.

## Revisiting

Adding a uuid later stays possible: a `.meta` beside each asset, and a manifest
mapping uuid to path. Scene files would then hold either form, and the loader
would resolve both. That is a migration, not a rewrite — which is why deferring
it is safe.
