# Friction log

What hurt while building the MVP (`games/keystone`). This list is the input to
M4–M8: each entry is a thing the engine made harder than it should have been.

Written as encountered, not curated. An entry is not a complaint — it is
evidence that a decision made earlier is now costing something.

## 1. `wgpu` leaked through the public API

Creating a texture required `Texture::from_rgba(gpu, layout, …)`, where `layout`
is a `wgpu::BindGroupLayout`. A game therefore had to add `wgpu` as a direct
dependency to draw anything it built itself.

**Fixed during M3** by re-exporting the type as `raster_render::TextureLayout`.
The underlying question stands: how much of `wgpu` should a game ever see? A
`Texture::solid(gpu, colour)` and `Texture::from_pixels` that take no layout at
all would remove the question entirely.

**Points at:** M4 (painter API), and a general pass over what `raster-render`
exposes.

## 2. No text rendering, so no real menus

The title screen, the pause banner and the death screen are coloured rectangles.
A game cannot say "Press Enter" without a font.

This is known — `raster-ui` is M4 and bitmap fonts are on its list — but it is
worth recording how early it bites: the very first game hits it in its first
hour, before enemies or scoring.

**Points at:** M4, bitmap font rendering. Should be near the front of M4, not
the middle.

## 3. Level geometry had no way to be checked

Three of the first room layouts were unfinishable: platforms sat four tiles above
the floor, for a jump that clears 2.7. Every hand-written test passed, because
they all teleported the player onto the key rather than making it walk there.

The bug was only found by writing a reachability test that walks the tile grid
from the spawn point.

Two conclusions:

- A game needs a way to assert "this level is completable" that does not depend
  on the author remembering to check.
- The jump height, the run speed and the tile size are related by arithmetic no
  one writes down. `jump_height()` in the test is three lines; it belongs in the
  engine, next to whatever a future editor uses to draw a jump arc.

**Points at:** M6 (editor) — a level editor that cannot show reach is a level
editor that ships unfinishable rooms.

## 4. Enemies were placed one tile above the floor

`walker(at(13.0, 9.0))` put an enemy on row 9 when the floor was row 10, so it
had nothing under its feet and reversed direction every frame, vibrating in
place.

Placing an actor means placing its *feet*, and the code said its *top-left
corner*. Every author will make this mistake once per actor.

**Points at:** an `on_ground(column, row)` helper, which the game now has
locally. It belongs in `raster-2d`, and a future editor should snap to it.

## 5. Rooms are Rust, not scenes

`docs/decisions/018` expected rooms to be authored as scene files. They are
`Vec<String>` in Rust instead, because a scene describes *actors* and a room is
mostly *tiles*, and the two have no shared file format.

The scene format works — it is used for actors — but a room needs both halves in
one file to be editable as a unit.

**Points at:** M2.6's deferred question about nesting, and M6. A `.room` that
holds a tilemap and a scene is probably the answer.

## 6. Input has no "confirm"

`Action` offers `JUMP`, `ATTACK`, `INTERACT` and `PAUSE`. A menu needs "confirm"
and "cancel", which are not gameplay actions and not the same as jump.

The game binds `JUMP` and `INTERACT` to mean confirm, which works and reads
badly.

**Points at:** M4. UI navigation actions belong with the UI toolkit, not in the
gameplay action set.
