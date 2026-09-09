use raster_math::{Rect, Vec2};
use raster_ui::{Id, Keys, Pointer, Theme, Ui};

const AREA: Rect = Rect {
    position: Vec2 { x: 0.0, y: 0.0 },
    size: Vec2 { x: 120.0, y: 16.0 },
};

fn ui() -> Ui {
    Ui::new(Theme::dark())
}

/// Un clic complet sur `area`, en deux frames.
fn click(ui: &mut Ui, id: Id, area: Rect) -> bool {
    let centre = Vec2::new(
        area.position.x + area.size.x / 2.0,
        area.position.y + area.size.y / 2.0,
    );

    let appui = Pointer {
        at: centre,
        down: true,
        pressed: true,
        released: false,
        ..Pointer::default()
    };
    ui.begin(appui, Keys::default());
    ui.interact(id, area, true);
    ui.end();

    let relache = Pointer {
        at: centre,
        down: false,
        pressed: false,
        released: true,
        ..Pointer::default()
    };
    ui.begin(relache, Keys::default());
    let response = ui.interact(id, area, true);
    ui.end();

    response.clicked
}

#[test]
fn typed_characters_reach_the_focused_field() {
    let mut ui = ui();
    let id = Id::new("nom");
    let mut valeur = String::new();

    ui.focus(id);
    ui.set_typed("abc");
    ui.begin(Pointer::default(), Keys::default());
    let response = ui.interact(id, AREA, true);
    assert!(response.focused);

    // Ce que `text_field` fait de la saisie, sans avoir besoin d'un batcher.
    for c in ui.typed().chars() {
        valeur.push(c);
    }
    ui.end();

    assert_eq!(valeur, "abc");
}

#[test]
fn a_field_without_focus_ignores_typing() {
    let mut ui = ui();
    let id = Id::new("nom");

    ui.set_typed("abc");
    ui.begin(Pointer::default(), Keys::default());
    let response = ui.interact(id, AREA, true);
    ui.end();

    assert!(
        !response.focused,
        "un champ sans focus ne doit rien recevoir"
    );
}

#[test]
fn typing_is_cleared_between_frames() {
    let mut ui = ui();

    ui.set_typed("a");
    ui.begin(Pointer::default(), Keys::default());
    ui.end();
    assert_eq!(ui.typed(), "a");

    ui.set_typed("");
    ui.begin(Pointer::default(), Keys::default());
    ui.end();
    assert_eq!(ui.typed(), "", "la saisie de la frame precedente persiste");
}

#[test]
fn a_field_remembers_its_text_between_frames() {
    let mut ui = ui();
    let id = Id::new("nom");

    assert_eq!(ui.remember_text(id, "defaut"), "defaut");
    ui.store_text(id, "Gabin".to_owned());

    ui.begin(Pointer::default(), Keys::default());
    ui.end();

    assert_eq!(ui.remember_text(id, "defaut"), "Gabin");
}

#[test]
fn clicking_a_list_row_selects_it() {
    let mut ui = ui();
    let liste = Id::new("liste");

    // La troisieme ligne, telle que `list` la place.
    let theme = *ui.theme();
    let row = theme.row_height;
    let ligne = Rect::new(theme.padding, theme.padding + 2.0 * row, 100.0, row);

    assert!(click(&mut ui, liste.index(2), ligne));
}

#[test]
fn list_rows_have_distinct_identities() {
    let liste = Id::new("liste");
    let a = liste.index(0);
    let b = liste.index(1);

    assert_ne!(a, b);
    assert_eq!(a, Id::new("liste").index(0), "l'identite doit etre stable");
}

#[test]
fn tabs_and_lists_do_not_share_identities() {
    // Deux widgets indexes sous des noms differents ne doivent pas se croiser.
    assert_ne!(Id::new("onglets").index(0), Id::new("liste").index(0));
}

#[test]
fn a_radio_reports_its_click() {
    let mut ui = ui();
    let groupe = Id::new("difficulte");

    assert!(click(&mut ui, groupe.index(1), AREA));
}

#[test]
fn keyboard_navigation_walks_a_group_of_radios() {
    let mut ui = ui();
    let groupe = Id::new("difficulte");
    let suivant = Keys {
        next: true,
        ..Keys::default()
    };

    let mut vus = Vec::new();
    for _ in 0..3 {
        ui.begin(Pointer::default(), suivant);
        for i in 0..3 {
            ui.interact(groupe.index(i), AREA, true);
        }
        ui.end();
        vus.push(ui.focused());
    }

    assert_eq!(
        vus,
        [
            Some(groupe.index(0)),
            Some(groupe.index(1)),
            Some(groupe.index(2))
        ]
    );
}

#[test]
fn backspace_is_carried_separately_from_text() {
    // Une touche d'effacement n'est pas un caractere : elle a son champ.
    let keys = Keys {
        backspace: true,
        ..Keys::default()
    };

    assert!(keys.backspace);
    assert!(!keys.confirm, "les touches ne doivent pas se contaminer");
}

#[test]
fn editing_appends_what_was_typed() {
    let mut valeur = "Ke".to_owned();
    raster_ui::edit(&mut valeur, "ys", false);

    assert_eq!(valeur, "Keys");
}

#[test]
fn editing_refuses_what_the_font_cannot_draw() {
    // Sans ce filtre, une lettre accentuee entrerait et sortirait en trou.
    let mut valeur = String::new();
    raster_ui::edit(&mut valeur, "aéb", false);

    assert_eq!(valeur, "ab");
}

#[test]
fn backspace_removes_the_last_character() {
    let mut valeur = "abc".to_owned();
    raster_ui::edit(&mut valeur, "", true);
    assert_eq!(valeur, "ab");

    // Sur un champ vide, effacer ne doit pas paniquer.
    let mut vide = String::new();
    raster_ui::edit(&mut vide, "", true);
    assert_eq!(vide, "");
}

#[test]
fn typing_and_erasing_in_one_frame_both_apply() {
    let mut valeur = "ab".to_owned();
    raster_ui::edit(&mut valeur, "cd", true);

    // La saisie entre, puis l'effacement retire le dernier.
    assert_eq!(valeur, "abc");
}

#[test]
fn a_field_shows_the_end_of_a_text_that_overflows() {
    use raster_ui::Painter;

    let painter = Painter::new(0);
    let large = painter.measure("abcdefghij").x;

    // Assez large : rien n'est coupe.
    assert_eq!(
        raster_ui::clip_end(&painter, "abcdefghij", large),
        "abcdefghij"
    );

    // Trop etroit : c'est la fin qu'on garde, car c'est la qu'on tape.
    let etroit = raster_ui::clip_end(&painter, "abcdefghij", painter.measure("ghij").x);
    assert!(
        "abcdefghij".ends_with(&etroit),
        "la coupe doit garder la fin, pas le debut : {etroit:?}"
    );
    assert!(painter.measure(&etroit).x <= painter.measure("ghij").x);
}

#[test]
fn a_field_narrower_than_one_glyph_shows_nothing() {
    use raster_ui::Painter;

    let painter = Painter::new(0);
    assert_eq!(raster_ui::clip_end(&painter, "abc", 0.0), "");
}
