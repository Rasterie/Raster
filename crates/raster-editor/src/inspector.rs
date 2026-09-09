use raster_core::reflect::{FieldInfo, ReflectObject, Value, ValueKind};

/// How a field should be presented.
///
/// Derive de la reflexion : l'inspecteur ne connait aucun type de jeu, il lit
/// ce que `#[derive(Reflect)]` a decrit.
#[derive(Debug, Clone, PartialEq)]
pub enum Editor {
    Checkbox,
    /// Un nombre a glissement, avec son pas.
    Number {
        speed: f32,
    },
    /// Un nombre borne : l'inspecteur montre un curseur.
    Slider {
        min: f32,
        max: f32,
    },
    Text,
    /// Un chemin d'asset, avec un selecteur de fichier.
    Asset,
    /// Deux nombres cote a cote.
    Vector2,
    /// Quatre nombres : position et taille.
    Rectangle,
    /// Un champ que l'inspecteur montre sans permettre de le changer.
    ReadOnly,
    /// Ce que l'inspecteur ne sait pas presenter : affiche, jamais edite.
    Unsupported,
}

/// One line of the inspector.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Le nom affiche : le nom Rust, pas celui de serialisation.
    pub label: &'static str,
    /// Le nom a utiliser pour ecrire la valeur.
    pub field: &'static str,
    pub editor: Editor,
    pub value: Value,
    pub tooltip: Option<&'static str>,
}

/// Builds the rows for an object, in declaration order.
///
/// Un champ dont la valeur ne peut pas etre lue est saute : un inspecteur ne
/// doit pas s'arreter parce qu'un champ est bizarre.
#[must_use]
pub fn rows(object: &dyn ReflectObject) -> Vec<Row> {
    object
        .type_info()
        .fields
        .iter()
        .filter_map(|field| {
            let value = object.get_field(field.name)?;
            Some(Row {
                label: field.name,
                field: field.name,
                editor: editor_for(field),
                value,
                tooltip: field.attrs.tooltip,
            })
        })
        .collect()
}

/// The editor a field's type and attributes call for.
#[must_use]
pub fn editor_for(field: &FieldInfo) -> Editor {
    if field.attrs.readonly {
        return Editor::ReadOnly;
    }

    match field.kind {
        ValueKind::Bool => Editor::Checkbox,
        ValueKind::Str => Editor::Text,
        ValueKind::Asset => Editor::Asset,
        ValueKind::Vec2 | ValueKind::IVec2 => Editor::Vector2,
        ValueKind::Rect => Editor::Rectangle,

        ValueKind::Int | ValueKind::Float => match (field.attrs.min, field.attrs.max) {
            // Deux bornes font un curseur ; une seule n'en fait pas un.
            (Some(min), Some(max)) => Editor::Slider {
                min: min as f32,
                max: max as f32,
            },
            _ => Editor::Number {
                speed: speed_for(field),
            },
        },

        _ => Editor::Unsupported,
    }
}

/// Le pas d'un glissement : un entier avance d'un, un flottant d'un centieme
/// de sa course quand elle est connue.
fn speed_for(field: &FieldInfo) -> f32 {
    if field.kind == ValueKind::Int {
        return 1.0;
    }

    match (field.attrs.min, field.attrs.max) {
        (Some(min), Some(max)) => ((max - min) / 100.0) as f32,
        _ => 0.1,
    }
}

/// Writes a value back, respecting the field's bounds.
///
/// # Errors
///
/// If the field does not exist, or the value has the wrong type.
pub fn set(
    object: &mut dyn ReflectObject,
    field: &str,
    value: Value,
) -> Result<(), raster_core::reflect::ReflectError> {
    object.set_field(field, value)
}

/// Clamps a number to a field's bounds.
///
/// L'inspecteur borne plutot que refuse : un curseur tire au-dela doit
/// s'arreter, pas rejeter la modification.
#[must_use]
pub fn clamp(field: &FieldInfo, value: f64) -> f64 {
    let low = field.attrs.min.unwrap_or(f64::NEG_INFINITY);
    let high = field.attrs.max.unwrap_or(f64::INFINITY);
    value.clamp(low, high.max(low))
}
