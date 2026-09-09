use raster_math::{Rect, Vec2};
use raster_ui::text::{self, Align};

/// Rend un texte dans une grille de caracteres, comme le peintre le poserait
/// a l'ecran. Ce que `Painter::text` fait avec des rectangles, en ASCII.
fn render(
    content: &str,
    width: usize,
    height: usize,
    at: Vec2,
    align: Align,
    scale: f32,
) -> String {
    let mut grid = vec![vec![' '; width]; height];

    for placed in text::layout(content, at, align, scale) {
        for (gx, gy) in text::glyph_pixels(placed.c) {
            for sy in 0..scale as usize {
                for sx in 0..scale as usize {
                    let x = placed.at.x as usize + gx as usize * scale as usize + sx;
                    let y = placed.at.y as usize + gy as usize * scale as usize + sy;
                    if y < height && x < width {
                        grid[y][x] = '#';
                    }
                }
            }
        }
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_word_draws_where_it_is_placed() {
    let ecran = render("HI", 16, 8, Vec2::new(1.0, 0.0), Align::Left, 1.0);

    // Le H et le I, cote a cote, decales d'un pixel depuis la gauche.
    let attendu = [
        " #   #  ###     ",
        " #   #   #      ",
        " #   #   #      ",
        " #####   #      ",
        " #   #   #      ",
        " #   #   #      ",
        " #   #  ###     ",
        "                ",
    ]
    .join("\n");

    assert_eq!(ecran, attendu, "\n{ecran}");
}

#[test]
fn centred_text_is_actually_centred() {
    let ecran = render("AB", 24, 7, Vec2::new(12.0, 0.0), Align::Centre, 1.0);

    // Autant de colonnes vides de chaque cote, a un pixel pres.
    let ligne = ecran.lines().next().unwrap();
    let gauche = ligne.len() - ligne.trim_start().len();
    let droite = ligne.len() - ligne.trim_end().len();

    assert!(
        gauche.abs_diff(droite) <= 1,
        "{gauche} a gauche, {droite} a droite :\n{ecran}"
    );
}

#[test]
fn nothing_is_drawn_outside_the_text_bounds() {
    // Ce que `text_bounds` promet doit contenir tout ce qui est dessine.
    let at = Vec2::new(10.0, 5.0);
    let taille = text::measure("Hello") * 2.0;
    let bounds = Rect::new(at.x, at.y, taille.x, taille.y);

    for placed in text::layout("Hello", at, Align::Left, 2.0) {
        for (gx, gy) in text::glyph_pixels(placed.c) {
            let x = placed.at.x + gx as f32 * 2.0;
            let y = placed.at.y + gy as f32 * 2.0;

            assert!(
                x >= bounds.position.x && x < bounds.position.x + bounds.size.x,
                "un pixel sort a droite : {x} pour {bounds:?}"
            );
            assert!(
                y >= bounds.position.y && y < bounds.position.y + bounds.size.y,
                "un pixel sort en bas : {y} pour {bounds:?}"
            );
        }
    }
}

#[test]
fn scaling_makes_every_pixel_a_block() {
    let simple = render("I", 8, 8, Vec2::ZERO, Align::Left, 1.0);
    let double = render("I", 16, 16, Vec2::ZERO, Align::Left, 2.0);

    let pixels_simple = simple.chars().filter(|c| *c == '#').count();
    let pixels_double = double.chars().filter(|c| *c == '#').count();

    assert_eq!(
        pixels_double,
        pixels_simple * 4,
        "doubler l'echelle doit quadrupler les pixels"
    );
}

#[test]
fn two_lines_do_not_overlap() {
    let ecran = render("AA\nAA", 16, 16, Vec2::ZERO, Align::Left, 1.0);
    let lignes: Vec<&str> = ecran.lines().collect();

    // La ligne 7 separe les deux rangees de lettres.
    assert!(
        lignes[7].trim().is_empty(),
        "les lignes se touchent :\n{ecran}"
    );
    assert!(!lignes[8].trim().is_empty(), "la seconde ligne manque");
}

#[test]
fn a_full_screen_of_text_is_legible() {
    // L'ecran-titre du jeu, tel qu'il sera dessine.
    let ecran = render("KEYSTONE", 60, 8, Vec2::new(30.0, 0.0), Align::Centre, 1.0);

    // Huit lettres, chacune posant des pixels : aucune colonne de lettre vide.
    let colonnes_pleines = (0..60)
        .filter(|&x| ecran.lines().any(|l| l.chars().nth(x) == Some('#')))
        .count();

    assert!(
        colonnes_pleines >= 32,
        "seulement {colonnes_pleines} colonnes dessinees :\n{ecran}"
    );
}
