use raster_math::{Rect, Vec2};
use raster_ui::{Id, Keys, Pointer, State, Theme, Ui};

fn ui() -> Ui {
    Ui::new(Theme::dark())
}

fn at(x: f32, y: f32) -> Pointer {
    Pointer {
        at: Vec2::new(x, y),
        ..Pointer::default()
    }
}

const AREA: Rect = Rect {
    position: Vec2 { x: 10.0, y: 10.0 },
    size: Vec2 { x: 100.0, y: 20.0 },
};

#[test]
fn a_pointer_inside_hovers() {
    let mut ui = ui();
    let id = Id::new("bouton");

    ui.begin(at(50.0, 20.0), Keys::default());
    // Deux passes : le survol se resout a la fin de la frame, car le dernier
    // widget dessine est celui du dessus.
    ui.interact(id, AREA, true);
    ui.end();

    ui.begin(at(50.0, 20.0), Keys::default());
    let response = ui.interact(id, AREA, true);
    ui.end();

    assert!(response.hovered);
    assert_eq!(response.state(), State::Hovered);
}

#[test]
fn a_pointer_outside_does_not_hover() {
    let mut ui = ui();
    ui.begin(at(500.0, 500.0), Keys::default());
    let response = ui.interact(Id::new("bouton"), AREA, true);
    ui.end();

    assert!(!response.hovered);
    assert_eq!(response.state(), State::Idle);
}

#[test]
fn a_click_needs_press_then_release_on_the_widget() {
    let mut ui = ui();
    let id = Id::new("bouton");

    let mut appui = at(50.0, 20.0);
    appui.down = true;
    appui.pressed = true;
    ui.begin(appui, Keys::default());
    let pendant = ui.interact(id, AREA, true);
    ui.end();

    assert!(!pendant.clicked, "un clic ne compte pas des l'appui");
    assert!(pendant.held);

    let mut relache = at(50.0, 20.0);
    relache.released = true;
    ui.begin(relache, Keys::default());
    let apres = ui.interact(id, AREA, true);
    ui.end();

    assert!(apres.clicked, "le relachement sur le widget doit valider");
}

#[test]
fn releasing_outside_cancels_the_click() {
    let mut ui = ui();
    let id = Id::new("bouton");

    let mut appui = at(50.0, 20.0);
    appui.down = true;
    appui.pressed = true;
    ui.begin(appui, Keys::default());
    ui.interact(id, AREA, true);
    ui.end();

    // Relache loin du bouton : le clic est annule, comme partout ailleurs.
    let mut ailleurs = at(500.0, 500.0);
    ailleurs.released = true;
    ui.begin(ailleurs, Keys::default());
    let response = ui.interact(id, AREA, true);
    ui.end();

    assert!(!response.clicked, "relacher dehors ne doit pas valider");
}

#[test]
fn a_dragged_widget_keeps_the_pointer_when_it_leaves() {
    let mut ui = ui();
    let id = Id::new("curseur");

    let mut appui = at(50.0, 20.0);
    appui.down = true;
    appui.pressed = true;
    ui.begin(appui, Keys::default());
    ui.interact(id, AREA, true);
    ui.end();

    // La souris sort, bouton toujours enfonce : le widget garde la main.
    let mut dehors = at(500.0, 500.0);
    dehors.down = true;
    ui.begin(dehors, Keys::default());
    let response = ui.interact(id, AREA, true);
    ui.end();

    assert!(
        response.held,
        "la capture est perdue des que la souris sort"
    );
    assert_eq!(ui.captured(), Some(id));
}

#[test]
fn releasing_frees_the_capture() {
    let mut ui = ui();
    let id = Id::new("curseur");

    let mut appui = at(50.0, 20.0);
    appui.down = true;
    appui.pressed = true;
    ui.begin(appui, Keys::default());
    ui.interact(id, AREA, true);
    ui.end();
    assert_eq!(ui.captured(), Some(id));

    let mut relache = at(50.0, 20.0);
    relache.released = true;
    ui.begin(relache, Keys::default());
    ui.end();

    assert_eq!(ui.captured(), None);
}

#[test]
fn the_last_widget_drawn_wins_the_hover() {
    let mut ui = ui();
    let dessous = Id::new("dessous");
    let dessus = Id::new("dessus");

    // Deux widgets au meme endroit : celui dessine en dernier est au-dessus.
    for _ in 0..2 {
        ui.begin(at(50.0, 20.0), Keys::default());
        ui.interact(dessous, AREA, true);
        ui.interact(dessus, AREA, true);
        ui.end();
    }

    assert_eq!(ui.hovered(), Some(dessus));
}

#[test]
fn a_disabled_widget_never_reacts() {
    let mut ui = ui();
    let id = Id::new("bouton");

    // Deux frames : le survol se resout a la fin de la premiere, et une seule
    // passe ne verrait donc jamais un survol errone.
    let mut response = None;
    for _ in 0..2 {
        let mut clic = at(50.0, 20.0);
        clic.down = true;
        clic.pressed = true;
        ui.begin(clic, Keys::default());
        response = Some(ui.interact(id, AREA, false));
        ui.end();
    }

    let response = response.unwrap();
    assert!(!response.hovered);
    assert!(!response.clicked);
    assert_eq!(response.state(), State::Disabled);
    assert_eq!(
        ui.count(),
        0,
        "un widget desactive ne doit pas prendre le focus"
    );
    assert_eq!(ui.hovered(), None, "un widget desactive a capte le survol");
    assert_eq!(ui.focused(), None, "un widget desactive a pris le focus");
}

#[test]
fn clicking_gives_focus() {
    let mut ui = ui();
    let id = Id::new("bouton");

    let mut appui = at(50.0, 20.0);
    appui.down = true;
    appui.pressed = true;
    ui.begin(appui, Keys::default());
    ui.interact(id, AREA, true);
    ui.end();

    assert_eq!(ui.focused(), Some(id));
}

#[test]
fn tab_walks_the_widgets_in_order() {
    let mut ui = ui();
    let a = Id::new("a");
    let b = Id::new("b");
    let c = Id::new("c");

    let suivant = Keys {
        next: true,
        ..Keys::default()
    };

    let mut vus = Vec::new();
    for _ in 0..3 {
        ui.begin(Pointer::default(), suivant);
        ui.interact(a, AREA, true);
        ui.interact(b, AREA, true);
        ui.interact(c, AREA, true);
        ui.end();
        vus.push(ui.focused());
    }

    assert_eq!(vus, [Some(a), Some(b), Some(c)]);
}

#[test]
fn tab_wraps_around() {
    let mut ui = ui();
    let a = Id::new("a");
    let b = Id::new("b");
    let suivant = Keys {
        next: true,
        ..Keys::default()
    };

    for _ in 0..3 {
        ui.begin(Pointer::default(), suivant);
        ui.interact(a, AREA, true);
        ui.interact(b, AREA, true);
        ui.end();
    }

    assert_eq!(ui.focused(), Some(a), "le focus doit revenir au premier");
}

#[test]
fn shift_tab_walks_backwards() {
    let mut ui = ui();
    let a = Id::new("a");
    let b = Id::new("b");
    let precedent = Keys {
        previous: true,
        ..Keys::default()
    };

    ui.begin(Pointer::default(), precedent);
    ui.interact(a, AREA, true);
    ui.interact(b, AREA, true);
    ui.end();

    assert_eq!(
        ui.focused(),
        Some(b),
        "sans focus, reculer prend le dernier"
    );
}

#[test]
fn confirm_activates_the_focused_widget() {
    let mut ui = ui();
    let id = Id::new("bouton");

    ui.focus(id);
    ui.begin(
        Pointer::default(),
        Keys {
            confirm: true,
            ..Keys::default()
        },
    );
    let response = ui.interact(id, AREA, true);
    ui.end();

    assert!(response.clicked, "Entree doit activer le widget au focus");
    assert!(response.focused);
}

#[test]
fn focus_on_a_vanished_widget_is_dropped() {
    let mut ui = ui();
    let id = Id::new("disparu");

    ui.focus(id);
    // Une frame ou le widget n'est pas dessine : un panneau referme.
    ui.begin(Pointer::default(), Keys::default());
    ui.end();

    assert_eq!(ui.focused(), None);
}

#[test]
fn a_widget_remembers_a_value_between_frames() {
    let mut ui = ui();
    let id = Id::new("ascenseur");

    assert_eq!(ui.remember(id, 0.0), 0.0);
    ui.store(id, 42.0);

    ui.begin(Pointer::default(), Keys::default());
    ui.end();

    assert_eq!(
        ui.remember(id, 0.0),
        42.0,
        "l'etat n'a pas survecu a la frame"
    );
}

#[test]
fn two_ids_from_the_same_name_match() {
    assert_eq!(Id::new("jouer"), Id::new("jouer"));
    assert_ne!(Id::new("jouer"), Id::new("quitter"));
}

#[test]
fn the_same_name_under_two_parents_differs() {
    // Deux boutons "Supprimer" dans deux panneaux ne doivent pas se confondre.
    let a = Id::new("panneau_a").child("supprimer");
    let b = Id::new("panneau_b").child("supprimer");

    assert_ne!(a, b);
    assert_eq!(a, Id::new("panneau_a").child("supprimer"));
}

#[test]
fn list_items_get_distinct_ids() {
    let liste = Id::new("liste");
    let ids: Vec<_> = (0..5).map(|i| liste.index(i)).collect();

    for (i, id) in ids.iter().enumerate() {
        for (j, autre) in ids.iter().enumerate() {
            if i != j {
                assert_ne!(id, autre, "les elements {i} et {j} partagent un id");
            }
        }
    }
}
