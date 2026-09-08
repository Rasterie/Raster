//! Derive macros for the Raster engine.
//!
//! This crate exists because Rust requires procedural macros to live in their
//! own `proc-macro` crate. Everything here is re-exported from `raster-core`,
//! which is where it should be used from.

mod reflect;

use proc_macro::TokenStream;

/// Generates a [`Reflect`] implementation: a static `TypeInfo` describing every
/// field, plus dynamic get and set.
///
/// ```ignore
/// #[derive(Reflect, Default)]
/// struct Player {
///     #[property(min = 0.0, max = 500.0, tooltip = "Pixels per second")]
///     speed: f32,
///
///     #[property(rename = "hp")]
///     health: u32,
///
///     #[property(skip)]
///     cache: Vec<u32>,
/// }
/// ```
///
/// Supported attributes: `skip`, `readonly`, `min`, `max`, `rename`, `tooltip`.
#[proc_macro_derive(Reflect, attributes(property))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    reflect::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
