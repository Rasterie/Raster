//! Scenes: a list of actor instances, in a readable file.
//!
//! Un fichier de scene ne definit pas ce qu'est un acteur — cela vit dans le
//! code. Il liste des instances et les valeurs qui different du type.

mod format;

pub use format::{from_toml, table_to_value, to_toml};

use crate::reflect::{Reflect, TypeRegistry, Value};
use crate::{ActorId, World};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// Why a scene could not be read or written.
#[derive(Debug)]
pub enum SceneError {
    Io(std::io::Error),
    /// The file is not valid TOML.
    Parse(toml::de::Error),
    /// The file is valid TOML but not a scene.
    Malformed(String),
    /// An actor type the registry does not know.
    UnknownType(String),
    /// A field could not be written.
    Field(crate::reflect::ReflectError),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read the scene: {e}"),
            Self::Parse(e) => write!(f, "the scene is not valid TOML: {e}"),
            Self::Malformed(what) => write!(f, "the scene is malformed: {what}"),
            Self::UnknownType(name) => write!(
                f,
                "unknown actor type `{name}`; register it before loading the scene"
            ),
            Self::Field(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SceneError {}

impl From<std::io::Error> for SceneError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for SceneError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

/// One actor instance in a scene.
#[derive(Debug, Clone)]
pub struct Instance {
    /// The reflected type name, resolved through the registry.
    pub type_name: String,
    /// Les champs qui different des valeurs par defaut du type : un fichier de
    /// scene reste ainsi lisible et ne change que quand quelque chose change.
    pub fields: BTreeMap<String, Value>,
}

/// A list of actor instances, loadable into a world.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub name: String,
    pub instances: Vec<Instance>,
}

impl Scene {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instances: Vec::new(),
        }
    }

    /// Adds an actor, keeping only the fields that differ from its defaults.
    pub fn add<T: Reflect + Default>(&mut self, actor: &T) {
        let defaults = T::default().to_value();
        let current = actor.to_value();

        let fields = match (&current, &defaults) {
            (Value::Struct(now), Value::Struct(base)) => now
                .iter()
                .filter(|(name, value)| base.get(*name) != Some(*value))
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
            _ => BTreeMap::new(),
        };

        self.instances.push(Instance {
            type_name: T::type_info().name.to_owned(),
            fields,
        });
    }

    /// Reads a scene from disk.
    ///
    /// # Errors
    ///
    /// Fails if the file cannot be read or is not a valid scene.
    pub fn load(path: impl AsRef<Path>, registry: &TypeRegistry) -> Result<Self, SceneError> {
        let text = std::fs::read_to_string(path)?;
        Self::parse(&text, registry)
    }

    /// Reads a scene from text.
    ///
    /// # Errors
    ///
    /// As [`Scene::load`], minus the file access.
    pub fn parse(text: &str, registry: &TypeRegistry) -> Result<Self, SceneError> {
        let document: toml::Table = text.parse()?;

        let name = document
            .get("scene")
            .and_then(|s| s.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_owned();

        let mut instances = Vec::new();

        if let Some(actors) = document.get("actor") {
            let actors = actors
                .as_array()
                .ok_or_else(|| SceneError::Malformed("`actor` should be a list".to_owned()))?;

            for entry in actors {
                let table = entry.as_table().ok_or_else(|| {
                    SceneError::Malformed("an actor should be a table".to_owned())
                })?;

                let type_name = table
                    .get("type")
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| SceneError::Malformed("an actor needs a `type`".to_owned()))?
                    .to_owned();

                let info = registry
                    .info(&type_name)
                    .ok_or_else(|| SceneError::UnknownType(type_name.clone()))?;

                let mut fields = BTreeMap::new();
                for (key, value) in table {
                    if key == "type" {
                        continue;
                    }
                    let hint = info.field_by_serialized_name(key).map(|f| &f.kind);
                    fields.insert(key.clone(), from_toml(value, hint));
                }

                instances.push(Instance { type_name, fields });
            }
        }

        Ok(Self { name, instances })
    }

    /// Writes the scene as TOML.
    #[must_use]
    pub fn to_toml_string(&self) -> String {
        let mut out = String::new();

        out.push_str("[scene]\n");
        out.push_str(&format!("name = {:?}\n", self.name));

        for instance in &self.instances {
            out.push_str("\n[[actor]]\n");
            out.push_str(&format!("type = {:?}\n", instance.type_name));

            for (name, value) in &instance.fields {
                // Une valeur absente s'ecrit en omettant la cle.
                if matches!(value, Value::None) {
                    continue;
                }
                out.push_str(&format!("{name} = {}\n", to_toml(value)));
            }
        }

        out
    }

    /// Writes the scene to disk.
    ///
    /// # Errors
    ///
    /// Fails if the file cannot be written.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), SceneError> {
        std::fs::write(path, self.to_toml_string())?;
        Ok(())
    }

    /// Spawns every instance into a world.
    ///
    /// # Errors
    ///
    /// Fails on an unknown type or a field that cannot be written.
    pub fn spawn_into(
        &self,
        world: &mut World,
        registry: &TypeRegistry,
    ) -> Result<Vec<ActorId>, SceneError> {
        let mut spawned = Vec::with_capacity(self.instances.len());

        for instance in &self.instances {
            let value = Value::Struct(instance.fields.clone());

            let id = registry
                .spawn_into(world, &instance.type_name, &value)
                .ok_or_else(|| SceneError::UnknownType(instance.type_name.clone()))?;

            spawned.push(id);
        }

        Ok(spawned)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}
