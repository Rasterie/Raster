use raster_ed_visual::Frame;
use raster_ed_visual::tools::{self, Tool};
use raster_math::IVec2;
use raster_render::Colour;

const ROUGE: Colour = Colour::rgb(1.0, 0.0, 0.0);
const BLEU: Colour = Colour::rgb(0.0, 0.0, 1.0);

fn at(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

/// Dessine une frame en texte, pour lire ce qui a ete pose.
fn render(frame: &Frame) -> String {
    (0..frame.height())
        .map(|y| {
            (0..frame.width())
                .map(|x| {
                    if frame.get(at(x, y)).unwrap().to_array()[3] > 0.0 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_brush_of_one_paints_one_pixel() {
    let mut frame = Frame::new(5, 5);
    tools::brush(&mut frame, at(2, 2), 1, ROUGE);

    assert_eq!(frame.get(at(2, 2)), Some(ROUGE));
    assert_eq!(frame.get(at(1, 2)).unwrap().to_array()[3], 0.0);
}

#[test]
fn a_brush_of_three_paints_a_square_around_the_pixel() {
    let mut frame = Frame::new(5, 5);
    tools::brush(&mut frame, at(2, 2), 3, ROUGE);

    assert_eq!(
        render(&frame),
        ".....\n\
         .###.\n\
         .###.\n\
         .###.\n\
         ....."
    );
}

#[test]
fn a_brush_at_the_edge_paints_only_what_fits() {
    let mut frame = Frame::new(3, 3);
    // Un pinceau qui deborde ne doit pas paniquer.
    tools::brush(&mut frame, at(0, 0), 3, ROUGE);

    assert_eq!(render(&frame), "##.\n##.\n...");
}

#[test]
fn a_line_never_leaves_a_gap() {
    // Une interpolation flottante trouerait une pente raide.
    let pixels = tools::line_pixels(at(0, 0), at(2, 8));

    for pair in pixels.windows(2) {
        let step = (pair[1].x - pair[0].x).abs() + (pair[1].y - pair[0].y).abs();
        assert!(
            step <= 2,
            "un saut de {step} entre {:?} et {:?}",
            pair[0],
            pair[1]
        );
    }
    assert_eq!(pixels.first(), Some(&at(0, 0)));
    assert_eq!(pixels.last(), Some(&at(2, 8)));
}

#[test]
fn a_line_covers_the_same_ground_in_both_directions() {
    // Bresenham departage les pixels a egale distance selon le sens de
    // parcours : les deux lignes ne sont pas identiques pixel a pixel, mais
    // elles ont la meme longueur et les memes extremites.
    let avant = tools::line_pixels(at(1, 1), at(7, 4));
    let apres = tools::line_pixels(at(7, 4), at(1, 1));

    assert_eq!(avant.len(), apres.len());
    assert_eq!(avant.first(), apres.last());
    assert_eq!(avant.last(), apres.first());

    // Et aucune ne s'ecarte de la droite theorique de plus d'un pixel.
    for pixel in avant.iter().chain(apres.iter()) {
        let t = (pixel.x - 1) as f32 / 6.0;
        let attendu = 1.0 + t * 3.0;
        assert!(
            (pixel.y as f32 - attendu).abs() <= 1.0,
            "{pixel:?} s'ecarte de la droite"
        );
    }
}

#[test]
fn a_line_to_itself_is_one_pixel() {
    assert_eq!(tools::line_pixels(at(3, 3), at(3, 3)), [at(3, 3)]);
}

#[test]
fn a_horizontal_line_is_straight() {
    let mut frame = Frame::new(6, 3);
    tools::line(&mut frame, at(1, 1), at(4, 1), 1, ROUGE);

    assert_eq!(render(&frame), "......\n.####.\n......");
}

#[test]
fn filling_covers_a_connected_area() {
    let mut frame = Frame::filled(4, 4, BLEU);
    // Un mur qui coupe la frame en deux.
    for y in 0..4 {
        frame.set(at(2, y), ROUGE);
    }

    let painted = tools::fill(&mut frame, at(0, 0), ROUGE);

    assert_eq!(painted, 8, "seule la moitie gauche doit etre remplie");
    assert_eq!(frame.get(at(3, 0)), Some(BLEU), "la droite est intacte");
}

#[test]
fn filling_does_not_cross_diagonals() {
    // Deux zones qui ne se touchent qu'en diagonale sont deux zones.
    let mut frame = Frame::filled(3, 3, BLEU);
    frame.set(at(1, 0), ROUGE);
    frame.set(at(0, 1), ROUGE);

    let painted = tools::fill(&mut frame, at(0, 0), ROUGE);

    assert_eq!(painted, 1, "le coin est isole");
    assert_eq!(frame.get(at(2, 2)), Some(BLEU));
}

#[test]
fn filling_with_the_same_colour_does_nothing() {
    let mut frame = Frame::filled(4, 4, BLEU);

    // Sans ce test, le remplissage boucherait indefiniment.
    assert_eq!(tools::fill(&mut frame, at(0, 0), BLEU), 0);
}

#[test]
fn filling_outside_the_frame_does_nothing() {
    let mut frame = Frame::filled(3, 3, BLEU);
    assert_eq!(tools::fill(&mut frame, at(10, 10), ROUGE), 0);
}

#[test]
fn filling_transparent_treats_every_transparent_as_one() {
    let mut frame = Frame::new(3, 3);
    // Un pixel efface n'a pas de couleur visible : deux transparents de teintes
    // differentes sont la meme zone.
    frame.set(at(1, 1), Colour::rgba(1.0, 0.0, 0.0, 0.0));

    assert_eq!(tools::fill(&mut frame, at(0, 0), ROUGE), 9);
}

#[test]
fn a_rectangle_is_hollow() {
    let mut frame = Frame::new(5, 5);
    tools::rectangle(&mut frame, at(1, 1), at(3, 3), ROUGE);

    assert_eq!(
        render(&frame),
        ".....\n\
         .###.\n\
         .#.#.\n\
         .###.\n\
         ....."
    );
}

#[test]
fn a_filled_rectangle_is_solid() {
    let mut frame = Frame::new(4, 4);
    tools::rectangle_filled(&mut frame, at(3, 3), at(1, 1), ROUGE);

    // Les coins peuvent venir dans n'importe quel ordre.
    assert_eq!(render(&frame), "....\n.###\n.###\n.###");
}

#[test]
fn an_ellipse_is_round_and_hollow() {
    let mut frame = Frame::new(7, 7);
    tools::ellipse(&mut frame, at(0, 0), at(6, 6), ROUGE);

    let dessin = render(&frame);
    // Le centre reste vide, les bords sont poses.
    assert_eq!(
        frame.get(at(3, 3)).unwrap().to_array()[3],
        0.0,
        "\n{dessin}"
    );
    assert!(
        frame.get(at(3, 0)).unwrap().to_array()[3] > 0.0,
        "\n{dessin}"
    );
    assert!(
        frame.get(at(0, 3)).unwrap().to_array()[3] > 0.0,
        "\n{dessin}"
    );
    // Les coins du rectangle ne sont pas dans l'ellipse.
    assert_eq!(
        frame.get(at(0, 0)).unwrap().to_array()[3],
        0.0,
        "\n{dessin}"
    );
}

#[test]
fn tools_know_what_they_do() {
    assert!(Tool::Brush.is_continuous());
    assert!(
        !Tool::Line.is_continuous(),
        "une ligne attend le relachement"
    );

    assert!(Tool::Fill.paints());
    assert!(!Tool::Picker.paints(), "une pipette ne peint pas");
    assert!(!Tool::Select.paints());
}

#[test]
fn two_transparent_colours_are_the_same() {
    assert!(tools::same(
        Colour::rgba(1.0, 0.0, 0.0, 0.0),
        Colour::rgba(0.0, 0.0, 1.0, 0.0)
    ));
    assert!(!tools::same(ROUGE, BLEU));
    assert!(tools::same(ROUGE, ROUGE));
}
