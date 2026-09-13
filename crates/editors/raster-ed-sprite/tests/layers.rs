use raster_ed_sprite::layers::{self, Blend, Layers};
use raster_math::IVec2;
use raster_render::Colour;

const ROUGE: Colour = Colour::rgb(1.0, 0.0, 0.0);
const BLEU: Colour = Colour::rgb(0.0, 0.0, 1.0);

fn at(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

#[test]
fn a_sprite_starts_with_one_layer() {
    let layers = Layers::new(16, 16);

    assert_eq!(layers.len(), 1);
    assert_eq!(layers.index(), 0);
    assert!(layers.current().visible);
}

#[test]
fn adding_puts_the_new_layer_above_and_selects_it() {
    let mut layers = Layers::new(8, 8);
    layers.current_mut().unwrap().frame.set(at(0, 0), ROUGE);

    let index = layers.add().unwrap();

    assert_eq!(index, 1, "le nouveau va au-dessus");
    assert_eq!(layers.index(), 1);
    assert!(layers.current().frame.is_blank(), "et il est vide");
}

#[test]
fn duplicating_copies_the_pixels() {
    let mut layers = Layers::new(8, 8);
    layers.current_mut().unwrap().frame.set(at(2, 2), ROUGE);

    layers.duplicate();

    assert_eq!(layers.current().frame.get(at(2, 2)), Some(ROUGE));
    assert_eq!(layers.len(), 2);
}

#[test]
fn a_stack_refuses_to_lose_its_last_layer() {
    let mut layers = Layers::new(8, 8);

    assert!(!layers.remove(0), "un sprite sans calque n'a pas de sens");
    assert_eq!(layers.len(), 1);
}

#[test]
fn a_stack_refuses_to_grow_past_its_limit() {
    let mut layers = Layers::new(4, 4);
    while layers.len() < Layers::MAX {
        assert!(layers.add().is_some());
    }

    assert_eq!(layers.add(), None);
    assert_eq!(layers.duplicate(), None);
}

#[test]
fn a_hidden_layer_cannot_be_drawn_on() {
    let mut layers = Layers::new(8, 8);
    layers.get_mut(0).unwrap().visible = false;

    // Peindre sur ce qu'on ne voit pas serait pire que de refuser.
    assert!(layers.current_mut().is_none());
}

#[test]
fn a_locked_layer_cannot_be_drawn_on() {
    let mut layers = Layers::new(8, 8);
    layers.get_mut(0).unwrap().locked = true;

    assert!(layers.current_mut().is_none());
    assert!(layers.current().visible, "mais il reste visible");
}

#[test]
fn moving_a_layer_keeps_the_selection_on_it() {
    let mut layers = Layers::new(4, 4);
    layers.add();
    layers.add();
    layers.select(0);
    let nom = layers.current().name.clone();

    // Le calque du bas remonte tout en haut.
    layers.move_layer(0, 2);

    assert_eq!(
        layers.index(),
        2,
        "la selection suit le calque, pas la place"
    );
    assert_eq!(layers.current().name, nom);
}

#[test]
fn moving_another_layer_shifts_the_selection_correctly() {
    let mut layers = Layers::new(4, 4);
    layers.add();
    layers.add();
    layers.select(1);
    let nom = layers.current().name.clone();

    // Un calque sous la selection passe au-dessus : la selection descend d'un.
    layers.move_layer(0, 2);

    assert_eq!(
        layers.current().name,
        nom,
        "la selection a change de calque"
    );
}

#[test]
fn moving_nowhere_changes_nothing() {
    let mut layers = Layers::new(4, 4);
    layers.add();

    assert!(!layers.move_layer(0, 0));
    assert!(!layers.move_layer(0, 99));
}

#[test]
fn merging_down_combines_the_pixels() {
    let mut layers = Layers::new(4, 4);
    layers.current_mut().unwrap().frame.set(at(0, 0), ROUGE);

    layers.add();
    layers.current_mut().unwrap().frame.set(at(1, 1), BLEU);

    assert!(layers.merge_down());

    assert_eq!(layers.len(), 1);
    assert_eq!(layers.current().frame.get(at(0, 0)), Some(ROUGE));
    assert_eq!(layers.current().frame.get(at(1, 1)), Some(BLEU));
}

#[test]
fn the_bottom_layer_has_nothing_to_merge_into() {
    let mut layers = Layers::new(4, 4);
    layers.add();
    layers.select(0);

    assert!(!layers.merge_down());
    assert_eq!(layers.len(), 2);
}

#[test]
fn flattening_stacks_from_the_bottom_up() {
    let mut layers = Layers::new(2, 2);
    layers.current_mut().unwrap().frame.fill(ROUGE);

    layers.add();
    layers.current_mut().unwrap().frame.fill(BLEU);

    // Le calque du dessus recouvre : c'est le sens de l'empilement.
    assert_eq!(layers.flatten().get(at(0, 0)), Some(BLEU));
}

#[test]
fn a_hidden_layer_is_left_out_of_the_flatten() {
    let mut layers = Layers::new(2, 2);
    layers.current_mut().unwrap().frame.fill(ROUGE);
    layers.add();
    layers.current_mut().unwrap().frame.fill(BLEU);
    layers.get_mut(1).unwrap().visible = false;

    assert_eq!(layers.flatten().get(at(0, 0)), Some(ROUGE));
}

// --- Fusion des couleurs ---

#[test]
fn a_transparent_layer_changes_nothing() {
    let result = layers::blend(ROUGE, Colour::TRANSPARENT, Blend::Normal, 1.0);
    assert_eq!(result, ROUGE);
}

#[test]
fn zero_opacity_changes_nothing() {
    let result = layers::blend(ROUGE, BLEU, Blend::Normal, 0.0);
    assert_eq!(result, ROUGE);
}

#[test]
fn full_opacity_replaces() {
    let result = layers::blend(ROUGE, BLEU, Blend::Normal, 1.0);
    assert_eq!(result.to_array(), BLEU.to_array());
}

#[test]
fn half_opacity_mixes() {
    let result = layers::blend(ROUGE, BLEU, Blend::Normal, 0.5);
    let [r, _, b, _] = result.to_array();

    assert!(r > 0.4 && r < 0.6, "rouge a {r}");
    assert!(b > 0.4 && b < 0.6, "bleu a {b}");
}

#[test]
fn adding_brightens_without_going_past_one() {
    let gris = Colour::rgb(0.6, 0.6, 0.6);
    let result = layers::blend(gris, gris, Blend::Add, 1.0);

    for c in &result.to_array()[..3] {
        assert!(*c <= 1.0, "un canal a deborde : {c}");
        assert!(*c > 0.6, "l'addition doit eclaircir");
    }
}

#[test]
fn multiplying_darkens() {
    let gris = Colour::rgb(0.5, 0.5, 0.5);
    let result = layers::blend(gris, gris, Blend::Multiply, 1.0);

    assert!(
        result.to_array()[0] < 0.5,
        "la multiplication doit assombrir"
    );
}

#[test]
fn clipping_never_paints_on_emptiness() {
    // C'est ainsi qu'on ombre une forme sans deborder.
    let result = layers::blend(Colour::TRANSPARENT, BLEU, Blend::Clip, 1.0);
    assert_eq!(result, Colour::TRANSPARENT);

    let sur_plein = layers::blend(ROUGE, BLEU, Blend::Clip, 1.0);
    assert_eq!(sur_plein.to_array(), BLEU.to_array());
}

#[test]
fn painting_on_emptiness_keeps_the_new_colour() {
    let result = layers::blend(Colour::TRANSPARENT, BLEU, Blend::Normal, 1.0);

    assert_eq!(result.to_array()[3], 1.0, "l'alpha doit suivre");
    assert!(
        (result.to_array()[2] - 1.0).abs() < 0.01,
        "et la couleur aussi"
    );
}
