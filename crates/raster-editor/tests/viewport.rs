use raster_core::World;
use raster_core::reflect::Reflect;
use raster_editor::viewport::{self, Handle, Mode, Viewport};
use raster_math::{Rect, Vec2};

const PANEL: Rect = Rect {
    position: Vec2 { x: 100.0, y: 50.0 },
    size: Vec2 { x: 640.0, y: 360.0 },
};

#[derive(Reflect, Default, Debug)]
struct Prop {
    position: Vec2,
}

#[test]
fn the_centre_of_the_panel_is_the_camera() {
    let mut view = Viewport::new();
    view.camera = Vec2::new(200.0, 300.0);

    let centre = PANEL.position + PANEL.size * 0.5;
    assert_eq!(view.to_world(PANEL, centre), view.camera);
}

#[test]
fn screen_and_world_are_inverses() {
    let mut view = Viewport::new();
    view.camera = Vec2::new(-40.0, 90.0);
    view.zoom = 4;

    for point in [
        Vec2::ZERO,
        Vec2::new(123.0, -45.0),
        Vec2::new(1000.0, 1000.0),
    ] {
        let round_trip = view.to_world(PANEL, view.to_screen(PANEL, point));
        assert!(
            (round_trip - point).length() < 0.001,
            "{point:?} est revenu en {round_trip:?}"
        );
    }
}

#[test]
fn zooming_keeps_the_point_under_the_cursor() {
    let mut view = Viewport::new();
    let curseur = Vec2::new(300.0, 200.0);
    let avant = view.to_world(PANEL, curseur);

    view.zoom_at(PANEL, curseur, 1);
    let apres = view.to_world(PANEL, curseur);

    // Sans ce point fixe, zoomer deplacerait ce qu'on regarde.
    assert!(
        (apres - avant).length() < 0.001,
        "le point sous le curseur a bouge : {avant:?} puis {apres:?}"
    );
}

#[test]
fn zoom_stays_within_bounds() {
    let mut view = Viewport::new();

    for _ in 0..20 {
        view.zoom_at(PANEL, Vec2::ZERO, 1);
    }
    assert_eq!(view.zoom, Viewport::MAX_ZOOM);

    for _ in 0..20 {
        view.zoom_at(PANEL, Vec2::ZERO, -1);
    }
    assert_eq!(view.zoom, 1, "le zoom ne doit jamais tomber a zero");
}

#[test]
fn the_visible_area_shrinks_as_the_zoom_grows() {
    let mut view = Viewport::new();
    view.zoom = 1;
    let large = view.visible(PANEL);

    view.zoom = 4;
    let serre = view.visible(PANEL);

    assert!(serre.size.x < large.size.x);
    assert_eq!(serre.size.x, large.size.x / 4.0);
}

#[test]
fn snapping_lands_on_the_grid() {
    let view = Viewport::new();

    assert_eq!(view.snapped(Vec2::new(17.0, 31.0)), Vec2::new(16.0, 32.0));
    assert_eq!(view.snapped(Vec2::new(-3.0, 8.1)), Vec2::new(0.0, 16.0));
}

#[test]
fn without_snapping_positions_stay_whole() {
    let mut view = Viewport::new();
    view.snap = false;

    // Toujours au pixel : un acteur a mi-pixel rendrait flou.
    assert_eq!(view.snapped(Vec2::new(17.4, 31.6)), Vec2::new(17.0, 32.0));
}

#[test]
fn a_box_select_is_normalised() {
    let mut view = Viewport::new();

    // Tire vers le haut a gauche : la taille doit rester positive.
    view.begin_drag(Vec2::new(100.0, 100.0), Mode::BoxSelect);
    view.drag_to(Vec2::new(40.0, 30.0));
    let rect = view.end_drag().unwrap();

    assert_eq!(rect.position, Vec2::new(40.0, 30.0));
    assert_eq!(rect.size, Vec2::new(60.0, 70.0));
}

#[test]
fn a_drag_reports_how_far_the_cursor_went() {
    let mut view = Viewport::new();
    view.begin_drag(Vec2::ZERO, Mode::Move);

    assert_eq!(view.drag_to(Vec2::new(10.0, 5.0)), Vec2::new(10.0, 5.0));
    // Le suivant est relatif au precedent, pas au depart.
    assert_eq!(view.drag_to(Vec2::new(12.0, 5.0)), Vec2::new(2.0, 0.0));
}

#[test]
fn ending_a_drag_returns_to_selecting() {
    let mut view = Viewport::new();
    view.begin_drag(Vec2::ZERO, Mode::Move);
    assert!(view.dragging());

    view.end_drag();
    assert!(!view.dragging());
    assert_eq!(view.mode, Mode::Select);
}

#[test]
fn ending_a_drag_that_never_began_gives_nothing() {
    assert_eq!(Viewport::new().end_drag(), None);
}

#[test]
fn selecting_replaces_what_was_selected() {
    let mut world = World::new();
    let a = world.spawn(Prop::default());
    let b = world.spawn(Prop::default());
    let mut view = Viewport::new();

    view.select(a);
    view.select(b);

    assert_eq!(view.selection(), [b]);
}

#[test]
fn toggling_adds_then_removes() {
    let mut world = World::new();
    let a = world.spawn(Prop::default());
    let b = world.spawn(Prop::default());
    let mut view = Viewport::new();

    view.toggle(a);
    view.toggle(b);
    assert_eq!(view.selection(), [a, b]);

    view.toggle(a);
    assert_eq!(view.selection(), [b]);
}

#[test]
fn the_inspector_sees_a_lone_selection_only() {
    let mut world = World::new();
    let a = world.spawn(Prop::default());
    let b = world.spawn(Prop::default());
    let mut view = Viewport::new();

    assert_eq!(view.only_selected(), None, "rien de selectionne");

    view.select(a);
    assert_eq!(view.only_selected(), Some(a));

    // A plusieurs, il n'y a pas de valeur commune a montrer.
    view.toggle(b);
    assert_eq!(view.only_selected(), None);
}

#[test]
fn selecting_the_same_actor_twice_keeps_one_entry() {
    let mut world = World::new();
    let a = world.spawn(Prop::default());
    let mut view = Viewport::new();

    view.select_all([a, a, a]);
    assert_eq!(view.selection(), [a]);
}

#[test]
fn a_deleted_actor_leaves_the_selection() {
    let mut world = World::new();
    let a = world.spawn(Prop::default());
    let b = world.spawn(Prop::default());
    let mut view = Viewport::new();
    view.select_all([a, b]);

    world.despawn(a);
    // Une selection qui survit a une suppression pointerait dans le vide.
    view.retain_selection(|id| world.contains(id));

    assert_eq!(view.selection(), [b]);
}

#[test]
fn handles_surround_an_actor() {
    let bounds = Rect::new(100.0, 100.0, 16.0, 16.0);
    let poignees = viewport::handles(bounds);

    assert_eq!(poignees.len(), 3);
    assert_eq!(poignees[0].0, Handle::Body);
    assert_eq!(poignees[0].1, bounds, "le corps couvre l'acteur");
}

#[test]
fn an_axis_handle_constrains_the_movement() {
    let libre = Vec2::new(10.0, 7.0);

    assert_eq!(viewport::constrain(Handle::Body, libre), libre);
    assert_eq!(
        viewport::constrain(Handle::AxisX, libre),
        Vec2::new(10.0, 0.0)
    );
    assert_eq!(
        viewport::constrain(Handle::AxisY, libre),
        Vec2::new(0.0, 7.0)
    );
}
