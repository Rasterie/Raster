/// A type an actor holds as a field, and that a subsystem needs to reach
/// without knowing the actor.
///
/// The renderer must draw every `Sprite` without knowing that `Player` exists.
/// Reflection can *find* those fields — `ValueKind::Struct("Sprite")` says
/// which types hold one — but reading one back yields an owned `Value`, an
/// allocated map per actor per frame. That is fine for an inspector and far too
/// slow for a render loop.
///
/// This trait is the fast path: a direct borrow, no allocation, resolved at
/// compile time.
pub trait Component: 'static {}

/// Implemented by `#[derive(Reflect)]` for every field whose type is a
/// [`Component`], once per (actor, component type) pair.
///
/// An actor holding two sprites implements this once and returns both, which is
/// why the methods yield slices rather than a single reference.
pub trait HasComponent<C: Component> {
    /// Every `C` this actor holds, in field order.
    fn components(&self) -> impl Iterator<Item = &C>;

    /// The same, mutably.
    fn components_mut(&mut self) -> impl Iterator<Item = &mut C>;
}
