//! Actors: the entities that populate a world.
//!
//! An actor is a Rust type, its components are its fields, and its behaviour is
//! its methods. See `docs/architecture/actors.md` for the reasoning, and
//! `docs/decisions/013-actor-storage.md` for why they are stored in typed
//! pools.

mod id;
mod pool;
mod traits;

pub use id::ActorId;
pub use traits::{Actor, Behaviour};

pub(crate) use id::Generations;
pub(crate) use pool::Pool;
