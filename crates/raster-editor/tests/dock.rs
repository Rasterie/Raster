use raster_editor::{Dock, Node};
use raster_math::Rect;
use raster_ui::layout::Axis;

const SCREEN: Rect = Rect {
    position: raster_math::Vec2 { x: 0.0, y: 0.0 },
    size: raster_math::Vec2 {
        x: 1280.0,
        y: 800.0,
    },
};

/// La disposition type d'un editeur.
fn shell() -> Dock {
    Dock::new(Node::split(
        Axis::Horizontal,
        0.2,
        Node::tabs(&["assets"]),
        Node::split(
            Axis::Horizontal,
            0.75,
            Node::split(
                Axis::Vertical,
                0.8,
                Node::tabs(&["viewport"]),
                Node::tabs(&["console", "sortie"]),
            ),
            Node::tabs(&["inspecteur"]),
        ),
    ))
}

#[test]
fn every_panel_gets_a_place() {
    let places = shell().layout(SCREEN);
    let noms: Vec<&str> = places.iter().map(|p| p.panel.as_str()).collect();

    // La console et la sortie partagent un groupe : seul l'actif est place.
    assert_eq!(noms, ["assets", "viewport", "console", "inspecteur"]);
}

#[test]
fn panels_never_overlap() {
    let places = shell().layout(SCREEN);

    for (i, a) in places.iter().enumerate() {
        for b in places.iter().skip(i + 1) {
            assert!(
                !a.area.intersects(b.area),
                "`{}` et `{}` se chevauchent : {:?} et {:?}",
                a.panel,
                b.panel,
                a.area,
                b.area
            );
        }
    }
}

#[test]
fn everything_stays_inside_the_screen() {
    for place in shell().layout(SCREEN) {
        assert!(
            place.area.position.x >= 0.0
                && place.area.position.y >= 0.0
                && place.area.position.x + place.area.size.x <= SCREEN.size.x + 0.01
                && place.area.position.y + place.area.size.y <= SCREEN.size.y + 0.01,
            "`{}` sort de l'ecran : {:?}",
            place.panel,
            place.area
        );
    }
}

#[test]
fn a_ratio_divides_in_proportion() {
    let dock = Dock::new(Node::tabs(&["a"]));
    let (gauche, droite) = dock.divide(SCREEN, Axis::Horizontal, 0.25);

    let utilisable = SCREEN.size.x - dock.handle;
    assert!((gauche.size.x - utilisable * 0.25).abs() < 0.01);
    assert!((droite.size.x - utilisable * 0.75).abs() < 0.01);
}

#[test]
fn the_handle_sits_between_the_two_halves() {
    let dock = Dock::new(Node::tabs(&["a"]));
    let (gauche, droite) = dock.divide(SCREEN, Axis::Horizontal, 0.5);
    let poignee = dock.handle_area(SCREEN, Axis::Horizontal, 0.5);

    assert!((poignee.position.x - (gauche.position.x + gauche.size.x)).abs() < 0.01);
    assert!((poignee.position.x + poignee.size.x - droite.position.x).abs() < 0.01);
    assert_eq!(poignee.size.x, dock.handle);
}

#[test]
fn a_lone_panel_has_no_tab_bar() {
    let places = Dock::new(Node::tabs(&["seul"])).layout(SCREEN);

    assert_eq!(places[0].tab_bar, None, "un panneau seul n'a pas d'onglets");
    assert_eq!(places[0].area, SCREEN, "il occupe tout");
}

#[test]
fn tabbed_panels_leave_room_for_their_bar() {
    let dock = Dock::new(Node::tabs(&["console", "sortie"]));
    let places = dock.layout(SCREEN);

    let bar = places[0].tab_bar.expect("deux onglets font une barre");
    assert_eq!(bar.size.y, dock.tab_height);
    assert_eq!(places[0].area.size.y, SCREEN.size.y - dock.tab_height);
    assert_eq!(places[0].siblings, ["console", "sortie"]);
}

#[test]
fn the_active_tab_is_the_one_placed() {
    let dock = Dock::new(Node::Tabs {
        panels: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        active: 2,
    });

    assert_eq!(dock.layout(SCREEN)[0].panel, "c");
}

#[test]
fn closing_a_tab_keeps_its_siblings() {
    let mut dock = Dock::new(Node::tabs(&["console", "sortie"]));

    assert!(dock.close("console"));
    assert_eq!(dock.panels(), ["sortie"]);
}

#[test]
fn closing_the_last_tab_of_a_split_collapses_it() {
    let mut dock = shell();

    // L'inspecteur ferme : son voisin doit remonter et prendre la place.
    assert!(dock.close("inspecteur"));
    assert!(!dock.panels().contains(&"inspecteur".to_owned()));

    // Rien ne doit se chevaucher apres le repli.
    let places = dock.layout(SCREEN);
    for (i, a) in places.iter().enumerate() {
        for b in places.iter().skip(i + 1) {
            assert!(!a.area.intersects(b.area), "{} et {}", a.panel, b.panel);
        }
    }
}

#[test]
fn closing_the_last_panel_is_refused() {
    let mut dock = Dock::new(Node::tabs(&["seul"]));

    // Un ecran vide n'est pas un etat utile.
    assert!(!dock.close("seul"));
    assert_eq!(dock.panels(), ["seul"]);
}

#[test]
fn closing_something_absent_changes_nothing() {
    let mut dock = shell();
    let avant = dock.panels();

    assert!(!dock.close("inexistant"));
    assert_eq!(dock.panels(), avant);
}

#[test]
fn a_ratio_is_kept_within_bounds() {
    // Un ratio de zero ferait disparaitre un panneau sans moyen de le reprendre.
    let node = Node::split(
        Axis::Horizontal,
        0.0,
        Node::tabs(&["a"]),
        Node::tabs(&["b"]),
    );

    if let Node::Split { ratio, .. } = node {
        assert!(ratio > 0.0, "le ratio doit rester utilisable : {ratio}");
    } else {
        panic!("attendu un Split");
    }
}

#[test]
fn a_node_knows_what_it_holds() {
    let dock = shell();

    assert!(dock.root.contains("viewport"));
    assert!(dock.root.contains("console"));
    assert!(!dock.root.contains("inexistant"));
    assert_eq!(dock.panels().len(), 5);
}

#[test]
fn a_zero_sized_screen_produces_no_negative_areas() {
    let places = shell().layout(Rect::new(0.0, 0.0, 2.0, 2.0));

    for place in places {
        assert!(
            place.area.size.x >= 0.0 && place.area.size.y >= 0.0,
            "`{}` a une taille negative : {:?}",
            place.panel,
            place.area
        );
    }
}

#[test]
fn a_panel_never_appears_twice() {
    // Un repli qui duplique laisserait deux panneaux identiques, chacun
    // croyant etre le vrai.
    let mut dock = shell();
    dock.close("inspecteur");

    let mut vus = dock.panels();
    vus.sort();
    let avant = vus.len();
    vus.dedup();

    assert_eq!(vus.len(), avant, "un panneau apparait en double : {vus:?}");
}

#[test]
fn closing_removes_exactly_one_panel() {
    let mut dock = shell();
    let avant = dock.panels().len();

    dock.close("console");

    assert_eq!(
        dock.panels().len(),
        avant - 1,
        "fermer doit retirer un panneau, pas plus ni moins"
    );
}
