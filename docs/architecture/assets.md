# Assets and scenes

## Code is the source of truth

The rule, stated concretely:

**An actor type is defined by its Rust file. A scene is a list of instances and
the values they override.**

Deleting every scene in a project loses level layouts. It does not lose what a
`Player` is. Where the scene file *is* the entity definition, deleting it loses
the entity itself — which is what Raster avoids.

This has a practical consequence that matters daily: scene files stay small,
readable and diffable, because they contain differences from defaults rather
than full descriptions of everything.

## Asset identity

Every asset has a stable identity independent of its path, so that moving or
renaming a file does not break references.

```rust
pub struct AssetId<T> {
    uuid: Uuid,
    _marker: PhantomData<T>,
}
```

The type parameter is compile-time safety: `AssetId<Texture>` cannot be passed
where `AssetId<Cue>` is expected.

A manifest maps uuid to path, and is versioned alongside the project. The `.meta`
file next to each asset carries its uuid and import settings — the approach Unity
uses, which is well proven and survives file moves.

The `asset!` macro resolves a path at compile time to a typed id, so a typo is a
build error rather than a runtime `None`:

```rust
sprite: Sprite::new(asset!("actors/player.sprite")),
```

## Asset types

Each belongs to a domain and has a dedicated editor. This table is the concrete
form of "one asset type, one editor".

| Asset | Extension | Domain | Editor |
| --- | --- | --- | --- |
| Sprite | `.sprite` | Visual | Sprite editor (Rasterie) |
| Palette | `.palette` | Visual | Palette editor (Rasterie) |
| Tileset | `.tileset` | Visual | Tileset editor |
| Tilemap | `.tilemap` | Visual | Tilemap painter |
| Animation | `.anim` | Visual | Animation timeline |
| Material | `.material` | Visual | Material graph |
| Widget | `.widget` | Visual | UI designer |
| Font | `.font` | Visual | Font editor |
| Cue | `.cue` | Audio | Cue graph (Resonance) |
| Instrument | `.instr` | Audio | Synthesis editor (Resonance) |
| Track | `.track` | Audio | Sequencer (Resonance) |
| Scene | `.scene` | Game | Scene viewport |

## File format

Text, always. Human-readable, diffable, mergeable. A binary format may be added
later as a *build* artefact for shipping, never as the authoring format.

The likely choice is TOML for hand-editable documents and a compact custom
format for grid data like tilemaps, where TOML would be absurd for a million
tiles.

A scene, illustratively:

```toml
[scene]
name = "surface_spawn"

[[actor]]
type = "Player"
id = "a1b2c3d4"
position = [120.0, 64.0]

[[actor]]
type = "Chest"
id = "e5f6a7b8"
position = [200.0, 64.0]
speed = 0.0          # only fields that differ from the type's defaults
contents = ["item:torch", "item:rope"]
```

The `type` field is the reflected type name; the remaining keys are reflected
property writes. This is why [reflection](reflection.md) is Wave 1 work — scene
loading is impossible without it.

## Loading

Assets load asynchronously, with a handle that resolves later:

```rust
let tex: Handle<Texture> = ctx.assets.load(asset!("actors/player.sprite"));
```

A handle is reference-counted; dropping the last one queues the asset for
unloading. A handle that has not finished loading yields a placeholder — a
magenta checker for textures, silence for audio — rather than blocking or
panicking. Games must never stutter because an asset was requested late.

## Hot reload

Editing an asset while the game runs updates it live. This is not a luxury: it is
the entire point of having the editors inside the engine. Changing a sprite in
the sprite editor and seeing it change in the running game, with no export step,
is the experience the whole project is arranged around.

Implementation: the asset system watches the project directory, reloads on
change, and swaps the asset behind existing handles. Actors holding a
`Handle<Texture>` need no notification — they hold the same handle, and its
contents changed.

## Importing

External files are imported rather than used directly: a PNG becomes a Sprite
asset, a WAV becomes a Cue. The import step records its settings in the `.meta`
file so that re-importing is deterministic.

This matters for Raster specifically. A sprite authored in Rasterie carries its
parameters — the shape grammar, the palette, the generation settings — not just
pixels. An imported PNG carries only pixels. Both are valid Sprite assets; one
can be re-generated and the other cannot, and the asset must be honest about
which it is.

## Open questions

- Whether scenes can nest, and if so how overrides on a nested scene work. This
  is Unity's prefab problem and it is genuinely hard; deferring it is reasonable,
  ignoring it forever is not.
- Whether tilemaps are assets or part of scenes. For a Terraria-like the world is
  generated and mutated at runtime, so it is probably neither — a save-file
  concern rather than an asset one.
- How a shipped build packs assets: a single archive, or loose files.
