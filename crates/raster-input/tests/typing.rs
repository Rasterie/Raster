use raster_input::{Bindings, Input};

fn input() -> Input {
    Input::new(Bindings::default_layout())
}

#[test]
fn typed_characters_accumulate_in_order() {
    let mut input = input();

    input.type_char('a');
    input.type_char('b');
    input.type_char('c');

    assert_eq!(input.typed(), "abc");
}

#[test]
fn control_characters_never_enter_the_buffer() {
    // Retour arriere, tabulation et entree arrivent comme des caracteres : ce
    // sont des commandes, pas du texte, et elles ont leurs propres touches.
    let mut input = input();

    input.type_char('a');
    input.type_char('\u{8}');
    input.type_char('\t');
    input.type_char('\n');
    input.type_char('\u{1b}');
    input.type_char('b');

    assert_eq!(input.typed(), "ab");
}

#[test]
fn the_buffer_is_emptied_each_frame() {
    let mut input = input();

    input.type_char('a');
    assert_eq!(input.typed(), "a");

    // Sans ce vidage, une lettre tapee une fois s'insererait a chaque frame.
    input.begin_frame(1.0 / 60.0);
    assert_eq!(input.typed(), "");
}

#[test]
fn a_fresh_input_has_typed_nothing() {
    assert_eq!(input().typed(), "");
}

#[test]
fn characters_outside_ascii_still_reach_the_buffer() {
    // L'entree ne juge pas ce qui est dessinable : c'est au champ de filtrer,
    // sinon un jeu avec une autre police perdrait ses accents.
    let mut input = input();
    input.type_char('é');

    assert_eq!(input.typed(), "é");
}
