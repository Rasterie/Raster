use crate::reflect::Reflect;

/// A type that can live in a world.
///
/// Une struct ordinaire : ses composants sont ses champs, son comportement ses
/// methodes. Implemente automatiquement, rien a ecrire a la main.
pub trait Actor: Reflect + Default + Sized + 'static {
    /// Le nom utilise dans les fichiers de scene et l'inspecteur.
    #[must_use]
    fn type_name() -> &'static str {
        Self::type_info().name
    }
}

impl<T: Reflect + Default + Sized + 'static> Actor for T {}

/// Les points d'entree du cycle de vie, tous optionnels.
///
/// Pas d'`on_render` : les acteurs ne dessinent pas, le renderer lit leurs
/// composants. Un rappel de dessin casserait le regroupement.
pub trait Behaviour: Actor {
    /// Once per frame, with the time since the last one.
    fn tick(&mut self, _dt: f32) {}

    /// A cadence fixe, pour ce qui est couple a la physique.
    fn fixed_tick(&mut self, _dt: f32) {}

    /// Added to the world, after its fields are initialised.
    fn on_spawn(&mut self) {}

    /// Removed from the world, before its storage is freed.
    fn on_despawn(&mut self) {}
}
