use crate::undo::{Command, Target, target_as};
use raster_core::reflect::{TypeRegistry, Value};
use raster_core::{ActorId, World};
use raster_math::Vec2;

/// The scene being edited.
///
/// Le monde et le registre ensemble : une commande qui cree un acteur a besoin
/// des deux, et les separer obligerait a les passer partout.
pub struct Editing {
    pub world: World,
    pub registry: TypeRegistry,
}

impl Editing {
    #[must_use]
    pub fn new(registry: TypeRegistry) -> Self {
        Self {
            world: World::new(),
            registry,
        }
    }
}

impl Target for Editing {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Adds an actor of a named type.
///
/// Garde ce qui a ete cree : refaire doit remettre le meme acteur, avec les
/// memes valeurs, pas un acteur neuf.
#[derive(Debug)]
pub struct Spawn {
    pub type_name: String,
    pub fields: Value,
    /// L'acteur pose, une fois la commande appliquee.
    spawned: Option<ActorId>,
}

impl Spawn {
    #[must_use]
    pub fn new(type_name: impl Into<String>, at: Vec2) -> Self {
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("position".to_owned(), Value::Vec2(at));

        Self {
            type_name: type_name.into(),
            fields: Value::Struct(fields),
            spawned: None,
        }
    }

    /// The actor this created, once applied.
    #[must_use]
    pub fn spawned(&self) -> Option<ActorId> {
        self.spawned
    }
}

impl Command for Spawn {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn apply(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };
        self.spawned =
            editing
                .registry
                .spawn_into(&mut editing.world, &self.type_name, &self.fields);
    }

    fn revert(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };
        if let Some(id) = self.spawned.take() {
            editing.world.despawn(id);
        }
    }

    fn label(&self) -> String {
        format!("Ajoute {}", self.type_name)
    }
}

/// Removes an actor, keeping enough to put it back.
#[derive(Debug)]
pub struct Despawn {
    pub actor: ActorId,
    /// Le type et les valeurs de l'acteur retire : sans eux, annuler ne
    /// pourrait rendre qu'un acteur vide.
    saved: Option<(String, Value)>,
}

impl Despawn {
    #[must_use]
    pub fn new(actor: ActorId) -> Self {
        Self { actor, saved: None }
    }
}

impl Command for Despawn {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn apply(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };

        if let Some(object) = editing.world.reflect(self.actor) {
            self.saved = Some((object.type_info().name.to_owned(), object.to_value()));
        }
        editing.world.despawn(self.actor);
    }

    fn revert(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };
        let Some((type_name, fields)) = &self.saved else {
            return;
        };

        if let Some(id) = editing
            .registry
            .spawn_into(&mut editing.world, type_name, fields)
        {
            // L'identifiant change : le monde en attribue un neuf. Ce que la
            // selection doit rattraper apres une annulation.
            self.actor = id;
        }
    }

    fn label(&self) -> String {
        "Supprime".to_owned()
    }
}

/// Moves actors by an offset.
///
/// Fusionne avec la suivante : un glissement tient en une entree, quel que
/// soit le nombre de pixels parcourus.
#[derive(Debug)]
pub struct MoveActors {
    pub actors: Vec<ActorId>,
    pub delta: Vec2,
}

impl MoveActors {
    #[must_use]
    pub fn new(actors: Vec<ActorId>, delta: Vec2) -> Self {
        Self { actors, delta }
    }
}

impl Command for MoveActors {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn apply(&mut self, target: &mut dyn Target) {
        shift(target, &self.actors, self.delta);
    }

    fn revert(&mut self, target: &mut dyn Target) {
        shift(target, &self.actors, -self.delta);
    }

    fn label(&self) -> String {
        format!("Deplace {} acteur(s)", self.actors.len())
    }

    fn merge(&mut self, other: &dyn Command) -> bool {
        // Deux deplacements du meme ensemble s'additionnent.
        let Some(next) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        if next.actors != self.actors {
            return false;
        }
        self.delta += next.delta;
        true
    }
}

/// Changes one field of one actor.
#[derive(Debug)]
pub struct SetField {
    pub actor: ActorId,
    pub field: String,
    pub value: Value,
    /// Ce qu'il y avait avant, retenu a la premiere application.
    previous: Option<Value>,
}

impl SetField {
    #[must_use]
    pub fn new(actor: ActorId, field: impl Into<String>, value: Value) -> Self {
        Self {
            actor,
            field: field.into(),
            value,
            previous: None,
        }
    }
}

impl Command for SetField {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn apply(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };
        let Some(object) = editing.world.reflect_mut(self.actor) else {
            return;
        };

        if self.previous.is_none() {
            self.previous = object.get_field(&self.field);
        }
        let _ = object.set_field(&self.field, self.value.clone());
    }

    fn revert(&mut self, target: &mut dyn Target) {
        let Some(editing) = target_as::<Editing>(target) else {
            return;
        };
        let (Some(object), Some(before)) =
            (editing.world.reflect_mut(self.actor), self.previous.clone())
        else {
            return;
        };
        let _ = object.set_field(&self.field, before);
    }

    fn label(&self) -> String {
        format!("Change {}", self.field)
    }

    fn merge(&mut self, other: &dyn Command) -> bool {
        // Tirer un curseur ne fait qu'une entree : on garde la valeur d'avant
        // et la derniere valeur atteinte.
        let Some(next) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        if next.actor != self.actor || next.field != self.field {
            return false;
        }
        self.value = next.value.clone();
        true
    }
}

/// Applique un decalage a chaque acteur, par la reflexion.
fn shift(target: &mut dyn Target, actors: &[ActorId], delta: Vec2) {
    let Some(editing) = target_as::<Editing>(target) else {
        return;
    };

    for actor in actors {
        let Some(object) = editing.world.reflect_mut(*actor) else {
            continue;
        };
        let Some(Value::Vec2(at)) = object.get_field("position") else {
            continue;
        };
        let _ = object.set_field("position", Value::Vec2(at + delta));
    }
}
