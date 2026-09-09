//! The 2D content types that sit between the engine's core and its renderer:
//! tilemaps, and later animation and cameras.
//!
//! Tiles are emphatically not actors. A world may hold millions of them, stored
//! as compact arrays in chunks; only a tile with behaviour — a chest, a machine
//! — gets an actor alongside it.

#![forbid(unsafe_code)]

mod autotile;
mod chunk;
mod tile;
mod tilemap;

pub use autotile::{Neighbours, neighbours_of, should_autotile};
pub use chunk::Chunk;
pub use tile::{Collision, TileId, TileKind, Tileset};
pub use tilemap::Tilemap;
