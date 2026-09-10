//! Rasterie: parametric pixel art.
//!
//! Porte du moteur TypeScript, dont la suite de tests fait foi pour la parite.

pub mod oklch;
pub mod ramp;

pub use oklch::{Oklch, Rgb, oklch_to_rgb, rgb_to_oklch};
pub use ramp::{RampOptions, generate};
