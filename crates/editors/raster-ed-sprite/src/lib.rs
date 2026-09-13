//! The sprite editor.
//!
//! Le dessin manuel et la generation parametrique de Rasterie, sur le meme
//! canevas.

pub mod generate;
pub mod layers;

pub use generate::{Light, Params, generate};
pub use layers::{Blend, Layer, Layers};
