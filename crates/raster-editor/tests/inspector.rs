use raster_core::asset::AssetId;
use raster_core::reflect::{Reflect, Value};
use raster_editor::inspector::{self, Editor};
use raster_math::{IVec2, Rect, Vec2};

/// Un type que l'inspecteur n'a jamais vu : c'est tout l'interet.
#[derive(Reflect, Default, Debug)]
struct Enemy {
    #[property(min = 0.0, max = 500.0, tooltip = "Pixels par seconde")]
    speed: f32,

    #[property(readonly)]
    id: u32,

    // Jamais lu : ce champ existe pour verifier que `skip` le cache.
    #[property(skip)]
    #[allow(dead_code)]
    cache: Vec<u32>,

    health: u32,
    alive: bool,
    name: String,
    position: Vec2,
    tile: IVec2,
    bounds: Rect,
    texture: AssetId,
    nickname: Option<String>,
}

#[test]
fn the_inspector_describes_a_type_it_has_never_seen() {
    let rows = inspector::rows(&Enemy::default());
    let noms: Vec<&str> = rows.iter().map(|r| r.label).collect();

    assert_eq!(
        noms,
        [
            "speed", "id", "health", "alive", "name", "position", "tile", "bounds", "texture",
            "nickname"
        ]
    );
    assert!(
        !noms.contains(&"cache"),
        "un champ ignore ne doit pas paraitre"
    );
}

#[test]
fn each_type_gets_the_editor_it_calls_for() {
    let rows = inspector::rows(&Enemy::default());
    let editeur = |nom: &str| {
        rows.iter()
            .find(|r| r.label == nom)
            .map(|r| r.editor.clone())
            .unwrap()
    };

    assert_eq!(editeur("alive"), Editor::Checkbox);
    assert_eq!(editeur("name"), Editor::Text);
    assert_eq!(editeur("position"), Editor::Vector2);
    assert_eq!(editeur("tile"), Editor::Vector2);
    assert_eq!(editeur("bounds"), Editor::Rectangle);
    assert_eq!(editeur("texture"), Editor::Asset);
    assert_eq!(editeur("health"), Editor::Number { speed: 1.0 });
}

#[test]
fn two_bounds_make_a_slider() {
    let rows = inspector::rows(&Enemy::default());
    let speed = rows.iter().find(|r| r.label == "speed").unwrap();

    assert_eq!(
        speed.editor,
        Editor::Slider {
            min: 0.0,
            max: 500.0
        },
        "un champ borne doit se regler au curseur"
    );
}

#[test]
fn a_readonly_field_is_shown_but_not_edited() {
    let rows = inspector::rows(&Enemy::default());
    let id = rows.iter().find(|r| r.label == "id").unwrap();

    assert_eq!(id.editor, Editor::ReadOnly);
}

#[test]
fn a_tooltip_reaches_the_row() {
    let rows = inspector::rows(&Enemy::default());
    let speed = rows.iter().find(|r| r.label == "speed").unwrap();

    assert_eq!(speed.tooltip, Some("Pixels par seconde"));
}

#[test]
fn rows_carry_the_current_values() {
    let enemy = Enemy {
        health: 42,
        alive: true,
        position: Vec2::new(10.0, 20.0),
        ..Enemy::default()
    };

    let rows = inspector::rows(&enemy);
    let valeur = |nom: &str| rows.iter().find(|r| r.label == nom).unwrap().value.clone();

    assert_eq!(valeur("health"), Value::Int(42));
    assert_eq!(valeur("alive"), Value::Bool(true));
    assert_eq!(valeur("position"), Value::Vec2(Vec2::new(10.0, 20.0)));
}

#[test]
fn writing_a_value_reaches_the_object() {
    let mut enemy = Enemy::default();

    inspector::set(&mut enemy, "health", Value::Int(7)).unwrap();
    assert_eq!(enemy.health, 7);

    inspector::set(&mut enemy, "position", Value::Vec2(Vec2::new(3.0, 4.0))).unwrap();
    assert_eq!(enemy.position, Vec2::new(3.0, 4.0));
}

#[test]
fn writing_a_readonly_field_is_refused() {
    let mut enemy = Enemy::default();

    // L'inspecteur affiche `id` mais ne doit pas l'ecrire.
    assert!(inspector::set(&mut enemy, "id", Value::Int(9)).is_err());
    assert_eq!(enemy.id, 0);
}

#[test]
fn writing_the_wrong_type_is_refused() {
    let mut enemy = Enemy::default();

    assert!(inspector::set(&mut enemy, "health", Value::Str("beaucoup".into())).is_err());
    assert_eq!(enemy.health, 0);
}

#[test]
fn writing_an_unknown_field_is_refused() {
    let mut enemy = Enemy::default();
    assert!(inspector::set(&mut enemy, "inexistant", Value::Int(1)).is_err());
}

#[test]
fn a_value_is_clamped_to_its_bounds() {
    let info = Enemy::type_info();
    let speed = info.field("speed").unwrap();

    // Un curseur tire au-dela doit s'arreter, pas refuser la modification.
    assert_eq!(inspector::clamp(speed, 9999.0), 500.0);
    assert_eq!(inspector::clamp(speed, -50.0), 0.0);
    assert_eq!(inspector::clamp(speed, 250.0), 250.0);
}

#[test]
fn an_unbounded_field_is_never_clamped() {
    let info = Enemy::type_info();
    let health = info.field("health").unwrap();

    assert_eq!(inspector::clamp(health, 1e9), 1e9);
    assert_eq!(inspector::clamp(health, -1e9), -1e9);
}

#[test]
fn a_bounded_float_drags_by_a_hundredth_of_its_range() {
    let rows = inspector::rows(&Enemy::default());
    let health = rows.iter().find(|r| r.label == "health").unwrap();

    // Un entier avance d'un ; un flottant borne avance plus finement.
    assert_eq!(health.editor, Editor::Number { speed: 1.0 });
}

#[test]
fn an_option_is_shown_without_being_edited() {
    let rows = inspector::rows(&Enemy::default());
    let nickname = rows.iter().find(|r| r.label == "nickname").unwrap();

    // Ce que l'inspecteur ne sait pas presenter, il l'affiche sans mentir.
    assert_eq!(nickname.editor, Editor::Unsupported);
}

/// Un objet dont un champ refuse de se lire : l'inspecteur doit continuer.
#[derive(Debug)]
struct Broken;

impl raster_core::reflect::ReflectObject for Broken {
    fn type_info(&self) -> &'static raster_core::reflect::TypeInfo {
        <Enemy as Reflect>::type_info()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn apply(&mut self, _: &Value) -> Result<(), raster_core::reflect::ReflectError> {
        Ok(())
    }
    fn get_field(&self, name: &str) -> Option<Value> {
        // Un seul champ se lit : les autres sont muets.
        (name == "health").then_some(Value::Int(3))
    }
    fn set_field(&mut self, _: &str, _: Value) -> Result<(), raster_core::reflect::ReflectError> {
        Ok(())
    }
    fn set_field_unchecked(
        &mut self,
        _: &str,
        _: Value,
    ) -> Result<(), raster_core::reflect::ReflectError> {
        Ok(())
    }
    fn set_field_by_serialized_name(
        &mut self,
        _: &str,
        _: Value,
    ) -> Result<(), raster_core::reflect::ReflectError> {
        Ok(())
    }
    fn to_value(&self) -> Value {
        Value::None
    }
}

#[test]
fn a_field_that_cannot_be_read_is_skipped_not_faked() {
    // Sauter est honnete ; afficher une valeur vide ferait croire que le champ
    // vaut zero, et l'ecrire l'ecraserait.
    let rows = inspector::rows(&Broken);

    assert_eq!(rows.len(), 1, "seul le champ lisible doit paraitre");
    assert_eq!(rows[0].label, "health");
    assert_eq!(rows[0].value, Value::Int(3));
}
