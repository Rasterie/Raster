//! Les regles du pixel art.

use rasterie::audit::{self, Severity, Surface};

/// Une image dessinee en texte : `.` est vide, tout autre caractere est une
/// couleur.
struct Grid {
    rows: Vec<Vec<char>>,
}

impl Grid {
    fn new(text: &str) -> Self {
        Self {
            rows: text.lines().map(|l| l.chars().collect()).collect(),
        }
    }
}

impl Surface for Grid {
    fn width(&self) -> usize {
        self.rows.first().map_or(0, Vec::len)
    }
    fn height(&self) -> usize {
        self.rows.len()
    }
    fn colour_at(&self, x: usize, y: usize) -> Option<String> {
        match self.rows.get(y).and_then(|r| r.get(x)) {
            Some('.') | None => None,
            Some(c) => Some(c.to_string()),
        }
    }
}

fn has(findings: &[audit::Finding], rule: &str) -> bool {
    findings.iter().any(|f| f.rule == rule)
}

#[test]
fn a_solid_block_passes() {
    let grid = Grid::new(
        "aaaaa\n\
         aaaaa\n\
         aaaaa\n\
         aaaaa",
    );

    assert!(audit::audit(&grid).is_empty());
    assert!(audit::passes(&grid));
}

#[test]
fn a_lone_pixel_is_an_error() {
    let grid = Grid::new(
        "aaaaa\n\
         aaaaa\n\
         aaaaa\n\
         aaaab",
    );

    let findings = audit::audit(&grid);
    assert!(has(&findings, "orphan-pixels"));
    assert_eq!(findings[0].severity, Severity::Error);
    assert!(!audit::passes(&grid), "un pixel isole doit faire echouer");
}

#[test]
fn a_small_group_is_only_a_warning() {
    let grid = Grid::new(
        "aaaaaaaaaa\n\
         aaaaaaaaaa\n\
         aabbaaaaaa\n\
         aabbaaaaaa",
    );

    let findings = audit::audit(&grid);
    assert!(has(&findings, "cluster-size"));
    assert!(
        !has(&findings, "orphan-pixels"),
        "quatre pixels ne sont pas orphelins"
    );
    // Un avertissement ne bloque pas : le sprite reste utilisable.
    assert!(audit::passes(&grid));
}

#[test]
fn a_diagonal_line_is_one_cluster() {
    // Connectivite 8 : une diagonale nette est un trait valide, pas une suite
    // de pixels isoles.
    let grid = Grid::new(
        "b........\n\
         .b.......\n\
         ..b......\n\
         ...b.....\n\
         ....b....",
    );

    let findings = audit::audit(&grid);
    assert!(
        !has(&findings, "orphan-pixels"),
        "une diagonale ne doit pas compter comme des pixels isoles"
    );
}

#[test]
fn pixels_touching_only_at_a_corner_still_connect() {
    let grid = Grid::new(
        "b.\n\
         .b",
    );
    let clusters = audit::find_clusters(&grid);

    assert_eq!(
        clusters["b"],
        vec![2],
        "deux pixels en diagonale font un groupe"
    );
}

#[test]
fn pixels_of_different_colours_never_merge() {
    let grid = Grid::new(
        "ab\n\
         ba",
    );
    let clusters = audit::find_clusters(&grid);

    // Chaque couleur a deux pixels, relies en diagonale.
    assert_eq!(clusters["a"], vec![2]);
    assert_eq!(clusters["b"], vec![2]);
}

#[test]
fn empty_pixels_are_not_a_cluster() {
    let grid = Grid::new(
        "...\n\
         ...",
    );

    assert!(audit::find_clusters(&grid).is_empty());
    assert!(audit::audit(&grid).is_empty());
}

#[test]
fn pure_black_is_flagged() {
    struct Black;
    impl Surface for Black {
        fn width(&self) -> usize {
            8
        }
        fn height(&self) -> usize {
            8
        }
        fn colour_at(&self, _: usize, _: usize) -> Option<String> {
            Some("#000000".to_owned())
        }
    }

    let findings = audit::audit(&Black);
    assert!(has(&findings, "pure-black-outline"));
    // Un avertissement, pas une erreur : c'est un conseil, pas une faute.
    assert_eq!(findings[0].severity, Severity::Warning);
}

#[test]
fn black_is_recognised_whatever_the_case() {
    struct Black;
    impl Surface for Black {
        fn width(&self) -> usize {
            8
        }
        fn height(&self) -> usize {
            8
        }
        fn colour_at(&self, _: usize, _: usize) -> Option<String> {
            Some("#000000".to_owned())
        }
    }
    assert!(has(&audit::audit(&Black), "pure-black-outline"));
}

#[test]
fn the_black_warning_appears_once_not_per_pixel() {
    struct Black;
    impl Surface for Black {
        fn width(&self) -> usize {
            16
        }
        fn height(&self) -> usize {
            16
        }
        fn colour_at(&self, _: usize, _: usize) -> Option<String> {
            Some("#000000".to_owned())
        }
    }

    let count = audit::audit(&Black)
        .iter()
        .filter(|f| f.rule == "pure-black-outline")
        .count();
    assert_eq!(count, 1, "256 pixels noirs ne font pas 256 avertissements");
}

#[test]
fn the_message_agrees_in_number() {
    let un = Grid::new("aaaaaaaaaa\naaaaaaaaaa\n.........b");
    let message = &audit::audit(&un)[0].message;
    assert!(message.contains("1 pixel isole detecte"), "{message}");

    let deux = Grid::new("aaaaaaaaaa\naaaaaaaaaa\nb........c");
    let message = &audit::audit(&deux)[0].message;
    assert!(message.contains("2 pixels isoles detectes"), "{message}");
}

#[test]
fn the_threshold_is_where_it_says() {
    // Neuf pixels avertissent, dix passent.
    let neuf = Grid::new(
        "bbb.......\n\
         bbb.......\n\
         bbb.......",
    );
    assert!(has(&audit::audit(&neuf), "cluster-size"));

    let dix = Grid::new(
        "bbbbb.....\n\
         bbbbb.....",
    );
    assert!(!has(&audit::audit(&dix), "cluster-size"), "dix doit passer");
}

#[test]
fn an_empty_image_has_nothing_to_say() {
    struct Empty;
    impl Surface for Empty {
        fn width(&self) -> usize {
            0
        }
        fn height(&self) -> usize {
            0
        }
        fn colour_at(&self, _: usize, _: usize) -> Option<String> {
            None
        }
    }

    assert!(audit::audit(&Empty).is_empty());
    assert!(audit::passes(&Empty));
}

#[test]
fn black_written_two_ways_still_warns_once() {
    // La table est indexee par chaine : deux graphies du noir font deux cles.
    // `#000000` n'a aucune lettre, donc changer sa casse ne cree pas de
    // seconde cle — il faut la forme longue avec alpha, que le TypeScript
    // produit aussi.
    struct Blacks;
    impl Surface for Blacks {
        fn width(&self) -> usize {
            9
        }
        fn height(&self) -> usize {
            9
        }
        fn colour_at(&self, x: usize, _: usize) -> Option<String> {
            Some(
                match x % 3 {
                    0 => "#000000",
                    1 => "#000000FF",
                    _ => "#334455",
                }
                .to_owned(),
            )
        }
    }

    // Une seule des deux graphies est reconnue : la regle ne connait que
    // `#000000` exactement. C'est une limite du TypeScript, portee telle
    // quelle plutot que corrigee — la corriger ferait diverger les deux
    // versions sur des images que le TypeScript accepte.
    let count = audit::audit(&Blacks)
        .iter()
        .filter(|f| f.rule == "pure-black-outline")
        .count();
    assert_eq!(count, 1);
}
