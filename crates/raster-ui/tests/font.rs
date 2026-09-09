use raster_ui::font::{self, ADVANCE, GLYPH_HEIGHT, GLYPH_WIDTH, LINE_HEIGHT};

/// Dessine un glyphe en texte, pour comparer a ce qu'on attend.
fn render(c: char) -> String {
    (0..GLYPH_HEIGHT)
        .map(|y| {
            (0..GLYPH_WIDTH)
                .map(|x| if font::pixel(c, x, y) { '#' } else { '.' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_printable_ascii_has_a_glyph() {
    for code in 32u8..=126 {
        let c = code as char;
        assert!(
            font::glyph(c).is_some(),
            "pas de glyphe pour {c:?} ({code})"
        );
    }
}

#[test]
fn characters_outside_the_table_have_no_glyph() {
    for c in ['\n', '\t', '\u{7f}', 'é', 'あ', '\0'] {
        assert!(
            font::glyph(c).is_none(),
            "{c:?} ne devrait pas avoir de glyphe"
        );
    }
}

#[test]
fn the_space_is_blank_and_everything_else_is_not() {
    assert_eq!(
        render(' '),
        ".....\n.....\n.....\n.....\n.....\n.....\n....."
    );

    for code in 33u8..=126 {
        let c = code as char;
        let dessine = (0..GLYPH_HEIGHT).any(|y| (0..GLYPH_WIDTH).any(|x| font::pixel(c, x, y)));
        assert!(dessine, "le glyphe {c:?} est vide");
    }
}

#[test]
fn the_letter_a_looks_like_an_a() {
    // Un test qui lit le dessin : une table de masques se relit mal, et une
    // ligne decalee ne se verrait que sur un ecran.
    assert_eq!(
        render('A'),
        ".###.\n\
         #...#\n\
         #...#\n\
         #####\n\
         #...#\n\
         #...#\n\
         #...#"
    );
}

#[test]
fn the_digit_zero_is_not_the_letter_o() {
    // Une police ou 0 et O se confondent rend un HUD illisible.
    assert_ne!(render('0'), render('O'), "0 et O sont identiques");
    assert_ne!(render('1'), render('l'), "1 et l sont identiques");
}

#[test]
fn a_glyph_fits_in_its_cell() {
    for code in 32u8..=126 {
        let c = code as char;
        let rows = font::glyph(c).unwrap();

        for (y, &row) in rows.iter().enumerate() {
            assert!(
                row < (1 << GLYPH_WIDTH),
                "{c:?} ligne {y} deborde de {GLYPH_WIDTH} colonnes : {row:#b}"
            );
        }
    }
}

#[test]
fn pixels_outside_a_glyph_are_never_set() {
    assert!(!font::pixel('A', GLYPH_WIDTH, 0));
    assert!(!font::pixel('A', 0, GLYPH_HEIGHT));
    assert!(
        !font::pixel('é', 0, 0),
        "un caractere absent ne dessine rien"
    );
}

#[test]
fn letters_that_should_differ_do() {
    // Une paire identique signale un copier-coller rate dans la table.
    let mut vus: Vec<(char, String)> = Vec::new();

    for code in 33u8..=126 {
        let c = code as char;
        let dessin = render(c);

        if let Some((autre, _)) = vus.iter().find(|(_, d)| *d == dessin) {
            // Seules quelques paires ont le droit de se ressembler.
            let permis = matches!((autre, c), ('C', 'c') | ('S', 's') | ('X', 'x'));
            assert!(permis, "{autre:?} et {c:?} ont le meme dessin");
        }
        vus.push((c, dessin));
    }
}

#[test]
fn the_metrics_leave_room_between_glyphs() {
    assert!(ADVANCE > GLYPH_WIDTH, "les lettres se toucheraient");
    assert!(LINE_HEIGHT > GLYPH_HEIGHT, "les lignes se toucheraient");
}

#[test]
fn uppercase_and_lowercase_are_distinct() {
    for c in 'a'..='z' {
        let majuscule = c.to_ascii_uppercase();
        if matches!(c, 'c' | 's' | 'x' | 'o' | 'v' | 'w' | 'z') {
            continue;
        }
        assert_ne!(
            render(c),
            render(majuscule),
            "{c:?} et {majuscule:?} sont identiques"
        );
    }
}

/// Les lignes ou un glyphe pose des pixels.
fn rows_used(c: char) -> (u32, u32) {
    let mut first = GLYPH_HEIGHT;
    let mut last = 0;
    for y in 0..GLYPH_HEIGHT {
        if (0..GLYPH_WIDTH).any(|x| font::pixel(c, x, y)) {
            first = first.min(y);
            last = y;
        }
    }
    (first, last)
}

#[test]
fn lowercase_bodies_sit_on_the_same_line() {
    // Un corps decale d'une ligne se voit tout de suite a l'ecran, et jamais
    // dans un test qui ne regarde que la forme de la table.
    let (haut, _) = rows_used('o');

    for c in [
        'a', 'c', 'e', 'g', 'm', 'n', 'p', 'q', 'r', 's', 'u', 'v', 'w', 'x', 'z',
    ] {
        let (debut, _) = rows_used(c);
        assert_eq!(
            debut, haut,
            "le corps de {c:?} commence ligne {debut}, celui de 'o' ligne {haut}"
        );
    }
}

#[test]
fn every_lowercase_ends_on_the_same_baseline() {
    // Une cellule de sept lignes ne laisse pas la place a un vrai jambage sous
    // le corps : les descendantes le portent dans leurs deux dernieres lignes.
    // Ce qui compte est qu'aucune lettre ne flotte au-dessus des autres.
    let (_, base) = rows_used('o');

    for c in 'a'..='z' {
        let (_, bas) = rows_used(c);
        assert_eq!(bas, base, "{c:?} finit ligne {bas} au lieu de {base}");
    }
}

#[test]
fn descenders_are_shaped_differently_from_their_neighbours() {
    // Si p ressemblait a o, le mot "pop" serait illisible.
    for (descendante, voisine) in [('p', 'o'), ('q', 'o'), ('g', 'o'), ('y', 'v')] {
        let mut differe = false;
        for y in 0..GLYPH_HEIGHT {
            for x in 0..GLYPH_WIDTH {
                if font::pixel(descendante, x, y) != font::pixel(voisine, x, y) {
                    differe = true;
                }
            }
        }
        assert!(differe, "{descendante:?} et {voisine:?} sont identiques");
    }
}

#[test]
fn capitals_all_start_at_the_top() {
    for c in 'A'..='Z' {
        let (haut, _) = rows_used(c);
        assert_eq!(haut, 0, "la majuscule {c:?} commence ligne {haut}");
    }
}
