# Keystone

A small platformer: three rooms, a key in each, a door at the end.

Built with [Raster](https://github.com/Rasterie/Raster) to prove the engine can
ship a game. No editor was used — the rooms are hand-written and the code is
Rust.

## Playing

| Key | Does |
| --- | --- |
| Arrows, WASD, ZQSD | Move |
| Space | Jump, and confirm on a menu |
| Escape | Pause, and quit from the title screen |

Take the key, reach the door, survive three rooms. Landing on an enemy from
above squashes it; walking into one costs a heart. Spikes cannot be squashed.

Progress is saved when a room is cleared, in `~/.keystone-save.toml`. Delete it
to start over.

## Running from source

```
cargo run --release -p keystone
```

## Known rough edges

The title screen and the menus are coloured bars: text rendering does not exist
yet, and arrives with the UI toolkit. See
[`docs/friction.md`](../../docs/friction.md) for the full list of what building
this game exposed.
