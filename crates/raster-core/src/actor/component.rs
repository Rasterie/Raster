/// Un type qu'un acteur detient et qu'un sous-systeme doit atteindre sans
/// connaitre l'acteur.
///
/// La reflexion sait les *trouver*, mais en rend une `Value` allouee par acteur
/// et par frame. Ce trait est le chemin rapide : un emprunt direct.
pub trait Component: 'static {}

/// Emis par `#[derive(Reflect)]`, une fois par couple (acteur, composant).
///
/// Rend un iterateur : un acteur peut porter deux sprites.
pub trait HasComponent<C: Component> {
    /// Every `C` this actor holds, in field order.
    fn components(&self) -> impl Iterator<Item = &C>;

    /// The same, mutably.
    fn components_mut(&mut self) -> impl Iterator<Item = &mut C>;
}
