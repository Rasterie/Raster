//! Keystone: the MVP game — a single-screen platformer.
//!
//! La logique vit ici, testable sans fenetre ; `main.rs` ne fait que la
//! brancher aux entrees, au rendu et au son.

pub mod game;
pub mod rooms;
pub mod rules;
pub mod save;
pub mod world;

pub use game::{Events, Foe, Game, Screen};
pub use rules::{Hit, Progress, Turn};
pub use world::{Enemy, Kind, Player, Room};
