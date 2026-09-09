//! The UI toolkit: text, layout, widgets.
//!
//! Ce que le jeu du MVP a reclame en premier : de quoi ecrire "Appuyez sur
//! Entree" sans dependre d'un asset.

pub mod font;
pub mod text;

pub use font::{ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH, LINE_HEIGHT};
pub use text::{Align, Placed, layout, measure};
