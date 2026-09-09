//! The UI toolkit: text, layout, widgets.
//!
//! Ce que le jeu du MVP a reclame en premier : de quoi ecrire "Appuyez sur
//! Entree" sans dependre d'un asset.

pub mod font;
pub mod id;
pub mod layout;
pub mod paint;
pub mod text;
pub mod theme;
pub mod ui;
pub mod widgets;

pub use font::{ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH, LINE_HEIGHT};
pub use id::Id;
pub use layout::{Anchor, Axis, Size, stack};
pub use paint::Painter;
pub use text::{Align, Placed, layout, measure};
pub use theme::{State, Theme};
pub use ui::{Keys, Pointer, Response, Ui};
pub use widgets::{button, checkbox, label, panel, progress, slider};
