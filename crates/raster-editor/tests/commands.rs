use raster_core::reflect::{Reflect, TypeRegistry, Value};
use raster_editor::History;
use raster_editor::commands::{Despawn, Editing, MoveActors, SetField, Spawn};
use raster_math::Vec2;

#[derive(Reflect, Default, Debug, PartialEq)]
struct Prop {
    position: Vec2,
    name: String,
    solid: bool,
}

fn editing() -> Editing {
    let mut registry = TypeRegistry::new();
    registry.register::<Prop>();
    Editing::new(registry)
}

#[test]
fn spawning_adds_an_actor() {
    let mut editing = editing();
    let mut history = History::new();

    history.push(
        Box::new(Spawn::new("Prop", Vec2::new(32.0, 48.0))),
        &mut editing,
    );

    assert_eq!(editing.world.count::<Prop>(), 1);
    let (_, prop) = editing.world.iter::<Prop>().next().unwrap();
    assert_eq!(prop.position, Vec2::new(32.0, 48.0));
}

#[test]
fn undoing_a_spawn_removes_the_actor() {
    let mut editing = editing();
    let mut history = History::new();

    history.push(Box::new(Spawn::new("Prop", Vec2::ZERO)), &mut editing);
    history.undo(&mut editing);

    assert_eq!(editing.world.count::<Prop>(), 0);
}

#[test]
fn redoing_a_spawn_puts_it_back_where_it_was() {
    let mut editing = editing();
    let mut history = History::new();

    history.push(
        Box::new(Spawn::new("Prop", Vec2::new(64.0, 16.0))),
        &mut editing,
    );
    history.undo(&mut editing);
    history.redo(&mut editing);

    let (_, prop) = editing.world.iter::<Prop>().next().unwrap();
    assert_eq!(
        prop.position,
        Vec2::new(64.0, 16.0),
        "refaire doit rendre le meme"
    );
}

#[test]
fn spawning_an_unknown_type_changes_nothing() {
    let mut editing = editing();
    let mut history = History::new();

    history.push(Box::new(Spawn::new("Inexistant", Vec2::ZERO)), &mut editing);

    assert_eq!(editing.world.count::<Prop>(), 0);
    // La commande reste dans l'historique, mais annuler ne doit pas paniquer.
    assert!(history.undo(&mut editing));
}

#[test]
fn despawning_removes_and_undoing_brings_back_the_values() {
    let mut editing = editing();
    let mut history = History::new();

    let id = editing.world.spawn(Prop {
        position: Vec2::new(10.0, 20.0),
        name: "caisse".to_owned(),
        solid: true,
    });

    history.push(Box::new(Despawn::new(id)), &mut editing);
    assert_eq!(editing.world.count::<Prop>(), 0);

    history.undo(&mut editing);
    let (_, prop) = editing
        .world
        .iter::<Prop>()
        .next()
        .expect("l'acteur revient");

    // Sans les valeurs sauvegardees, annuler ne rendrait qu'un acteur vide.
    assert_eq!(prop.position, Vec2::new(10.0, 20.0));
    assert_eq!(prop.name, "caisse");
    assert!(prop.solid);
}

#[test]
fn moving_shifts_every_selected_actor() {
    let mut editing = editing();
    let mut history = History::new();

    let a = editing.world.spawn(Prop::default());
    let b = editing.world.spawn(Prop {
        position: Vec2::new(100.0, 0.0),
        ..Prop::default()
    });

    history.push(
        Box::new(MoveActors::new(vec![a, b], Vec2::new(16.0, 8.0))),
        &mut editing,
    );

    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().position,
        Vec2::new(16.0, 8.0)
    );
    assert_eq!(
        editing.world.get::<Prop>(b).unwrap().position,
        Vec2::new(116.0, 8.0)
    );
}

#[test]
fn undoing_a_move_puts_everything_back() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());

    history.push(
        Box::new(MoveActors::new(vec![a], Vec2::new(32.0, 32.0))),
        &mut editing,
    );
    history.undo(&mut editing);

    assert_eq!(editing.world.get::<Prop>(a).unwrap().position, Vec2::ZERO);
}

#[test]
fn a_drag_of_many_steps_is_one_undo_entry() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());

    // Ce qu'un glissement produit : un pas par frame.
    for _ in 0..10 {
        history.push(
            Box::new(MoveActors::new(vec![a], Vec2::new(2.0, 0.0))),
            &mut editing,
        );
    }

    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().position,
        Vec2::new(20.0, 0.0)
    );
    assert_eq!(history.depth(), 1, "dix pas font une seule entree");

    history.undo(&mut editing);
    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().position,
        Vec2::ZERO,
        "une annulation doit defaire tout le glissement"
    );
}

#[test]
fn moves_of_different_selections_do_not_merge() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());
    let b = editing.world.spawn(Prop::default());

    history.push(
        Box::new(MoveActors::new(vec![a], Vec2::new(4.0, 0.0))),
        &mut editing,
    );
    history.push(
        Box::new(MoveActors::new(vec![b], Vec2::new(4.0, 0.0))),
        &mut editing,
    );

    assert_eq!(history.depth(), 2, "deux selections font deux entrees");
}

#[test]
fn setting_a_field_records_what_was_there() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop {
        name: "avant".to_owned(),
        ..Prop::default()
    });

    history.push(
        Box::new(SetField::new(a, "name", Value::Str("apres".to_owned()))),
        &mut editing,
    );
    assert_eq!(editing.world.get::<Prop>(a).unwrap().name, "apres");

    history.undo(&mut editing);
    assert_eq!(editing.world.get::<Prop>(a).unwrap().name, "avant");
}

#[test]
fn dragging_a_slider_is_one_entry_keeping_the_original_value() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());

    for x in [1.0, 2.0, 3.0, 4.0] {
        history.push(
            Box::new(SetField::new(a, "position", Value::Vec2(Vec2::new(x, 0.0)))),
            &mut editing,
        );
    }

    assert_eq!(history.depth(), 1);
    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().position,
        Vec2::new(4.0, 0.0)
    );

    // Annuler revient a la valeur d'avant le glissement, pas a l'avant-derniere.
    history.undo(&mut editing);
    assert_eq!(editing.world.get::<Prop>(a).unwrap().position, Vec2::ZERO);
}

#[test]
fn changes_to_different_fields_do_not_merge() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());

    history.push(
        Box::new(SetField::new(a, "name", Value::Str("x".into()))),
        &mut editing,
    );
    history.push(
        Box::new(SetField::new(a, "solid", Value::Bool(true))),
        &mut editing,
    );

    assert_eq!(history.depth(), 2);
}

#[test]
fn a_command_on_a_missing_actor_does_nothing() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());
    editing.world.despawn(a);

    // Un acteur supprime entre-temps ne doit pas faire tomber l'editeur.
    history.push(
        Box::new(MoveActors::new(vec![a], Vec2::new(8.0, 8.0))),
        &mut editing,
    );
    history.push(
        Box::new(SetField::new(a, "name", Value::Str("x".into()))),
        &mut editing,
    );
    assert!(history.undo(&mut editing));
}

#[test]
fn labels_describe_what_happened() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop::default());

    history.push(Box::new(Spawn::new("Prop", Vec2::ZERO)), &mut editing);
    assert_eq!(history.undo_label().as_deref(), Some("Ajoute Prop"));

    history.break_merge();
    history.push(Box::new(MoveActors::new(vec![a], Vec2::ZERO)), &mut editing);
    assert_eq!(history.undo_label().as_deref(), Some("Deplace 1 acteur(s)"));
}

#[test]
fn applying_a_command_twice_keeps_the_first_value_it_saw() {
    use raster_editor::Command;

    let mut editing = editing();
    let a = editing.world.spawn(Prop {
        name: "origine".to_owned(),
        ..Prop::default()
    });

    // Hors historique, une commande peut etre appliquee deux fois de suite.
    // Elle doit retenir ce qu'elle a vu la premiere fois, sinon annuler
    // rendrait un etat intermediaire.
    let mut command = SetField::new(a, "name", Value::Str("premier".into()));
    command.apply(&mut editing);
    command.value = Value::Str("second".into());
    command.apply(&mut editing);
    command.revert(&mut editing);

    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().name,
        "origine",
        "la commande a retenu un etat intermediaire"
    );
}

#[test]
fn redoing_a_field_change_does_not_lose_the_original_value() {
    let mut editing = editing();
    let mut history = History::new();
    let a = editing.world.spawn(Prop {
        name: "origine".to_owned(),
        ..Prop::default()
    });

    history.push(
        Box::new(SetField::new(a, "name", Value::Str("modifie".into()))),
        &mut editing,
    );

    // Annuler puis refaire reapplique la commande : si elle relisait la
    // valeur courante a ce moment, elle retiendrait "modifie" comme origine,
    // et la seconde annulation ne rendrait plus rien.
    history.undo(&mut editing);
    history.redo(&mut editing);
    history.undo(&mut editing);

    assert_eq!(
        editing.world.get::<Prop>(a).unwrap().name,
        "origine",
        "la valeur d'origine a ete perdue en refaisant"
    );
}
