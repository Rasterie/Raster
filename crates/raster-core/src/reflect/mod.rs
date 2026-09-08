//! Runtime type inspection: listing a type's fields, reading and writing them
//! by name.
//!
//! Unglamorous, and the most load-bearing mechanism in the engine. Four
//! features reduce to it — the inspector, scene serialisation, scripting and
//! hot reload — which is why it exists before anything appears on screen.
//!
//! ```
//! use raster_core::reflect::{Reflect, Value};
//!
//! #[derive(Reflect, Default)]
//! struct Player {
//!     #[property(min = 0.0, max = 500.0)]
//!     speed: f32,
//!     name: String,
//! }
//!
//! let mut player = Player::default();
//! player.set_field("speed", Value::Float(90.0)).unwrap();
//! assert_eq!(player.get_field("speed"), Some(Value::Float(90.0)));
//!
//! // Bounds clamp rather than reject, so a slider dragged past the end stops.
//! player.set_field("speed", Value::Float(9000.0)).unwrap();
//! assert_eq!(player.speed, 500.0);
//! ```

mod info;
mod registry;
mod traits;
mod value;

pub use info::{FieldInfo, PropertyAttrs, TypeInfo, ValueKind};
pub use registry::{ReflectObject, TypeRegistry};
pub use traits::{Reflect, ReflectError, ReflectValue};
pub use value::Value;

pub use raster_derive::Reflect;
