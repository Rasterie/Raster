use raster_math::Vec2;
use raster_ui::font::{ADVANCE, GLYPH_HEIGHT, LINE_HEIGHT};
use raster_ui::text::{self, Align};

#[test]
fn an_empty_string_measures_as_one_empty_line() {
    let taille = text::measure("");
    assert_eq!(taille.x, 0.0);
    assert_eq!(
        taille.y,
        (LINE_HEIGHT - 1) as f32,
        "une ligne vide a une hauteur"
    );
}

#[test]
fn a_line_is_as_wide_as_its_characters() {
    // Trois lettres : trois avances, moins l'espace de la derniere.
    assert_eq!(text::line_width("abc"), 3 * ADVANCE - 1);
    assert_eq!(text::line_width("a"), ADVANCE - 1);
    assert_eq!(text::line_width(""), 0);
}

#[test]
fn several_lines_stack() {
    let une = text::measure("abc");
    let trois = text::measure("abc\ndef\nghi");

    assert_eq!(trois.x, une.x, "la largeur suit la ligne la plus longue");
    assert_eq!(trois.y, (3 * LINE_HEIGHT - 1) as f32);
    assert_eq!(text::line_count("abc\ndef\nghi"), 3);
}

#[test]
fn width_follows_the_longest_line() {
    let taille = text::measure("a\nabcdef\nab");
    assert_eq!(taille.x, text::line_width("abcdef") as f32);
}

#[test]
fn text_lays_out_left_to_right() {
    let places = text::layout("ab", Vec2::ZERO, Align::Left, 1.0);

    assert_eq!(places.len(), 2);
    assert_eq!(places[0].c, 'a');
    assert_eq!(places[0].at, Vec2::ZERO);
    assert_eq!(places[1].at.x, ADVANCE as f32);
    assert_eq!(places[1].at.y, 0.0);
}

#[test]
fn spaces_advance_without_being_drawn() {
    let places = text::layout("a b", Vec2::ZERO, Align::Left, 1.0);

    assert_eq!(places.len(), 2, "l'espace ne doit pas produire de glyphe");
    assert_eq!(places[1].c, 'b');
    assert_eq!(
        places[1].at.x,
        2.0 * ADVANCE as f32,
        "l'espace doit quand meme avancer le curseur"
    );
}

#[test]
fn centred_text_straddles_its_position() {
    let largeur = text::line_width("abcd") as f32;
    let places = text::layout("abcd", Vec2::new(100.0, 0.0), Align::Centre, 1.0);

    assert_eq!(places[0].at.x, 100.0 - largeur / 2.0);
}

#[test]
fn right_aligned_text_ends_at_its_position() {
    let largeur = text::line_width("abcd") as f32;
    let places = text::layout("abcd", Vec2::new(100.0, 0.0), Align::Right, 1.0);

    assert_eq!(places[0].at.x, 100.0 - largeur);
}

#[test]
fn each_line_is_aligned_on_its_own_width() {
    // Sans cela, un bloc centre se decalerait sur la ligne la plus longue.
    let places = text::layout("a\nabcd", Vec2::new(100.0, 0.0), Align::Centre, 1.0);

    let premiere = places[0].at.x;
    let seconde = places.iter().find(|p| p.at.y > 0.0).unwrap().at.x;

    assert!(
        premiere > seconde,
        "la ligne courte doit etre plus a droite : {premiere} et {seconde}"
    );
}

#[test]
fn lines_are_spaced_by_the_line_height() {
    let places = text::layout("a\nb", Vec2::ZERO, Align::Left, 1.0);
    let seconde = places.iter().find(|p| p.c == 'b').unwrap();

    assert_eq!(seconde.at.y, LINE_HEIGHT as f32);
}

#[test]
fn scaling_multiplies_the_spacing() {
    let places = text::layout("ab\ncd", Vec2::ZERO, Align::Left, 3.0);

    assert_eq!(places[1].at.x, 3.0 * ADVANCE as f32);
    let seconde = places.iter().find(|p| p.c == 'c').unwrap();
    assert_eq!(seconde.at.y, 3.0 * LINE_HEIGHT as f32);
}

#[test]
fn a_negative_scale_is_treated_as_zero() {
    // Une echelle negative retournerait le texte : mieux vaut rien.
    let places = text::layout("ab", Vec2::ZERO, Align::Left, -2.0);
    assert_eq!(places[1].at.x, 0.0);
}

#[test]
fn a_glyph_has_pixels_and_a_space_has_none() {
    assert!(!text::glyph_pixels('A').is_empty());
    assert!(text::glyph_pixels(' ').is_empty());
    assert!(
        text::glyph_pixels('é').is_empty(),
        "un caractere absent ne dessine rien"
    );

    // Tous les pixels tiennent dans la cellule.
    for (x, y) in text::glyph_pixels('W') {
        assert!(x < raster_ui::GLYPH_WIDTH && y < GLYPH_HEIGHT);
    }
}

#[test]
fn drawable_text_is_recognised() {
    assert!(text::is_drawable("Appuyez sur Entree\nEchap pour quitter"));
    assert!(
        !text::is_drawable("Réglages"),
        "un accent n'est pas dessinable"
    );
}

#[test]
fn what_cannot_be_drawn_becomes_a_question_mark() {
    // Sans cela une etiquette accentuee sortirait avec des trous.
    assert_eq!(text::sanitise("Réglages"), "R?glages");
    assert_eq!(text::sanitise("deja\nvu"), "deja\nvu");
}

#[test]
fn every_placed_glyph_can_be_drawn() {
    let places = text::layout("Keystone v0.1.0!", Vec2::ZERO, Align::Left, 1.0);

    for place in places {
        assert!(
            !text::glyph_pixels(place.c).is_empty(),
            "{:?} a ete place mais ne dessine rien",
            place.c
        );
    }
}
