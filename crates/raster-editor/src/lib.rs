//! The editor shell: docking, history, the asset browser, the inspector.
//!
//! Le cadre qui tient les editeurs dedies, et rien de ce qu'ils font.

pub mod dock;
pub mod undo;

pub use dock::{Dock, Node, Placement};
pub use undo::{Command, History, Target, target_as};
