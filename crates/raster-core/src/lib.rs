//! Actors, the world, and reflection: the heart of the Raster engine.
//!
//! This crate describes *what exists* — not what it looks like or sounds like.
//! Rendering, audio, physics and input live in their own crates and build on
//! this one.

#![forbid(unsafe_code)]

pub mod actor;
pub mod asset;
pub mod reflect;
pub mod scene;
mod time;
mod world;

pub use actor::{Actor, ActorId, Behaviour};
pub use asset::{AssetError, AssetId, AssetStore, Handle, Project, Watcher};
pub use scene::{Scene, SceneError};
pub use time::{FrameLoop, Steps, Time};
pub use world::World;
