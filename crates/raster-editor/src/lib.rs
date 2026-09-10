//! The editor shell: docking, history, the asset browser, the inspector.
//!
//! Le cadre qui tient les editeurs dedies, et rien de ce qu'ils font.

pub mod browser;
pub mod commands;
pub mod dock;
pub mod inspector;
pub mod keys;
pub mod play;
pub mod session;
pub mod undo;
pub mod viewport;

pub use browser::{Browser, Entry, Kind};
pub use commands::{Despawn, Editing, MoveActors, SetField, Spawn};
pub use dock::{Dock, Node, Placement};
pub use inspector::{Editor as FieldEditor, Row};
pub use keys::{Editor, Shortcuts};
pub use play::{Session as PlaySession, State as PlayState};
pub use session::Session;
pub use undo::{Command, History, Target, target_as};
pub use viewport::{Handle, Mode, Viewport};
