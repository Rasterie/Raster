//! Actors, the world, and reflection: the heart of the Raster engine.
//!
//! This crate describes *what exists* — not what it looks like or sounds like.
//! Rendering, audio, physics and input live in their own crates and build on
//! this one.

#![forbid(unsafe_code)]

pub mod reflect;
