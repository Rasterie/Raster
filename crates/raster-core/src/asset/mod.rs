//! Assets: files a game loads, named by path and loaded once.

mod id;
mod project;
mod store;
mod watch;

pub use id::AssetId;
pub use project::{AssetError, Project};
pub use store::{AssetStore, Handle};
pub use watch::Watcher;
