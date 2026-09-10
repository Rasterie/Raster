//! What every Visual editor shares.
//!
//! Concu avant le deuxieme editeur : le troisieme forcerait sinon a reecrire
//! les deux premiers.

pub mod canvas;
pub mod frame;
pub mod palette;
pub mod tools;

pub use canvas::Canvas;
pub use frame::{Frame, Frames};
pub use palette::{Palette, Swatch};
pub use tools::Tool;
