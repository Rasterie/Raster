use raster_ed_visual::{Frame, Frames, Palette};
use raster_math::IVec2;
use raster_render::Colour;

const ROUGE: Colour = Colour::rgb(1.0, 0.0, 0.0);
const VERT: Colour = Colour::rgb(0.0, 1.0, 0.0);

#[test]
fn the_default_palette_has_sixteen_colours() {
    let palette = Palette::default_sixteen();

    assert_eq!(palette.len(), 16);
    assert!(!palette.is_empty());
}

#[test]
fn a_colour_can_be_added_and_read_back() {
    let mut palette = Palette::new("test");
    let index = palette.push(ROUGE).unwrap();

    assert_eq!(palette.get(index), Some(ROUGE));
    assert_eq!(palette.len(), 1);
}

#[test]
fn a_palette_refuses_to_grow_past_its_limit() {
    let mut palette = Palette::new("test");
    for _ in 0..Palette::MAX {
        assert!(palette.push(ROUGE).is_some());
    }

    // Au-dela, ce n'est plus une palette mais une image.
    assert_eq!(palette.push(VERT), None);
    assert_eq!(palette.len(), Palette::MAX);
}

#[test]
fn selecting_picks_the_colour_a_tool_paints_with() {
    let mut palette = Palette::default_sixteen();
    palette.select(5);

    assert_eq!(palette.primary(), palette.get(5).unwrap());
    assert_eq!(palette.primary_index(), 5);
}

#[test]
fn selecting_past_the_end_changes_nothing() {
    let mut palette = Palette::default_sixteen();
    palette.select(3);
    palette.select(999);

    assert_eq!(
        palette.primary_index(),
        3,
        "une selection hors liste est ignoree"
    );
}

#[test]
fn swapping_exchanges_the_two_colours() {
    let mut palette = Palette::default_sixteen();
    palette.select(2);
    palette.select_secondary(7);

    let (avant_p, avant_s) = (palette.primary(), palette.secondary());
    palette.swap();

    assert_eq!(palette.primary(), avant_s);
    assert_eq!(palette.secondary(), avant_p);
}

#[test]
fn removing_keeps_the_selection_on_the_same_colour() {
    let mut palette = Palette::new("test");
    for c in [ROUGE, VERT, Colour::rgb(0.0, 0.0, 1.0)] {
        palette.push(c);
    }
    palette.select(2);
    let regarde = palette.primary();

    // Retirer une couleur avant celle choisie : la selection doit suivre, pas
    // designer une couleur differente.
    palette.remove(0);
    assert_eq!(palette.primary(), regarde);
    assert_eq!(palette.primary_index(), 1);
}

#[test]
fn removing_the_selected_colour_stays_in_range() {
    let mut palette = Palette::new("test");
    palette.push(ROUGE);
    palette.push(VERT);
    palette.select(1);

    palette.remove(1);
    assert!(
        palette.primary_index() < palette.len(),
        "la selection sort de la liste"
    );
}

#[test]
fn removing_the_last_colour_leaves_a_usable_palette() {
    let mut palette = Palette::new("test");
    palette.push(ROUGE);
    palette.select(0);

    palette.remove(0);
    assert!(palette.is_empty());
    // Une palette vide rend du transparent plutot que de paniquer.
    assert_eq!(palette.primary().to_array()[3], 0.0);
}

#[test]
fn removing_something_absent_changes_nothing() {
    let mut palette = Palette::default_sixteen();
    assert!(!palette.remove(999));
    assert_eq!(palette.len(), 16);
}

#[test]
fn the_nearest_colour_is_found() {
    let mut palette = Palette::new("test");
    palette.push(Colour::rgb(0.0, 0.0, 0.0));
    palette.push(Colour::rgb(1.0, 1.0, 1.0));
    palette.push(ROUGE);

    // Un rouge legerement different doit tomber sur le rouge.
    assert_eq!(palette.nearest(Colour::rgb(0.9, 0.1, 0.05)), Some(2));
    assert_eq!(palette.nearest(Colour::rgb(0.95, 0.95, 0.95)), Some(1));
}

#[test]
fn an_empty_palette_has_no_nearest_colour() {
    assert_eq!(Palette::new("vide").nearest(ROUGE), None);
}

// --- Frames ---

#[test]
fn a_new_frame_is_transparent() {
    let frame = Frame::new(4, 4);

    assert!(frame.is_blank());
    assert_eq!(frame.get(IVec2::ZERO).unwrap().to_array()[3], 0.0);
}

#[test]
fn a_pixel_outside_the_frame_is_neither_read_nor_written() {
    let mut frame = Frame::new(3, 3);

    assert_eq!(frame.get(IVec2::new(3, 0)), None);
    assert_eq!(frame.get(IVec2::new(-1, 0)), None);
    // Un pinceau qui deborde ne doit pas etre une erreur.
    assert!(!frame.set(IVec2::new(10, 10), ROUGE));
    assert!(frame.is_blank());
}

#[test]
fn a_frame_survives_a_trip_through_bytes() {
    let mut frame = Frame::new(3, 2);
    frame.set(IVec2::new(0, 0), ROUGE);
    frame.set(IVec2::new(2, 1), VERT);

    let bytes = frame.to_rgba();
    assert_eq!(bytes.len(), 3 * 2 * 4);

    let relu = Frame::from_rgba(3, 2, &bytes).unwrap();
    assert_eq!(relu.get(IVec2::new(0, 0)).unwrap().to_array()[0], 1.0);
    assert_eq!(relu.size(), frame.size());
}

#[test]
fn bytes_of_the_wrong_length_are_refused() {
    // Mieux vaut refuser qu'afficher une image decalee.
    assert_eq!(Frame::from_rgba(4, 4, &[0; 10]), None);
    assert_eq!(Frame::from_rgba(0, 4, &[]), None);
}

#[test]
fn a_colour_out_of_range_does_not_wrap() {
    let mut frame = Frame::new(1, 1);
    frame.set(IVec2::ZERO, Colour::rgba(2.0, -1.0, 0.5, 1.0));

    let bytes = frame.to_rgba();
    // Sans bornage, 2.0 deborderait en boucle et le blanc deviendrait noir.
    assert_eq!(bytes[0], 255);
    assert_eq!(bytes[1], 0);
}

#[test]
fn resizing_keeps_the_top_left_corner() {
    let mut frame = Frame::new(4, 4);
    frame.set(IVec2::new(0, 0), ROUGE);
    frame.set(IVec2::new(3, 3), VERT);

    frame.resize(2, 2);

    assert_eq!(frame.size(), IVec2::new(2, 2));
    assert_eq!(frame.get(IVec2::ZERO), Some(ROUGE), "le coin doit rester");
    assert_eq!(frame.get(IVec2::new(3, 3)), None);
}

#[test]
fn growing_leaves_the_new_area_transparent() {
    let mut frame = Frame::filled(2, 2, ROUGE);
    frame.resize(4, 4);

    assert_eq!(frame.get(IVec2::ZERO), Some(ROUGE));
    assert_eq!(frame.get(IVec2::new(3, 3)).unwrap().to_array()[3], 0.0);
}

#[test]
fn a_sequence_starts_with_one_frame() {
    let frames = Frames::new(8, 8);

    assert_eq!(frames.len(), 1);
    assert_eq!(frames.index(), 0);
}

#[test]
fn inserting_adds_an_empty_frame_after_the_current() {
    let mut frames = Frames::new(4, 4);
    frames.current_mut().set(IVec2::ZERO, ROUGE);

    frames.insert_after();

    assert_eq!(frames.len(), 2);
    assert_eq!(frames.index(), 1, "la nouvelle devient la courante");
    assert!(frames.current().is_blank());
}

#[test]
fn duplicating_copies_what_was_drawn() {
    let mut frames = Frames::new(4, 4);
    frames.current_mut().set(IVec2::ZERO, ROUGE);

    frames.duplicate();

    // C'est ainsi qu'on anime : repartir de la frame precedente.
    assert_eq!(frames.current().get(IVec2::ZERO), Some(ROUGE));
    assert_eq!(frames.len(), 2);
}

#[test]
fn a_sequence_refuses_to_lose_its_last_frame() {
    let mut frames = Frames::new(4, 4);

    assert!(!frames.remove(0), "une sequence sans frame n'a pas de sens");
    assert_eq!(frames.len(), 1);
}

#[test]
fn removing_keeps_the_selection_in_range() {
    let mut frames = Frames::new(4, 4);
    frames.insert_after();
    frames.insert_after();
    frames.select(2);

    frames.remove(2);
    assert!(frames.index() < frames.len());
}

#[test]
fn removing_an_early_colour_shifts_a_late_selection() {
    // Avec une palette courte, un `min` final masque la difference : il faut
    // assez de couleurs apres la selection pour la voir se decaler.
    let mut palette = Palette::default_sixteen();
    palette.select(10);
    let regarde = palette.primary();

    palette.remove(2);

    assert_eq!(palette.primary_index(), 9, "la selection n'a pas suivi");
    assert_eq!(palette.primary(), regarde, "elle designe une autre couleur");
}

#[test]
fn removing_a_later_colour_leaves_the_selection_alone() {
    let mut palette = Palette::default_sixteen();
    palette.select(3);
    let regarde = palette.primary();

    palette.remove(10);

    assert_eq!(palette.primary_index(), 3);
    assert_eq!(palette.primary(), regarde);
}

#[test]
fn a_negative_channel_becomes_zero_not_white() {
    // `as u8` sature vers le haut en Rust, mais un negatif deviendrait zero
    // par troncature — sauf que -1.0 * 255 vaut -255, qui sature aussi a 0.
    // Le bornage explicite dit ce qu'on veut plutot que de s'y fier.
    let mut frame = Frame::new(1, 1);
    frame.set(IVec2::ZERO, Colour::rgba(-1.0, 0.5, 2.0, 1.0));

    let bytes = frame.to_rgba();
    assert_eq!(bytes[0], 0, "un canal negatif doit etre noir");
    assert_eq!(bytes[1], 128, "0.5 arrondi");
    assert_eq!(bytes[2], 255, "un canal au-dela de 1 reste blanc");
}

#[test]
fn rounding_is_not_truncation() {
    let mut frame = Frame::new(1, 1);
    // 0.999 * 255 = 254,7 : tronquer donnerait 254, arrondir 255.
    frame.set(IVec2::ZERO, Colour::rgba(0.999, 0.0, 0.0, 1.0));

    assert_eq!(frame.to_rgba()[0], 255, "la conversion doit arrondir");
}
