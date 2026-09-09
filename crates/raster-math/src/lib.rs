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
// missing_docs reste desactive : il reclamerait un commentaire sur chaque
// `abs` et `min`, qui ne ferait que paraphraser le nom.

mod angle;
mod easing;
mod irect;
mod ivec2;
mod mat3;
mod rect;
mod rng;
mod transform;
mod vec2;

pub use angle::{
    angle_delta, degrees_to_radians, lerp_angle, radians_to_degrees, rotate_towards, wrap_angle,
};
pub use easing::{Easing, inverse_lerp, lerp, move_towards, remap, smoothstep};
pub use irect::IRect;
pub use ivec2::IVec2;
pub use mat3::Mat3;
pub use rect::Rect;
pub use rng::Rng;
pub use transform::Transform2D;
pub use vec2::Vec2;
