# Non-goals

What Raster will not do. Each entry has a reason, because a non-goal without one
gets quietly reversed the first time someone wants the feature.

Reversing any of these is legitimate — but it should happen as a recorded
decision in `decisions/`, not as a drift.

## 3D

Not a reduced 3D engine, not 2.5D, not "2D now, 3D later".

Every engine that supports both makes 2D the compromise: the scene graph carries
a Z axis nobody uses, the renderer sorts in 3D, the physics is a 3D solver
constrained to a plane, and pixel-perfect rendering fights the pipeline.

Being 2D-only is what lets the renderer be a sprite batcher, collision be
axis-aligned AABB against a grid, and the camera be a rectangle. That is the
entire value proposition. Adding 3D would not extend Raster; it would delete the
reason it exists.

## A full physics engine

No rigid bodies with rotation, no joints, no constraint solver.

2D action games overwhelmingly use axis-aligned collision with hand-tuned
movement, because a physics solver makes a platformer feel wrong — you want a
character that stops instantly, not one that conserves momentum.

Rotated colliders are the specific exclusion: rotation stays visual, collision
stays axis-aligned. This makes tile collision an order of magnitude simpler and
faster, and it is what most 2D games do anyway.

A game needing real physics can integrate `rapier2d` itself.

## A code editor

Raster does not include a text editor, a debugger or a compiler frontend. You
write Rust in whatever you already use.

Building a decent code editor is a multi-year project, and there are excellent
ones already. Effort goes into the editors that do not exist — sprite, tilemap,
cue — not into a worse version of something you already have open.

## Visual scripting for gameplay

Node graphs are for materials and audio cues, where a graph genuinely reads
better than code. Gameplay logic is written in text.

Unreal's Blueprint spaghetti is the known failure mode: past a certain size, a
logic graph becomes unreadable, unrefactorable and undiffable. A solo project has
no reason to reproduce it.

## Networking and multiplayer

Not in scope. Retrofitting networking is famously hard, so this is worth being
honest about rather than pretending otherwise.

What the design does do is avoid making it impossible: actors addressed by id, a
message system, and a defined frame order are the prerequisites for adding it
later. That is as far as the accommodation goes — no prediction, no rollback, no
authority model.

## Console platforms

PC first — Windows, macOS, Linux. Web is plausible later since `wgpu` targets
WebGPU and Rasterie already runs there.

Consoles require NDAs, licensed SDKs and per-platform certification. Not
compatible with an open source project, and not compatible with a solo one.

## A plugin system, early

The editors are built natively into the engine at first, on purpose.

A plugin API designed before there are plugins is designed blind. Building the
sprite and cue editors in-tree teaches what an editor actually needs — asset
access, undo participation, docking, live preview — and *then* that surface can be
extracted, validated by two real cases.

Extracting a plugin API later is refactoring. Designing one first is guessing.

## Backwards compatibility, before 1.0

The API will break. Frequently, and without deprecation cycles.

Stability before the design is proven means preserving mistakes forever. Once
there are users and a 1.0, this reverses — but not before.

## Asset generation as the point

Rasterie generates pixel art, and Resonance can generate sound. That is not the
thesis.

The thesis is that the **tools live inside the engine** — no round trip to
Aseprite and back, no exporting WAVs from a DAW. Generation is a feature of one
of those tools, not the engine's reason for existing. The developer stays in
control of the content.

## Being Unity or Godot

Raster will not have their feature breadth. A solo project cannot, and trying
produces a worse version of everything.

What it can have is a specific, coherent point of view: 2D only, an actor model
where an entity is a type in a file, and dedicated editors grouped into three
domains. Someone who wants breadth should use Godot — it is very good.
