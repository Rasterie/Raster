use super::{TypeInfo, Value, ValueKind};
use raster_math::{IVec2, Rect, Vec2};
use std::fmt;

/// Why a reflected write was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReflectError {
    /// No field by that name. Carries the type and the name that was asked for.
    UnknownField {
        type_name: &'static str,
        field: String,
    },
    /// The value did not match the field's type.
    TypeMismatch {
        field: String,
        expected: String,
        got: &'static str,
    },
    /// The field is marked readonly and cannot be written through
    /// [`Reflect::set_field`].
    Readonly { field: String },
    /// An integer would not fit the field's type, or a float was not finite.
    OutOfRange { field: String, detail: String },
}

impl fmt::Display for ReflectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownField { type_name, field } => {
                write!(f, "{type_name} has no field `{field}`")
            }
            Self::TypeMismatch {
                field,
                expected,
                got,
            } => {
                write!(f, "field `{field}` expects {expected}, got {got}")
            }
            Self::Readonly { field } => write!(f, "field `{field}` is read-only"),
            Self::OutOfRange { field, detail } => {
                write!(f, "value out of range for field `{field}`: {detail}")
            }
        }
    }
}

impl std::error::Error for ReflectError {}

/// A type whose fields can be listed, read and written at runtime. Emis par
/// `#[derive(Reflect)]`.
pub trait Reflect: 'static {
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized;

    /// The same description, reachable through a trait object.
    fn type_info_dyn(&self) -> &'static TypeInfo;

    fn get_field(&self, name: &str) -> Option<Value>;

    fn set_field(&mut self, name: &str, value: Value) -> Result<(), ReflectError>;

    /// Ecrit un champ en ignorant `readonly`, qui interdit l'edition mais pas
    /// le rechargement d'une scene.
    fn set_field_unchecked(&mut self, name: &str, value: Value) -> Result<(), ReflectError>;

    /// Writes a field by its *serialised* name — celui que porte un fichier de
    /// scene, qui peut differer du nom Rust. Ignore aussi readonly.
    fn set_field_by_serialized_name(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), ReflectError>;

    /// The whole value, for serialisation and for nesting inside a parent.
    fn to_value(&self) -> Value;

    /// Applique les champs presents, ignorant les inconnus : une scene plus
    /// recente reste chargeable.
    fn apply(&mut self, value: &Value) -> Result<(), ReflectError> {
        let Value::Struct(fields) = value else {
            return Err(ReflectError::TypeMismatch {
                field: "<root>".into(),
                expected: "struct".into(),
                got: value.kind_name(),
            });
        };

        for (name, field_value) in fields {
            match self.set_field_by_serialized_name(name, field_value.clone()) {
                Ok(()) | Err(ReflectError::UnknownField { .. }) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// A leaf value reflection stores directly rather than descending into.
///
/// Distinct de [`Reflect`] : demander ses champs a un `f32` n'a pas de sens.
pub trait ReflectValue: Sized + 'static {
    /// The kind of value this type stores. Const, car un `FieldInfo` est un
    /// `static`.
    const KIND: ValueKind;

    fn value_kind() -> ValueKind {
        Self::KIND
    }
    fn to_reflect_value(&self) -> Value;
    /// Reads the value back, or reports why it could not.
    fn from_reflect_value(value: &Value) -> Result<Self, String>;
}

macro_rules! impl_reflect_value {
    ($ty:ty, $kind:expr, $to:expr, $from:expr) => {
        impl ReflectValue for $ty {
            const KIND: ValueKind = $kind;
            fn to_reflect_value(&self) -> Value {
                #[allow(clippy::redundant_closure_call)]
                ($to)(self)
            }
            fn from_reflect_value(value: &Value) -> Result<Self, String> {
                #[allow(clippy::redundant_closure_call)]
                ($from)(value)
            }
        }
    };
}

impl_reflect_value!(
    bool,
    ValueKind::Bool,
    |v: &bool| Value::Bool(*v),
    |v: &Value| v
        .as_bool()
        .ok_or_else(|| format!("expected bool, got {}", v.kind_name()))
);

impl_reflect_value!(
    String,
    ValueKind::Str,
    |v: &String| Value::Str(v.clone()),
    |v: &Value| v
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("expected string, got {}", v.kind_name()))
);

impl_reflect_value!(
    crate::asset::AssetId,
    ValueKind::Asset,
    |v: &crate::asset::AssetId| Value::Asset(v.clone()),
    // Une chaine nue est acceptee : un fichier ecrit a la main n'a pas a savoir
    // qu'un champ est un asset plutot qu'un texte.
    |v: &Value| match v {
        Value::Asset(id) => Ok(id.clone()),
        Value::Str(path) => Ok(crate::asset::AssetId::new(path)),
        other => Err(format!("expected an asset path, got {}", other.kind_name())),
    }
);

impl_reflect_value!(
    Vec2,
    ValueKind::Vec2,
    |v: &Vec2| Value::Vec2(*v),
    |v: &Value| v
        .as_vec2()
        .ok_or_else(|| format!("expected Vec2, got {}", v.kind_name()))
);

impl_reflect_value!(
    IVec2,
    ValueKind::IVec2,
    |v: &IVec2| Value::IVec2(*v),
    |v: &Value| v
        .as_ivec2()
        .ok_or_else(|| format!("expected IVec2, got {}", v.kind_name()))
);

impl_reflect_value!(
    Rect,
    ValueKind::Rect,
    |v: &Rect| Value::Rect(*v),
    |v: &Value| v
        .as_rect()
        .ok_or_else(|| format!("expected Rect, got {}", v.kind_name()))
);

/// Les entiers passent tous par `i64`, avec un controle au retour : ecrire 300
/// dans un `u8` doit echouer plutot que devenir 44.
macro_rules! impl_reflect_int {
    ($($ty:ty),*) => {$(
        impl ReflectValue for $ty {
            const KIND: ValueKind = ValueKind::Int;
            fn to_reflect_value(&self) -> Value {
                Value::Int(i64::from(*self))
            }
            fn from_reflect_value(value: &Value) -> Result<Self, String> {
                let raw = value
                    .as_int()
                    .ok_or_else(|| format!("expected int, got {}", value.kind_name()))?;
                Self::try_from(raw).map_err(|_| {
                    format!("{raw} does not fit in {}", stringify!($ty))
                })
            }
        }
    )*};
}

impl_reflect_int!(i8, i16, i32, u8, u16, u32);

/// `i64` et `u64` ne passent pas par la macro : leurs conversions different.
impl ReflectValue for i64 {
    const KIND: ValueKind = ValueKind::Int;
    fn to_reflect_value(&self) -> Value {
        Value::Int(*self)
    }
    fn from_reflect_value(value: &Value) -> Result<Self, String> {
        value
            .as_int()
            .ok_or_else(|| format!("expected int, got {}", value.kind_name()))
    }
}

impl ReflectValue for u64 {
    const KIND: ValueKind = ValueKind::Int;
    fn to_reflect_value(&self) -> Value {
        // Sature plutot que d'emettre un negatif au-dela de i64::MAX.
        Value::Int(i64::try_from(*self).unwrap_or(i64::MAX))
    }
    fn from_reflect_value(value: &Value) -> Result<Self, String> {
        let raw = value
            .as_int()
            .ok_or_else(|| format!("expected int, got {}", value.kind_name()))?;
        Self::try_from(raw).map_err(|_| format!("{raw} does not fit in u64"))
    }
}

impl ReflectValue for usize {
    const KIND: ValueKind = ValueKind::Int;
    fn to_reflect_value(&self) -> Value {
        Value::Int(i64::try_from(*self).unwrap_or(i64::MAX))
    }
    fn from_reflect_value(value: &Value) -> Result<Self, String> {
        let raw = value
            .as_int()
            .ok_or_else(|| format!("expected int, got {}", value.kind_name()))?;
        Self::try_from(raw).map_err(|_| format!("{raw} does not fit in usize"))
    }
}

/// Les flottants refusent NaN et l'infini : un NaN contamine tout calcul qu'il
/// touche et remonte difficilement au fichier fautif.
macro_rules! impl_reflect_float {
    ($($ty:ty),*) => {$(
        impl ReflectValue for $ty {
            const KIND: ValueKind = ValueKind::Float;
            fn to_reflect_value(&self) -> Value {
                Value::Float(f64::from(*self))
            }
            fn from_reflect_value(value: &Value) -> Result<Self, String> {
                let raw = value
                    .as_float()
                    .ok_or_else(|| format!("expected float, got {}", value.kind_name()))?;
                if !raw.is_finite() {
                    return Err(format!("{raw} is not a finite number"));
                }
                Ok(raw as Self)
            }
        }
    )*};
}

impl_reflect_float!(f32, f64);

impl<T: ReflectValue> ReflectValue for Vec<T> {
    const KIND: ValueKind = ValueKind::List(&T::KIND);

    fn to_reflect_value(&self) -> Value {
        Value::List(self.iter().map(T::to_reflect_value).collect())
    }

    fn from_reflect_value(value: &Value) -> Result<Self, String> {
        let items = value
            .as_list()
            .ok_or_else(|| format!("expected list, got {}", value.kind_name()))?;
        items
            .iter()
            .enumerate()
            .map(|(i, item)| T::from_reflect_value(item).map_err(|e| format!("at index {i}: {e}")))
            .collect()
    }
}

impl<T: ReflectValue> ReflectValue for Option<T> {
    const KIND: ValueKind = ValueKind::Option(&T::KIND);

    fn to_reflect_value(&self) -> Value {
        match self {
            Some(v) => v.to_reflect_value(),
            None => Value::None,
        }
    }

    fn from_reflect_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::None => Ok(None),
            other => T::from_reflect_value(other).map(Some),
        }
    }
}
