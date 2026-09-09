use std::fmt;

/// What kind of value a field holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    Bool,
    Int,
    Float,
    Str,
    Vec2,
    IVec2,
    Rect,
    /// Un chemin d'asset : une chaine, mais que l'inspecteur presente comme un
    /// selecteur de fichier plutot qu'un champ texte.
    Asset,
    /// A nested reflected struct, named so the registry can resolve it.
    Struct(&'static str),
    /// Une reference statique plutot qu'un `Box` : un `FieldInfo` est un
    /// `static`, ou l'allocation est impossible.
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
            Self::Asset => write!(f, "asset"),
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
    /// Bornes d'un champ numerique : l'inspecteur affiche un curseur, et les
    /// ecritures sont bornees plutot que refusees.
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Help text.
    pub tooltip: Option<&'static str>,
}

/// One reflected field.
pub struct FieldInfo {
    pub name: &'static str,
    /// Le nom dans les fichiers de scene : c'est lui qui permet de renommer un
    /// champ sans invalider les scenes existantes.
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

    /// Cherche un champ par son nom serialise.
    #[must_use]
    pub fn field_by_serialized_name(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.serialized_name == name)
    }
}
