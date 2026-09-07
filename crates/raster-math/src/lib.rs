//! 2D math for the Raster engine.
//!
//! Every type here is `Copy` and free of dependencies. Two conventions hold
//! throughout and are worth stating once:
//!
//! - **Y grows downwards**, matching screen space. `Vec2::UP` is therefore
//!   `(0, -1)`, and rotation is clockwise.
//! - **Edges are half-open**: a rectangle includes its top-left edge and
//!   excludes its bottom-right, so tiling rectangles never both claim a point
//!   on a shared boundary.

#![forbid(unsafe_code)]
/*
  missing_docs reste desactive volontairement : il reclamerait un commentaire
  sur chaque `abs`, `min` et `to_array`, et la seule chose qu'on pourrait y
  ecrire paraphraserait le nom. Les methodes documentees ci-dessous sont celles
  dont le comportement ne se devine pas — convention d'axe, cas degeneres,
  arrondi.
*/

mod ivec2;
mod rect;
mod vec2;

pub use ivec2::IVec2;
pub use rect::Rect;
pub use vec2::Vec2;
