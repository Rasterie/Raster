//! The sprite editor.
//!
//! Le dessin manuel et la generation parametrique de Rasterie, sur le meme
//! canevas.

pub mod draw;
pub mod generate;
pub mod layers;
pub mod sheet;
pub mod symmetry;

pub use draw::{Stroke, pick};
pub use generate::{Light, Params, generate};
pub use layers::{Blend, Layer, Layers};
pub use sheet::{Layout, Onion, pack, unpack};
pub use symmetry::Symmetry;
