//! Parite des formes avec la version TypeScript.
//!
//! Les masques attendus viennent de `buildFromRecipe`, relevés en la faisant
//! tourner. Un pixel de difference se voit sur un sprite.

use rasterie::grammar::{self, Angles, Corner, Fill, Recipe, Silhouette};

/// Compare un masque a ce que le TypeScript rend, lignes separees par `|`.
fn expect(recipe: Recipe, w: usize, h: usize, angles: Angles, expected: &str) {
    let mask = grammar::build(recipe, w, h, angles);
    let got = mask.to_text().replace('\n', "|");

    assert_eq!(
        got,
        expected,
        "\n{:?} {w}x{h} {angles:?}\nobtenu :\n{}\n",
        recipe.silhouette,
        mask.to_text()
    );
}

fn recipe(silhouette: Silhouette) -> Recipe {
    Recipe {
        silhouette,
        ..Recipe::default()
    }
}

#[test]
fn a_diamond_matches_the_typescript_one() {
    expect(
        recipe(Silhouette::Diamond),
        9,
        9,
        Angles::Free,
        "....#....|...###...|..#####..|.#######.|#########|.#######.|..#####..|...###...|....#....",
    );
}

#[test]
fn a_hexagon_matches() {
    expect(
        recipe(Silhouette::Hexagon),
        11,
        9,
        Angles::Free,
        ".....#.....|...#####...|###########|###########|###########|###########|###########|...#####...|.....#.....",
    );
}

#[test]
fn a_capsule_matches() {
    expect(
        recipe(Silhouette::Capsule),
        11,
        7,
        Angles::Free,
        "...#####...|.#########.|.#########.|###########|.#########.|.#########.|...#####...",
    );
}

#[test]
fn a_shield_matches() {
    expect(
        recipe(Silhouette::Shield),
        9,
        11,
        Angles::Free,
        "#########|#########|#########|#########|#########|#########|.#######.|.#######.|..#####..|...###...|....#....",
    );
}

#[test]
fn a_banner_matches() {
    expect(
        recipe(Silhouette::Banner),
        11,
        7,
        Angles::Free,
        "###########|.#########.|.#########.|..#######..|.#########.|.#########.|###########",
    );
}

#[test]
fn a_cross_matches() {
    expect(
        recipe(Silhouette::Cross),
        9,
        9,
        Angles::Free,
        "...###...|...###...|...###...|#########|#########|#########|...###...|...###...|...###...",
    );
}

#[test]
fn a_cut_corner_matches() {
    expect(
        Recipe {
            corner: Corner::Cut,
            ..Recipe::default()
        },
        9,
        9,
        Angles::Free,
        ".#######.|#########|#########|#########|#########|#########|#########|#########|.#######.",
    );
}

#[test]
fn a_round_corner_matches() {
    expect(
        Recipe {
            corner: Corner::Round,
            corner_size: 3,
            ..Recipe::default()
        },
        11,
        11,
        Angles::Free,
        "..#######..|..#######..|###########|###########|###########|###########|###########|###########|###########|..#######..|..#######..",
    );
}

#[test]
fn a_hollow_fill_matches() {
    expect(
        Recipe {
            fill: Fill::Hollow,
            thickness: 2,
            ..Recipe::default()
        },
        9,
        9,
        Angles::Free,
        "#########|#########|##.....##|##.....##|##.....##|##.....##|##.....##|#########|#########",
    );
}

#[test]
fn a_half_fill_matches() {
    expect(
        Recipe {
            fill: Fill::Half,
            ..Recipe::default()
        },
        8,
        8,
        Angles::Free,
        "########|########|########|########|........|........|........|........",
    );
}

#[test]
fn diagonal_angles_sharpen_the_edges() {
    // Les pentes canoniques : la seule diagonale que l'oeil suit sans accroc.
    expect(
        recipe(Silhouette::Diamond),
        11,
        11,
        Angles::Diagonal,
        ".....#.....|....###....|...#####...|..#######..|.#########.|###########|.#########.|..#######..|...#####...|....###....|.....#.....",
    );
}

#[test]
fn orthogonal_angles_pull_the_points_in() {
    expect(
        recipe(Silhouette::Diamond),
        11,
        11,
        Angles::Orthogonal,
        "...........|....###....|...#####...|..#######..|.#########.|.#########.|.#########.|..#######..|...#####...|....###....|...........",
    );
}

// --- Proprietes que la grammaire doit tenir, tous cas confondus ---

#[test]
fn the_grammar_covers_every_combination() {
    assert_eq!(grammar::COMBINATIONS, 7 * 5 * 4);
}

#[test]
fn no_recipe_ever_produces_an_empty_shape() {
    // Une recette qui rend une forme vide serait invisible dans l'editeur,
    // sans que rien ne signale l'erreur.
    for silhouette in Silhouette::ALL {
        for corner in Corner::ALL {
            for fill in Fill::ALL {
                let mask = grammar::build(
                    Recipe {
                        silhouette,
                        corner,
                        fill,
                        ..Recipe::default()
                    },
                    16,
                    16,
                    Angles::Free,
                );

                assert!(
                    !mask.is_empty(),
                    "{silhouette:?}/{corner:?}/{fill:?} ne dessine rien"
                );
            }
        }
    }
}

#[test]
fn every_shape_fits_inside_its_mask() {
    for silhouette in Silhouette::ALL {
        let mask = grammar::build(recipe(silhouette), 12, 12, Angles::Free);

        assert_eq!(mask.width(), 12);
        assert_eq!(mask.height(), 12);
    }
}

#[test]
fn a_mask_is_never_smaller_than_three() {
    // En dessous, aucune forme n'est reconnaissable.
    let mask = grammar::build(Recipe::default(), 1, 1, Angles::Free);

    assert_eq!(mask.width(), 3);
    assert_eq!(mask.height(), 3);
}

#[test]
fn a_solid_rect_fills_everything() {
    let mask = grammar::build(Recipe::default(), 8, 8, Angles::Free);
    assert_eq!(mask.count(), 64);
}

#[test]
fn hollowing_removes_pixels_without_emptying() {
    let solid = grammar::build(Recipe::default(), 16, 16, Angles::Free);
    let hollow = grammar::build(
        Recipe {
            fill: Fill::Hollow,
            ..Recipe::default()
        },
        16,
        16,
        Angles::Free,
    );

    assert!(
        hollow.count() < solid.count(),
        "evider doit retirer des pixels"
    );
    assert!(!hollow.is_empty());
}

#[test]
fn a_double_fill_puts_a_ring_back() {
    let hollow = grammar::build(
        Recipe {
            fill: Fill::Hollow,
            ..Recipe::default()
        },
        24,
        24,
        Angles::Free,
    );
    let double = grammar::build(
        Recipe {
            fill: Fill::Double,
            ..Recipe::default()
        },
        24,
        24,
        Angles::Free,
    );

    assert!(
        double.count() > hollow.count(),
        "le lisere central doit ajouter des pixels"
    );
}

#[test]
fn a_corner_size_of_zero_leaves_the_shape_alone() {
    let square = grammar::build(Recipe::default(), 9, 9, Angles::Free);
    let no_cut = grammar::build(
        Recipe {
            corner: Corner::Cut,
            corner_size: 0,
            ..Recipe::default()
        },
        9,
        9,
        Angles::Free,
    );

    assert_eq!(square.to_text(), no_cut.to_text());
}

#[test]
fn a_shape_is_symmetric_left_to_right() {
    // Une silhouette de travers se voit immediatement.
    for silhouette in [
        Silhouette::Diamond,
        Silhouette::Hexagon,
        Silhouette::Capsule,
        Silhouette::Cross,
        Silhouette::Banner,
    ] {
        let mask = grammar::build(recipe(silhouette), 11, 11, Angles::Free);

        for y in 0..11i64 {
            for x in 0..11i64 {
                assert_eq!(
                    mask.is_inside(x, y),
                    mask.is_inside(10 - x, y),
                    "{silhouette:?} n'est pas symetrique en ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn the_diagonal_threshold_never_touches_a_diamond() {
    // Un diamant est deja `|nx|+|ny| <= 1` : le seuil de 1.34 ne coupe rien.
    // Le tester ici ne prouverait donc pas que le seuil est le bon.
    let libre = grammar::build(recipe(Silhouette::Diamond), 13, 13, Angles::Free);
    let diagonal = grammar::build(recipe(Silhouette::Diamond), 13, 13, Angles::Diagonal);

    assert_eq!(libre.to_text(), diagonal.to_text());
}

#[test]
fn the_diagonal_threshold_bites_where_it_should() {
    // L'hexagone, la croix et le bandeau depassent 1.30 : c'est la que le
    // seuil se verifie.
    expect(
        recipe(Silhouette::Hexagon),
        13,
        13,
        Angles::Diagonal,
        "......#......|.....###.....|..#########..|.###########.|#############|#############|#############|#############|#############|.###########.|..#########..|.....###.....|......#......",
    );
    expect(
        recipe(Silhouette::Cross),
        13,
        13,
        Angles::Diagonal,
        "....#####....|....#####....|....#####....|....#####....|#############|#############|#############|#############|#############|....#####....|....#####....|....#####....|....#####....",
    );
    expect(
        recipe(Silhouette::Banner),
        13,
        13,
        Angles::Diagonal,
        "....#####....|...#######...|..#########..|.###########.|.###########.|..#########..|..#########..|..#########..|.###########.|.###########.|..#########..|...#######...|....#####....",
    );
}

#[test]
fn the_cross_arm_bites_at_the_right_size() {
    // Meme raison : a 21 colonnes, 0.400 separe 0.38 de 0.42.
    expect(
        recipe(Silhouette::Cross),
        21,
        21,
        Angles::Free,
        ".......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......|#####################|#####################|#####################|#####################|#####################|#####################|#####################|.......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......|.......#######.......",
    );
}
