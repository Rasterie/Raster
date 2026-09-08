use super::{Reflect, TypeInfo, Value};
use std::collections::BTreeMap;

/// Everything the engine knows how to construct from a name.
///
/// Loading a scene means turning the string `"Player"` into an actual `Player`,
/// which needs a name-to-constructor map that only the game can populate — the
/// engine cannot know the game's types.
///
/// Registration is explicit rather than collected by the linker. Linker-section
/// tricks work until something is compiled as a static library or built for
/// WebAssembly, and then they silently register nothing. A missing type here
/// fails loudly at load time instead.
#[derive(Default)]
pub struct TypeRegistry {
    entries: BTreeMap<&'static str, Entry>,
}

struct Entry {
    info: &'static TypeInfo,
    /// Builds a default instance, which `apply` then fills in from the scene.
    construct: fn() -> Box<dyn ReflectObject>,
}

/// A reflected value behind a trait object.
///
/// [`Reflect`] cannot be made into one directly because `type_info()` has no
/// receiver; this trait is the object-safe half.
pub trait ReflectObject: 'static {
    fn type_info(&self) -> &'static TypeInfo;
    fn get_field(&self, name: &str) -> Option<Value>;
    fn set_field(&mut self, name: &str, value: Value) -> Result<(), super::ReflectError>;
    fn set_field_unchecked(&mut self, name: &str, value: Value) -> Result<(), super::ReflectError>;
    fn set_field_by_serialized_name(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), super::ReflectError>;
    fn to_value(&self) -> Value;
    fn apply(&mut self, value: &Value) -> Result<(), super::ReflectError>;
    /// Downcasting, so a caller can recover the concrete type it registered.
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: Reflect + Sized> ReflectObject for T {
    fn type_info(&self) -> &'static TypeInfo {
        self.type_info_dyn()
    }
    fn get_field(&self, name: &str) -> Option<Value> {
        Reflect::get_field(self, name)
    }
    fn set_field(&mut self, name: &str, value: Value) -> Result<(), super::ReflectError> {
        Reflect::set_field(self, name, value)
    }
    fn set_field_unchecked(&mut self, name: &str, value: Value) -> Result<(), super::ReflectError> {
        Reflect::set_field_unchecked(self, name, value)
    }
    fn set_field_by_serialized_name(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), super::ReflectError> {
        Reflect::set_field_by_serialized_name(self, name, value)
    }
    fn to_value(&self) -> Value {
        Reflect::to_value(self)
    }
    fn apply(&mut self, value: &Value) -> Result<(), super::ReflectError> {
        Reflect::apply(self, value)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl TypeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a type, replacing any earlier entry under the same name.
    pub fn register<T: Reflect + Default + Sized>(&mut self) {
        let info = T::type_info();
        self.entries.insert(
            info.name,
            Entry {
                info,
                construct: || Box::new(T::default()),
            },
        );
    }

    #[must_use]
    pub fn info(&self, name: &str) -> Option<&'static TypeInfo> {
        self.entries.get(name).map(|e| e.info)
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// A default instance of the named type, or `None` if it was never
    /// registered.
    #[must_use]
    pub fn construct(&self, name: &str) -> Option<Box<dyn ReflectObject>> {
        self.entries.get(name).map(|e| (e.construct)())
    }

    /// Constructs the named type and applies `value` to it — the operation a
    /// scene loader performs for every actor it reads.
    pub fn construct_from(
        &self,
        name: &str,
        value: &Value,
    ) -> Result<Box<dyn ReflectObject>, super::ReflectError> {
        let mut object = self
            .construct(name)
            .ok_or_else(|| super::ReflectError::UnknownField {
                type_name: "TypeRegistry",
                field: name.to_owned(),
            })?;
        object.apply(value)?;
        Ok(object)
    }

    /// Every registered type name, in a stable order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.entries.keys().copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
