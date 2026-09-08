use std::fmt;

/// What kind of value a field holds.
///
/// The inspector picks a widget from this, and the serialiser uses it to reject
/// a scene file that has drifted from the code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    Bool,
    Int,
    Float,
    Str,
    Vec2,
    IVec2,
    Rect,
    /// A nested reflected struct, named so the registry can resolve it.
    Struct(&'static str),
    /*
      Une reference statique plutot qu'un Box : un FieldInfo est un `static`,
      et une allocation est impossible dans un contexte const. Le type imbrique
      est lui-meme un static, donc la reference est toujours disponible.
    */
    List(&'static ValueKind),
    Enum(&'static str),
    Option(&'static ValueKind),
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Str => write!(f, "string"),
            Self::Vec2 => write!(f, "Vec2"),
            Self::IVec2 => write!(f, "IVec2"),
            Self::Rect => write!(f, "Rect"),
            Self::Struct(name) | Self::Enum(name) => write!(f, "{name}"),
            Self::List(inner) => write!(f, "[{inner}]"),
            Self::Option(inner) => write!(f, "{inner}?"),
        }
    }
}

/// How the inspector should present a field, and whether tools may touch it.
#[derive(Debug, Clone, Default)]
pub struct PropertyAttrs {
    /// Displayed but not editable.
    pub readonly: bool,
    /// Bounds for a numeric field. The inspector renders a slider, and writes
    /// are clamped rather than rejected.
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Help text.
    pub tooltip: Option<&'static str>,
}

/// One reflected field.
pub struct FieldInfo {
    pub name: &'static str,
    /// The name used in scene files, which may differ from the Rust field name.
    ///
    /// This is what lets a field be renamed in code without invalidating every
    /// scene that references it.
    pub serialized_name: &'static str,
    pub kind: ValueKind,
    pub attrs: PropertyAttrs,
}

impl fmt::Debug for FieldInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldInfo")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// The static description of a reflected type.
#[derive(Debug)]
pub struct TypeInfo {
    pub name: &'static str,
    pub fields: &'static [FieldInfo],
}

impl TypeInfo {
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Looks a field up by its serialised name, which is what a scene file
    /// carries.
    #[must_use]
    pub fn field_by_serialized_name(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.serialized_name == name)
    }
}
