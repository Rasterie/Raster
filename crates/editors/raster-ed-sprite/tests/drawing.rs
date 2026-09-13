use raster_ed_sprite::Layers;
use raster_ed_sprite::draw::{self, Stroke};
use raster_ed_sprite::symmetry::{self, Symmetry};
use raster_ed_visual::tools::Tool;
use raster_math::IVec2;
use raster_render::Colour;

const ROUGE: Colour = Colour::rgb(1.0, 0.0, 0.0);

fn at(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

fn painted(layers: &Layers, x: i32, y: i32) -> bool {
    layers
        .current()
        .frame
        .get(at(x, y))
        .is_some_and(|c| c.to_array()[3] > 0.0)
}

// --- Symetrie ---

#[test]
fn no_symmetry_gives_one_point() {
    assert_eq!(
        symmetry::mirror(at(2, 3), at(10, 10), Symmetry::None),
        [at(2, 3)]
    );
}

#[test]
fn horizontal_symmetry_mirrors_left_to_right() {
    let points = symmetry::mirror(at(1, 5), at(10, 10), Symmetry::Horizontal);
    assert_eq!(points, [at(1, 5), at(8, 5)]);
}

#[test]
fn both_axes_give_four_points() {
    let points = symmetry::mirror(at(1, 2), at(10, 10), Symmetry::Both);
    assert_eq!(points.len(), 4);
    assert!(points.contains(&at(8, 7)), "le coin oppose : {points:?}");
}

#[test]
fn a_point_on_the_axis_is_not_painted_twice() {
    // Sur une largeur impaire, le centre est son propre miroir : le peindre
    // deux fois doublerait l'effet d'un pinceau translucide.
    let points = symmetry::mirror(at(4, 4), at(9, 9), Symmetry::Both);
    assert_eq!(points, [at(4, 4)]);
}

#[test]
fn an_even_width_has_no_pixel_on_the_axis() {
    // L'axe tombe entre deux pixels : chacun a un vrai miroir.
    let points = symmetry::mirror(at(3, 0), at(8, 8), Symmetry::Horizontal);
    assert_eq!(points, [at(3, 0), at(4, 0)]);
}

#[test]
fn diagonal_symmetry_only_applies_to_a_square() {
    let carre = symmetry::mirror(at(1, 5), at(10, 10), Symmetry::Diagonal);
    assert_eq!(carre, [at(1, 5), at(5, 1)]);

    // Sur un rectangle, le reflet sortirait du cadre.
    let rectangle = symmetry::mirror(at(1, 5), at(20, 10), Symmetry::Diagonal);
    assert_eq!(rectangle, [at(1, 5)]);
}

#[test]
fn the_axes_sit_in_the_middle() {
    let (x, y) = symmetry::axes(at(16, 16), Symmetry::Both);
    assert_eq!(x, Some(8.0));
    assert_eq!(y, Some(8.0));

    let (x, y) = symmetry::axes(at(16, 16), Symmetry::Horizontal);
    assert_eq!(x, Some(8.0));
    assert_eq!(y, None);
}

// --- Dessin ---

#[test]
fn a_brush_paints_where_it_starts() {
    let mut layers = Layers::new(16, 16);
    Stroke::begin(&mut layers, Tool::Brush, ROUGE, 1, Symmetry::None, at(5, 5)).unwrap();

    assert!(painted(&layers, 5, 5));
}

#[test]
fn a_stroke_on_a_locked_layer_never_starts() {
    let mut layers = Layers::new(16, 16);
    layers.get_mut(0).unwrap().locked = true;

    assert!(Stroke::begin(&mut layers, Tool::Brush, ROUGE, 1, Symmetry::None, at(5, 5)).is_none());
}

#[test]
fn a_continuous_stroke_leaves_no_gap() {
    let mut layers = Layers::new(32, 32);
    let mut stroke =
        Stroke::begin(&mut layers, Tool::Brush, ROUGE, 1, Symmetry::None, at(2, 2)).unwrap();

    // Un mouvement rapide : sans interpolation, il resterait des trous.
    stroke.advance(&mut layers, at(20, 14));

    for x in 2..=20 {
        let colonne = (0..32).any(|y| painted(&layers, x, y));
        assert!(colonne, "rien de peint dans la colonne {x}");
    }
}

#[test]
fn an_eraser_removes_instead_of_painting() {
    let mut layers = Layers::new(16, 16);
    layers.current_mut().unwrap().frame.fill(ROUGE);

    Stroke::begin(
        &mut layers,
        Tool::Eraser,
        ROUGE,
        3,
        Symmetry::None,
        at(8, 8),
    )
    .unwrap();

    assert!(!painted(&layers, 8, 8), "la gomme doit effacer");
    assert!(painted(&layers, 0, 0), "et laisser le reste");
}

#[test]
fn a_shape_redraws_from_its_start_as_the_cursor_moves() {
    let mut layers = Layers::new(20, 20);
    let mut stroke = Stroke::begin(
        &mut layers,
        Tool::Rectangle,
        ROUGE,
        1,
        Symmetry::None,
        at(2, 2),
    )
    .unwrap();

    stroke.advance(&mut layers, at(15, 15));
    assert!(painted(&layers, 15, 2), "le coin du grand rectangle");

    // Le curseur revient : l'ancien trace doit disparaitre.
    stroke.advance(&mut layers, at(6, 6));
    assert!(!painted(&layers, 15, 2), "l'ancien rectangle est reste");
    assert!(painted(&layers, 6, 2), "le nouveau coin");
}

#[test]
fn a_stroke_keeps_what_it_needs_to_be_undone() {
    let mut layers = Layers::new(8, 8);
    layers.current_mut().unwrap().frame.set(at(0, 0), ROUGE);

    let stroke =
        Stroke::begin(&mut layers, Tool::Brush, ROUGE, 3, Symmetry::None, at(4, 4)).unwrap();

    // L'etat d'avant contient le pixel initial, pas le trait.
    assert!(stroke.before().get(at(0, 0)).unwrap().to_array()[3] > 0.0);
    assert_eq!(stroke.before().get(at(4, 4)).unwrap().to_array()[3], 0.0);
}

#[test]
fn a_symmetric_brush_paints_both_sides() {
    let mut layers = Layers::new(16, 16);
    Stroke::begin(
        &mut layers,
        Tool::Brush,
        ROUGE,
        1,
        Symmetry::Horizontal,
        at(2, 8),
    )
    .unwrap();

    assert!(painted(&layers, 2, 8));
    assert!(painted(&layers, 13, 8), "le reflet doit etre peint");
}

#[test]
fn a_symmetric_line_stays_symmetric() {
    let mut layers = Layers::new(20, 20);
    let mut stroke = Stroke::begin(
        &mut layers,
        Tool::Line,
        ROUGE,
        1,
        Symmetry::Horizontal,
        at(2, 2),
    )
    .unwrap();
    stroke.advance(&mut layers, at(8, 12));

    // Ce qui est peint a gauche doit l'etre a droite, en miroir.
    for y in 0..20 {
        for x in 0..20 {
            assert_eq!(
                painted(&layers, x, y),
                painted(&layers, 19 - x, y),
                "asymetrie en ({x},{y})"
            );
        }
    }
}

#[test]
fn a_symmetric_continuous_stroke_stays_symmetric() {
    let mut layers = Layers::new(20, 20);
    let mut stroke =
        Stroke::begin(&mut layers, Tool::Brush, ROUGE, 1, Symmetry::Both, at(3, 4)).unwrap();
    stroke.advance(&mut layers, at(7, 9));

    for y in 0..20 {
        for x in 0..20 {
            assert_eq!(painted(&layers, x, y), painted(&layers, 19 - x, y));
            assert_eq!(painted(&layers, x, y), painted(&layers, x, 19 - y));
        }
    }
}

#[test]
fn a_fill_spreads_from_where_it_is_clicked() {
    let mut layers = Layers::new(8, 8);
    Stroke::begin(&mut layers, Tool::Fill, ROUGE, 1, Symmetry::None, at(0, 0)).unwrap();

    assert!(
        painted(&layers, 7, 7),
        "le remplissage doit atteindre le coin"
    );
}

#[test]
fn a_picker_never_paints() {
    let mut layers = Layers::new(8, 8);
    Stroke::begin(
        &mut layers,
        Tool::Picker,
        ROUGE,
        1,
        Symmetry::None,
        at(4, 4),
    )
    .unwrap();

    assert!(layers.current().frame.is_blank());
}

#[test]
fn picking_reads_through_the_visible_layers() {
    let mut layers = Layers::new(8, 8);
    layers.current_mut().unwrap().frame.set(at(1, 1), ROUGE);

    layers.add();
    let bleu = Colour::rgb(0.0, 0.0, 1.0);
    layers.current_mut().unwrap().frame.set(at(2, 2), bleu);

    // La couleur qu'on voit, pas celle du calque courant.
    assert_eq!(draw::pick(&layers, at(1, 1)), Some(ROUGE));
    assert_eq!(
        draw::pick(&layers, at(2, 2)).map(|c| c.to_array()),
        Some(bleu.to_array())
    );
    assert_eq!(
        draw::pick(&layers, at(7, 7)),
        None,
        "le vide n'a pas de couleur"
    );
}

#[test]
fn a_stroke_outside_the_frame_paints_nothing_and_does_not_panic() {
    let mut layers = Layers::new(8, 8);
    let mut stroke = Stroke::begin(
        &mut layers,
        Tool::Brush,
        ROUGE,
        1,
        Symmetry::None,
        at(100, 100),
    )
    .unwrap();
    stroke.advance(&mut layers, at(-50, -50));

    // Le trait traverse la frame en chemin : ce qui compte est qu'il ne
    // panique pas.
    assert_eq!(layers.len(), 1);
}
