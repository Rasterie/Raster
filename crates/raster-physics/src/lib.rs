//! AABB collision against a tile grid.
//!
//! Pas de solveur de corps rigides : un jeu d'action 2D veut un personnage qui
//! s'arrete net, pas un qui conserve son elan. Voir `docs/non-goals.md`.

#![forbid(unsafe_code)]

mod body;
mod resolve;

pub use body::{Body, Contacts};
pub use resolve::{grounded, move_body, overlapping};
