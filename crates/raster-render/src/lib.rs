//! The 2D renderer for the Raster engine.
//!
//! Everything here is pixel-art first: integer scaling, nearest-neighbour
//! filtering, and sprites snapped to the pixel grid while the camera
//! interpolates in sub-pixels. Getting that last pair wrong is what produces
//! either jitter or blur, and it is the most common failure in 2D engines.

#![forbid(unsafe_code)]

mod app;
mod batch;
mod camera;
mod gpu;
mod input_bridge;
mod png_load;
mod sprite;
mod target;
mod texture;

pub use app::{App, RunError, WindowConfig, run};
pub use batch::{SpriteBatch, Stats};
pub use camera::Camera;
pub use gpu::{Frame, Gpu, GpuError};
pub use png_load::{Image, ImageError};
pub use sprite::{Colour, Layer, SpriteDraw};
pub use target::RenderTarget;
pub use texture::Texture;
pub use wgpu::Color;
