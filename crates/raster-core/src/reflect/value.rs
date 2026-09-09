use raster_math::{IVec2, Rect, Vec2};
use std::collections::BTreeMap;
use std::fmt;

/// A value read from or written to a reflected field.
///
/// La monnaie commune a l'inspecteur, la serialisation et le scripting.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Vec2(Vec2),
    IVec2(IVec2),
    Rect(Rect),
    /// A path to an asset file.
    Asset(crate::asset::AssetId),
    /// A nested struct, keyed by field name. Ordonne plutot que hache : deux
    /// serialisations d'une meme valeur doivent donner le meme fichier.
    Struct(BTreeMap<String, Value>),
    /// A sequence: `Vec<T>`, or a fixed-size array.
    List(Vec<Value>),
    /// An enum variant and its payload, `None` for a unit variant.
    Enum {
        variant: String,
        payload: Option<Box<Value>>,
    },
    /// An absent optional value.
    None,
}

impl Value {
    /// A short name for the kind of value held, for error messages.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Str(_) => "string",
            Self::Vec2(_) => "Vec2",
            Self::IVec2(_) => "IVec2",
            Self::Rect(_) => "Rect",
            Self::Asset(_) => "asset",
            Self::Struct(_) => "struct",
            Self::List(_) => "list",
            Self::Enum { .. } => "enum",
            Self::None => "none",
        }
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Accepte un entier comme un flottant : un fichier ecrit `speed = 90`.
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_vec2(&self) -> Option<Vec2> {
        match self {
            Self::Vec2(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_ivec2(&self) -> Option<IVec2> {
        match self {
            Self::IVec2(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_rect(&self) -> Option<Rect> {
        match self {
            Self::Rect(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_struct(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Self::Struct(v) => Some(v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    /// La valeur d'un champ imbrique, par chemin pointe : `body.velocity.x`.
    #[must_use]
    pub fn path(&self, path: &str) -> Option<&Value> {
        let mut current = self;
        for segment in path.split('.') {
            current = match current {
                Self::Struct(fields) => fields.get(segment)?,
                Self::List(items) => items.get(segment.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(current)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Str(v) => write!(f, "{v:?}"),
            Self::Vec2(v) => write!(f, "{v}"),
            Self::IVec2(v) => write!(f, "{v}"),
            Self::Rect(v) => write!(f, "{v}"),
            Self::Asset(v) => write!(f, "{v:?}"),
            Self::Struct(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {value}")?;
                }
                write!(f, "}}")
            }
            Self::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Enum { variant, payload } => match payload {
                Some(p) => write!(f, "{variant}({p})"),
                None => write!(f, "{variant}"),
            },
            Self::None => write!(f, "none"),
        }
    }
}
