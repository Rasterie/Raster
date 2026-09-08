//! The 2D renderer for the Raster engine.
//!
//! Everything here is pixel-art first: integer scaling, nearest-neighbour
//! filtering, and sprites snapped to the pixel grid while the camera
//! interpolates in sub-pixels. Getting that last pair wrong is what produces
//! either jitter or blur, and it is the most common failure in 2D engines.
//!
//! At this milestone the renderer only opens a window and clears it. Sprite
//! batching arrives with M1.

#![forbid(unsafe_code)]

mod app;
mod gpu;

pub use app::{App, RunError, WindowConfig, run};
pub use gpu::{Frame, Gpu, GpuError};
pub use wgpu::Color;
