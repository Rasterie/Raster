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
    ///
    /// Silently truncating would corrupt data from a hand-edited scene file
    /// without telling anyone.
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

/// A type whose fields can be listed, read and written at runtime.
///
/// Implemented by `#[derive(Reflect)]` for structs, and by hand for the
/// primitives below. Four features depend on it: the inspector, scene
/// serialisation, scripting and hot reload.
pub trait Reflect: 'static {
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized;

    /// The same description, reachable through a trait object.
    fn type_info_dyn(&self) -> &'static TypeInfo;

    fn get_field(&self, name: &str) -> Option<Value>;

    fn set_field(&mut self, name: &str, value: Value) -> Result<(), ReflectError>;

    /// Writes a field, ignoring the readonly attribute.
    ///
    /// `readonly` means "the inspector must not offer this for editing", not
    /// "this value cannot be restored". Loading a scene has to write every
    /// serialised field, readonly ones included, or a saved value would be
    /// lost on the next load.
    fn set_field_unchecked(&mut self, name: &str, value: Value) -> Result<(), ReflectError>;

    /// Writes a field addressed by its *serialised* name, which is what a
    /// scene file carries and may differ from the Rust field name.
    ///
    /// Also ignores readonly, for the same reason as
    /// [`Reflect::set_field_unchecked`].
    fn set_field_by_serialized_name(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), ReflectError>;

    /// The whole value, for serialisation and for nesting inside a parent.
    fn to_value(&self) -> Value;

    /// Applies every field present in `value`, ignoring any it does not know.
    ///
    /// Tolerating unknown fields is deliberate: a scene written by a newer
    /// version of the game must still load in an older one, minus what it
    /// cannot understand. The alternative is refusing to open the file at all.
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

/// A leaf value that reflection stores directly rather than descending into.
///
/// Separate from [`Reflect`] because these have no fields: asking a `f32` to
/// list its fields is meaningless, and every derive would have to special-case
/// them otherwise.
pub trait ReflectValue: Sized + 'static {
    /// The kind of value this type stores.
    ///
    /// Const because a `FieldInfo` is a `static`, and its kind must therefore
    /// be built at compile time.
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

/// Integers all funnel through `i64`, with a range check on the way back.
///
/// A scene file could hold any integer; writing 300 into a `u8` must fail
/// loudly rather than wrap around to 44.
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

/// `i64` and `u64` cannot use the macro above: `i64::from(i64)` is not a
/// conversion, and `u64` does not convert into `i64` infallibly.
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
        // Beyond i64::MAX the value cannot be represented; saturating keeps the
        // file readable rather than emitting a negative number.
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

/// Floats reject NaN and infinity on the way in.
///
/// A NaN position propagates through every calculation it touches and is
/// painful to trace back to the scene file that introduced it.
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
