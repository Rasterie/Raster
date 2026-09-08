use crate::reflect::Reflect;

/// A type that can live in a world.
///
/// An actor is an ordinary Rust struct: its components are its fields and its
/// behaviour is its methods. The bounds say what the engine needs of it —
/// reflection so tools can inspect it, `Default` so a scene can build one
/// before filling it in.
///
/// Implemented automatically for any type meeting those bounds; there is
/// nothing to write by hand.
pub trait Actor: Reflect + Default + Sized + 'static {
    /// The name used in scene files and in the inspector.
    ///
    /// Defaults to the type name from reflection, which is almost always what
    /// you want.
    #[must_use]
    fn type_name() -> &'static str {
        Self::type_info().name
    }
}

impl<T: Reflect + Default + Sized + 'static> Actor for T {}

/// The lifecycle hooks an actor may implement.
///
/// Every hook is optional and defaults to doing nothing, so an actor implements
/// only what it needs.
///
/// Deliberately absent: `on_render`. Actors do not draw — the renderer reads
/// `Sprite` components. Giving actors a draw callback would break batching and
/// tie the renderer to gameplay.
pub trait Behaviour: Actor {
    /// Once per frame, with the time since the last one.
    fn tick(&mut self, _dt: f32) {}

    /// At a fixed timestep, for anything coupled to physics.
    ///
    /// A platformer whose jump height depends on frame rate is the bug this
    /// prevents, and it is discovered far too late.
    fn fixed_tick(&mut self, _dt: f32) {}

    /// Added to the world, after its fields are initialised.
    fn on_spawn(&mut self) {}

    /// Removed from the world, before its storage is freed.
    fn on_despawn(&mut self) {}
}
