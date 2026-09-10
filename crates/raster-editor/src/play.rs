use crate::commands::Editing;
use raster_core::reflect::{TypeRegistry, Value};
use raster_core::{ActorId, Scene, World};

/// Whether the editor is editing or running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Editing,
    Playing,
    /// En pause : le monde est fige mais la partie n'est pas terminee.
    Paused,
}

impl State {
    #[must_use]
    pub fn running(self) -> bool {
        self == Self::Playing
    }

    #[must_use]
    pub fn in_session(self) -> bool {
        self != Self::Editing
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Editing => "edition",
            Self::Playing => "en cours",
            Self::Paused => "en pause",
        }
    }
}

/// A play session: the world as it was, and the time spent running.
///
/// Ce qu'une execution jetable demande : de quoi remettre le monde exactement
/// comme il etait, sans qu'aucun changement du jeu ne puisse etre enregistre
/// par accident.
pub struct Session {
    pub state: State,
    /// Le monde d'avant, capture par la reflexion.
    snapshot: Option<Snapshot>,
    /// Le temps ecoule depuis le debut de la partie.
    pub elapsed: f32,
    /// Combien de pas fixes ont ete joues.
    pub steps: u64,
}

/// Le contenu d'un monde, sous une forme qui survit a son remplacement.
struct Snapshot {
    actors: Vec<(String, Value)>,
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: State::Editing,
            snapshot: None,
            elapsed: 0.0,
            steps: 0,
        }
    }

    /// Starts playing, keeping the world as it is now.
    ///
    /// Sans effet si une partie est deja en cours : relancer perdrait la
    /// capture d'origine, et l'arret ne rendrait plus le monde edite.
    pub fn start(&mut self, editing: &Editing) {
        if self.state.in_session() {
            return;
        }

        self.snapshot = Some(capture(&editing.world));
        self.state = State::Playing;
        self.elapsed = 0.0;
        self.steps = 0;
    }

    /// Stops, putting the edited world back exactly as it was.
    ///
    /// Renvoie le nombre d'acteurs restaures, ou `None` si rien ne tournait.
    pub fn stop(&mut self, editing: &mut Editing) -> Option<usize> {
        let snapshot = self.snapshot.take()?;
        self.state = State::Editing;

        editing.world.clear();
        let restored = restore(&snapshot, &mut editing.world, &editing.registry);
        Some(restored)
    }

    /// Pauses, or resumes.
    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            State::Playing => State::Paused,
            State::Paused => State::Playing,
            State::Editing => State::Editing,
        };
    }

    /// Advances one frame, if a session is running.
    ///
    /// Renvoie `true` si le monde a avance : l'appelant sait alors qu'il doit
    /// redessiner.
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.state != State::Playing {
            return false;
        }
        self.elapsed += dt;
        self.steps += 1;
        true
    }

    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    /// Whether stopping would restore something.
    #[must_use]
    pub fn has_snapshot(&self) -> bool {
        self.snapshot.is_some()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads a world into a form that survives it being cleared.
fn capture(world: &World) -> Snapshot {
    let mut actors = Vec::new();

    for id in ids(world) {
        if let Some(object) = world.reflect(id) {
            actors.push((object.type_info().name.to_owned(), object.to_value()));
        }
    }

    Snapshot { actors }
}

/// Puts a snapshot back, returning how many actors were restored.
fn restore(snapshot: &Snapshot, world: &mut World, registry: &TypeRegistry) -> usize {
    snapshot
        .actors
        .iter()
        .filter(|(name, fields)| registry.spawn_into(world, name, fields).is_some())
        .count()
}

/// Les identifiants vivants d'un monde.
///
/// `World` n'expose pas d'iteration sans type : passer par une scene serait
/// plus long, et la capture a besoin de chaque acteur, quel qu'il soit.
fn ids(world: &World) -> Vec<ActorId> {
    world.actor_ids()
}

/// Captures a world as a scene, for saving rather than for playing.
#[must_use]
pub fn to_scene(world: &World, name: &str) -> Scene {
    let mut scene = Scene::new(name);

    for id in ids(world) {
        if let Some(object) = world.reflect(id) {
            scene.add_erased(object);
        }
    }

    scene
}
