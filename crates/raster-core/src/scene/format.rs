use crate::reflect::Value;
use raster_math::{IVec2, Rect, Vec2};
use std::collections::BTreeMap;

/// Turns a reflected value into TOML.
///
/// Les types du moteur n'existent pas en TOML : un `Vec2` devient `[x, y]`, un
/// `Rect` `[x, y, w, h]`. Le décodage les reconnaît à leur longueur.
#[must_use]
pub fn to_toml(value: &Value) -> toml::Value {
    match value {
        Value::Bool(v) => toml::Value::Boolean(*v),
        Value::Int(v) => toml::Value::Integer(*v),
        Value::Float(v) => toml::Value::Float(*v),
        Value::Str(v) => toml::Value::String(v.clone()),
        // Un asset s'ecrit comme son chemin : c'est ce que le champ contient.
        Value::Asset(v) => toml::Value::String(v.to_string()),

        Value::Vec2(v) => floats(&[v.x, v.y]),
        Value::IVec2(v) => toml::Value::Array(vec![
            toml::Value::Integer(i64::from(v.x)),
            toml::Value::Integer(i64::from(v.y)),
        ]),
        Value::Rect(v) => floats(&[v.position.x, v.position.y, v.size.x, v.size.y]),

        Value::Struct(fields) => toml::Value::Table(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), to_toml(value)))
                .collect(),
        ),
        Value::List(items) => toml::Value::Array(items.iter().map(to_toml).collect()),

        Value::Enum { variant, payload } => match payload {
            Some(p) => {
                let mut table = toml::map::Map::new();
                table.insert("variant".to_owned(), toml::Value::String(variant.clone()));
                table.insert("value".to_owned(), to_toml(p));
                toml::Value::Table(table)
            }
            None => toml::Value::String(variant.clone()),
        },

        // TOML n'a pas de valeur nulle : on omet la clé à l'écriture, et son
        // absence signifie `None` à la lecture.
        Value::None => toml::Value::Table(toml::map::Map::new()),
    }
}

/// Turns TOML back into a reflected value.
///
/// `hint` says what the field expects, which is how a `[1.0, 2.0]` becomes a
/// `Vec2` rather than a list of floats.
#[must_use]
pub fn from_toml(value: &toml::Value, hint: Option<&crate::reflect::ValueKind>) -> Value {
    use crate::reflect::ValueKind;

    match value {
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Integer(v) => Value::Int(*v),
        toml::Value::Float(v) => Value::Float(*v),
        toml::Value::String(v) => match hint {
            Some(ValueKind::Asset) => Value::Asset(crate::AssetId::new(v)),
            _ => Value::Str(v.clone()),
        },
        toml::Value::Datetime(v) => Value::Str(v.to_string()),

        toml::Value::Array(items) => match (hint, items.len()) {
            (Some(ValueKind::Vec2), 2) => {
                Value::Vec2(Vec2::new(number(&items[0]), number(&items[1])))
            }
            (Some(ValueKind::IVec2), 2) => {
                Value::IVec2(IVec2::new(integer(&items[0]), integer(&items[1])))
            }
            (Some(ValueKind::Rect), 4) => Value::Rect(Rect::new(
                number(&items[0]),
                number(&items[1]),
                number(&items[2]),
                number(&items[3]),
            )),
            _ => {
                let inner = match hint {
                    Some(ValueKind::List(k)) => Some(*k),
                    _ => None,
                };
                Value::List(items.iter().map(|i| from_toml(i, inner)).collect())
            }
        },

        toml::Value::Table(table) => Value::Struct(
            table
                .iter()
                .map(|(name, value)| (name.clone(), from_toml(value, None)))
                .collect(),
        ),
    }
}

/// Reads a table into a value, using a type's fields to guide the conversion.
///
/// Sans les indications de type, un `[1.0, 2.0]` resterait une liste et le
/// champ `Vec2` la refuserait.
#[must_use]
pub fn table_to_value(table: &toml::Table, info: &crate::reflect::TypeInfo) -> Value {
    let mut fields = BTreeMap::new();

    for (name, value) in table {
        let hint = info.field_by_serialized_name(name).map(|f| &f.kind);
        fields.insert(name.clone(), from_toml(value, hint));
    }

    Value::Struct(fields)
}

fn floats(values: &[f32]) -> toml::Value {
    toml::Value::Array(
        values
            .iter()
            .map(|v| toml::Value::Float(f64::from(*v)))
            .collect(),
    )
}

/// Lit un nombre, qu'il soit ecrit `1` ou `1.0` : un fichier ecrit a la main
/// utilise la forme la plus courte.
fn number(value: &toml::Value) -> f32 {
    match value {
        toml::Value::Float(v) => *v as f32,
        toml::Value::Integer(v) => *v as f32,
        _ => 0.0,
    }
}

fn integer(value: &toml::Value) -> i32 {
    match value {
        toml::Value::Integer(v) => i32::try_from(*v).unwrap_or(0),
        toml::Value::Float(v) => *v as i32,
        _ => 0,
    }
}
