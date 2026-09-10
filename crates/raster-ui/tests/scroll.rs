use raster_math::{Rect, Vec2};
use raster_ui::{Drag, Id, Keys, Pointer, Theme, Ui};

const AREA: Rect = Rect {
    position: Vec2 { x: 0.0, y: 0.0 },
    size: Vec2 { x: 100.0, y: 60.0 },
};

fn ui() -> Ui {
    Ui::new(Theme::dark())
}

/// Pose le survol : il se resout en fin de frame, donc deux passes.
fn hover(ui: &mut Ui, id: Id, area: Rect, wheel: f32) {
    let centre = Vec2::new(
        area.position.x + area.size.x / 2.0,
        area.position.y + area.size.y / 2.0,
    );
    for _ in 0..2 {
        ui.begin(
            Pointer {
                at: centre,
                wheel,
                ..Pointer::default()
            },
            Keys::default(),
        );
        ui.interact(id, area, true);
        ui.end();
    }
}

#[test]
fn the_wheel_only_reaches_the_hovered_widget() {
    let mut ui = ui();
    let dessus = Id::new("dessus");
    let ailleurs = Id::new("ailleurs");

    hover(&mut ui, dessus, AREA, 3.0);

    assert_eq!(ui.wheel_over(dessus), 3.0);
    assert_eq!(
        ui.wheel_over(ailleurs),
        0.0,
        "la molette ne doit pas atteindre un widget non survole"
    );
}

#[test]
fn the_wheel_is_zero_without_hover() {
    let mut ui = ui();
    let id = Id::new("liste");

    ui.begin(
        Pointer {
            at: Vec2::new(500.0, 500.0),
            wheel: 5.0,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.interact(id, AREA, true);
    ui.end();

    assert_eq!(ui.wheel_over(id), 0.0);
}

#[test]
fn a_drag_survives_leaving_its_widget() {
    let mut ui = ui();
    let source = Id::new("arbre");

    ui.begin(
        Pointer {
            at: Vec2::new(10.0, 10.0),
            down: true,
            pressed: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.start_drag(source, "sprites/hero.png");
    ui.end();

    // La souris part ailleurs, bouton toujours enfonce.
    ui.begin(
        Pointer {
            at: Vec2::new(500.0, 400.0),
            down: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    let porte = ui.dragging().cloned();
    ui.end();

    assert_eq!(
        porte,
        Some(Drag {
            from: source,
            payload: "sprites/hero.png".to_owned()
        })
    );
}

#[test]
fn dropping_on_a_target_hands_over_the_payload() {
    let mut ui = ui();
    let source = Id::new("arbre");
    let cible = Rect::new(200.0, 200.0, 100.0, 100.0);

    ui.begin(
        Pointer {
            at: Vec2::new(10.0, 10.0),
            down: true,
            pressed: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.start_drag(source, "sprites/hero.png");
    ui.end();

    ui.begin(
        Pointer {
            at: Vec2::new(250.0, 250.0),
            released: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    let recu = ui.take_drop(cible);
    ui.end();

    assert!(recu.is_some(), "le depot n'a pas ete recu");
    assert_eq!(recu.unwrap().payload, "sprites/hero.png");
}

#[test]
fn a_drop_is_handed_over_only_once() {
    let mut ui = ui();
    let cible = Rect::new(0.0, 0.0, 100.0, 100.0);

    ui.begin(
        Pointer {
            down: true,
            pressed: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.start_drag(Id::new("arbre"), "a.png");
    ui.end();

    ui.begin(
        Pointer {
            at: Vec2::new(50.0, 50.0),
            released: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    assert!(ui.take_drop(cible).is_some());
    // Un second receveur ne doit pas recevoir le meme depot.
    assert!(
        ui.take_drop(cible).is_none(),
        "le depot a ete livre deux fois"
    );
    ui.end();
}

#[test]
fn dropping_outside_every_target_abandons_the_drag() {
    let mut ui = ui();

    ui.begin(
        Pointer {
            down: true,
            pressed: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.start_drag(Id::new("arbre"), "a.png");
    ui.end();

    // Relache loin de toute cible : le glissement est abandonne, sinon il
    // collerait au curseur indefiniment.
    ui.begin(
        Pointer {
            at: Vec2::new(900.0, 900.0),
            released: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.end();

    assert!(ui.dragging().is_none(), "le glissement colle au curseur");
}

#[test]
fn nothing_is_dropped_while_the_button_is_still_down() {
    let mut ui = ui();
    let cible = Rect::new(0.0, 0.0, 100.0, 100.0);

    ui.begin(
        Pointer {
            down: true,
            pressed: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    ui.start_drag(Id::new("arbre"), "a.png");
    ui.end();

    ui.begin(
        Pointer {
            at: Vec2::new(50.0, 50.0),
            down: true,
            ..Pointer::default()
        },
        Keys::default(),
    );
    assert!(ui.take_drop(cible).is_none(), "depose avant le relachement");
    ui.end();
}
