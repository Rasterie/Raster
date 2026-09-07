# Glossary

Terms used consistently across these documents. Where a word carries a different
meaning elsewhere, that is noted — most confusion between engines comes from the
same word meaning three things.

### Actor
A Rust type representing one entity in the world. Its components are its fields,
its behaviour is its methods. The core abstraction of Raster.

Closest relative: Unreal's `AActor`. Note that it is *not* a container with a
dynamic component list looked up at runtime — an actor's components are known to
the compiler.

### ActorId
A generational index identifying an actor. The only way to reference an actor
across an API boundary. Invalidated when the actor is destroyed, so a stale id
resolves to `None` rather than to a recycled slot.

### Anchor
A named point on an actor that another actor can attach to — a hand, a muzzle, a
mount point.

### Asset
A piece of content with a stable identity, edited by a dedicated editor: a
sprite, a palette, a cue, a scene. Identified by uuid rather than path, so moving
a file does not break references.

### AssetId
A typed, stable handle to an asset. `AssetId<Texture>` cannot be passed where
`AssetId<Cue>` is expected.

### Attachment
An optional parent-child relation between actors, affecting transform resolution
and lifetime. Not the structure that gives actors existence — unlike a scene
tree.

### Autotile
Choosing a tile's visual variant from its neighbours, so a painted region gets
correct edges and corners automatically. Raster uses the 47-variant bitmask
scheme.

### Behaviour
The trait carrying an actor's lifecycle hooks: `tick`, `on_spawn`, `on_collide`
and the rest. All optional.

### Bus
A named mixing channel in the audio domain — Master, Music, SFX, UI, Ambience.
Carries volume, effects and snapshots.

### Chunk
A fixed-size block of tiles, the unit of tilemap storage, streaming and mesh
rebuilding.

### Component
A struct providing one capability to an actor, held as a field. Has no
independent existence and cannot be placed in a scene alone.

The distinction this design turns on: a component is not a node that could be
placed in a scene by itself, and not an entry in a runtime lookup table.

### Coyote time
A short window after leaving a ledge during which a jump still registers. Along
with input buffering, the difference between controls that feel good and controls
that feel broken. Provided by the engine rather than reimplemented per game.

### Ctx
The context handed to an actor during a lifecycle hook. The single, enumerable
surface through which every engine capability is reached — and therefore the
thing scripting will eventually bind to.

### Cue
The unit of sound a game plays. Not a file: an assembled behaviour authored as a
graph — random selection, pitch variation, envelope, bus routing.

Modelled on Unreal's Sound Cue.

### Domain
One of the three top-level groupings — Game, Visual, Audio. A domain owns its
data model, its runtime and its editors, and editors within one share their
foundations by construction.

### Fixed tick
A lifecycle hook running at a fixed timestep, for physics-coupled logic.
Distinct from `tick`, which runs once per frame with variable delta.

### Handle
A reference-counted pointer to a loaded asset. May be unresolved while loading,
in which case it yields a placeholder rather than blocking.

### Hot reload
Editing an asset while the game runs and seeing it update live. The reason for
putting the editors inside the engine.

### Layer
Two unrelated meanings, disambiguated by domain. *Visual:* a render ordering
group. *Physics:* a collision category used with masks. Never used bare in the
docs without context.

### Message
Typed data sent from one actor to another, queued and delivered at a defined
point in the frame rather than re-entrantly. The mechanism actors use instead of
calling each other's methods.

### OKLCH
A perceptually uniform colour space. The basis of Rasterie's ramps, where hue
shifting — cold shadows, warm highlights — is what makes pixel art look
hand-made rather than mathematically darkened.

### Palette
A first-class asset holding colour ramps with their structure, not just a list of
colours. Swapping one recolours every sprite built on it.

### Pixel-perfect
The rendering discipline that keeps pixel art clean: integer scaling,
nearest-neighbour filtering, sprites snapped to the grid, camera interpolated in
sub-pixels. Getting the last pair wrong produces either jitter or blur.

### Reflection
Runtime inspection of types: listing an actor's fields, reading and writing them
by name. The mechanism shared by the inspector, scene serialisation, scripting
and hot reload.

### Scene
A list of actor instances and the property values they override, stored as
readable text. Not a definition of what an entity is — that lives in code.

Contrast with engines where the scene file *is* the entity definition — the
design Raster deliberately avoids.

### Source
Resonance's central trait: anything producing audio samples, via
`next_sample()`. Composable, which is what makes cue graphs possible without
changing Resonance.

### Tick
The per-frame lifecycle hook, with variable delta time.

### Tile
A cell in a tilemap grid. Emphatically *not* an actor — tiles are stored as
compact arrays in chunks. Only tiles with behaviour, like a chest, get an
associated actor.

### Value
The dynamic currency of reflection: the enum every reflected read yields and
every reflected write accepts. One conversion layer shared by the inspector, the
serialiser and scripting.

### Widget
A UI element: an object with identity and persistent state, in a retained-mode
tree. Also the `.widget` asset authored in the UI designer.

### World
The flat collection owning every actor, and the only path to mutate one. Has no
root and no tree.
