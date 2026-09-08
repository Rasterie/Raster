//! Input for the Raster engine.
//!
//! Gameplay asks about actions, never about keys: `input.pressed(&Action::JUMP)`
//! rather than checking Space. Rebinding and gamepad support then need no
//! change to gameplay code.
//!
//! Two things every action game reimplements are provided here instead, because
//! they are most of the difference between controls that feel responsive and
//! controls that feel broken: input buffering, and coyote time.

#![forbid(unsafe_code)]

mod action;
mod state;

pub use action::{Action, Axis, Binding, Bindings, GamepadButton, Key, MouseButton};
pub use state::{Grace, Input};
