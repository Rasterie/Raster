use raster_editor::dock::{Dock, Node};
use raster_editor::session::Session;
use raster_ui::layout::Axis;

fn shell() -> Dock {
    Dock::new(Node::split(
        Axis::Horizontal,
        0.2,
        Node::tabs(&["Assets"]),
        Node::split(
            Axis::Vertical,
            0.75,
            Node::tabs(&["Scene"]),
            Node::tabs(&["Console", "Sortie"]),
        ),
    ))
}

#[test]
fn a_layout_survives_a_round_trip() {
    let session = Session::new(shell());
    let relu = Session::from_toml(&session.to_toml(), Dock::new(Node::tabs(&["vide"])));

    assert_eq!(
        relu.dock.root, session.dock.root,
        "l'arbre a change en chemin"
    );
}

#[test]
fn every_setting_survives() {
    let mut session = Session::new(shell());
    session.snap = false;
    session.show_grid = false;
    session.zoom = 8;
    session.scene = Some("levels/forest.scene.toml".to_owned());

    let relu = Session::from_toml(&session.to_toml(), shell());

    assert!(!relu.snap);
    assert!(!relu.show_grid);
    assert_eq!(relu.zoom, 8);
    assert_eq!(relu.scene.as_deref(), Some("levels/forest.scene.toml"));
}

#[test]
fn a_deep_tree_survives() {
    // Trois niveaux d'imbrication : c'est la que le decodage se trompe de
    // barre de separation s'il ne compte pas les parentheses.
    let profond = Dock::new(Node::split(
        Axis::Horizontal,
        0.3,
        Node::split(
            Axis::Vertical,
            0.5,
            Node::tabs(&["a"]),
            Node::split(
                Axis::Horizontal,
                0.4,
                Node::tabs(&["b"]),
                Node::tabs(&["c"]),
            ),
        ),
        Node::tabs(&["d", "e"]),
    ));

    let session = Session::new(profond.clone());
    let relu = Session::from_toml(&session.to_toml(), Dock::new(Node::tabs(&["vide"])));

    assert_eq!(relu.dock.root, profond.root);
    assert_eq!(relu.dock.panels(), ["a", "b", "c", "d", "e"]);
}

#[test]
fn the_active_tab_survives() {
    let dock = Dock::new(Node::Tabs {
        panels: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        active: 2,
    });

    let relu = Session::from_toml(&Session::new(dock).to_toml(), Dock::new(Node::tabs(&["x"])));

    match relu.dock.root {
        Node::Tabs { active, .. } => assert_eq!(active, 2),
        Node::Split { .. } => panic!("attendu des onglets"),
    }
}

#[test]
fn a_ratio_survives_close_enough() {
    let dock = Dock::new(Node::split(
        Axis::Horizontal,
        0.375,
        Node::tabs(&["a"]),
        Node::tabs(&["b"]),
    ));

    let relu = Session::from_toml(&Session::new(dock).to_toml(), Dock::new(Node::tabs(&["x"])));

    match relu.dock.root {
        Node::Split { ratio, .. } => assert!((ratio - 0.375).abs() < 0.001, "{ratio}"),
        Node::Tabs { .. } => panic!("attendu une separation"),
    }
}

#[test]
fn a_corrupt_layout_falls_back_to_the_default() {
    // Une session illisible ne doit pas empecher d'ouvrir un projet.
    let defaut = shell();
    let relu = Session::from_toml("[layout]\ntree = \"n'importe quoi\"\n", defaut.clone());

    assert_eq!(relu.dock.root, defaut.root);
}

#[test]
fn a_truncated_tree_falls_back() {
    let defaut = shell();
    for casse in [
        "H:0.5(",
        "H:0.5([a@0]",
        "[a@",
        "[@0]",
        "X:0.5([a@0]|[b@0])",
        "",
    ] {
        let text = format!("[layout]\ntree = {casse:?}\n");
        let relu = Session::from_toml(&text, defaut.clone());
        assert_eq!(relu.dock.root, defaut.root, "`{casse}` n'a pas ete refuse");
    }
}

#[test]
fn an_empty_session_gives_the_defaults() {
    let relu = Session::from_toml("", shell());

    assert!(relu.snap, "l'aimantation est active par defaut");
    assert!(relu.show_grid);
    assert_eq!(relu.scene, None);
    assert_eq!(relu.dock.root, shell().root);
}

#[test]
fn an_out_of_range_zoom_is_clamped() {
    // Un zoom de zero rendrait la vue inutilisable.
    let relu = Session::from_toml("[session]\nzoom = 0\n", shell());
    assert_eq!(relu.zoom, 1);

    let relu = Session::from_toml("[session]\nzoom = 9999\n", shell());
    assert_eq!(relu.zoom, 16);
}

#[test]
fn an_active_tab_beyond_the_list_is_clamped() {
    // Un fichier edite a la main ne doit pas faire pointer dans le vide.
    let relu = Session::from_toml("[layout]\ntree = \"[a,b@7]\"\n", shell());

    match relu.dock.root {
        Node::Tabs { active, panels } => {
            assert!(active < panels.len(), "l'onglet actif sort de la liste");
        }
        Node::Split { .. } => panic!("attendu des onglets"),
    }
}

#[test]
fn a_session_survives_a_trip_through_a_file() {
    let root = std::env::temp_dir().join("raster-session-test");
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(&root).unwrap();

    let mut session = Session::new(shell());
    session.zoom = 4;
    session.scene = Some("a.scene.toml".to_owned());
    session.save(&root).unwrap();

    let relu = Session::load(&root, Dock::new(Node::tabs(&["vide"])));
    assert_eq!(relu.zoom, 4);
    assert_eq!(relu.dock.root, shell().root);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_missing_session_gives_the_default() {
    let relu = Session::load(std::path::Path::new("/raster/aucun/projet"), shell());
    assert_eq!(relu.dock.root, shell().root);
}

#[test]
fn a_session_file_is_readable() {
    let texte = Session::new(shell()).to_toml();

    assert!(texte.contains("snap = true"), "{texte}");
    assert!(texte.contains("[layout]"), "{texte}");
    assert!(
        texte.contains("Assets"),
        "les noms de panneaux doivent rester lisibles"
    );
}
