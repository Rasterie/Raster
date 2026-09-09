use raster_core::asset::{AssetError, AssetId, AssetStore, Project};
use raster_core::reflect::{Reflect, TypeRegistry, Value, ValueKind};
use raster_core::{Scene, World};

#[test]
fn two_spellings_of_one_path_are_one_asset() {
    assert_eq!(
        AssetId::new("sprites/hero.png"),
        AssetId::new("./sprites/hero.png")
    );
    assert_eq!(
        AssetId::new("sprites/hero.png"),
        AssetId::new("sprites//hero.png")
    );
    assert_eq!(
        AssetId::new("sprites/hero.png"),
        AssetId::new("sprites\\hero.png")
    );
    assert_eq!(
        AssetId::new("sprites/hero.png"),
        AssetId::new("tiles/../sprites/hero.png")
    );
}

#[test]
fn different_paths_stay_different() {
    assert_ne!(AssetId::new("a/hero.png"), AssetId::new("b/hero.png"));
    assert_ne!(AssetId::new("hero.png"), AssetId::new("Hero.png"));
}

#[test]
fn an_identifier_exposes_its_name_and_extension() {
    let id = AssetId::new("sprites/hero.PNG");
    assert_eq!(id.file_name(), "hero.PNG");
    assert_eq!(id.extension().as_deref(), Some("png"));
    assert_eq!(id.as_str(), "sprites/hero.PNG");

    assert_eq!(AssetId::new("noext").extension(), None);
    assert_eq!(AssetId::new(".hidden").extension(), None);
    assert_eq!(AssetId::new("a/.hidden").extension(), None);
}

#[test]
fn a_path_that_climbs_past_the_root_keeps_its_dots() {
    // Normaliser un `..` en trop le rendrait invisible : il doit rester
    // visible pour que `resolve` puisse le refuser.
    assert_eq!(AssetId::new("../secret").as_str(), "../secret");
    assert_eq!(AssetId::new("a/../../secret").as_str(), "../secret");
}

#[test]
fn an_absolute_path_is_never_an_asset_of_the_project() {
    // Le `/` de tete disparait a la normalisation : sans traitement, un chemin
    // absolu deviendrait un asset du projet au lieu d'etre refuse.
    for absolu in [
        "/etc/passwd",
        r"\\serveur\partage",
        r"C:\Windows\notepad.exe",
    ] {
        assert!(
            AssetId::new(absolu).as_str().starts_with(".."),
            "`{absolu}` a ete pris pour un chemin du projet"
        );
    }
}

#[test]
fn a_project_resolves_an_identifier_under_its_root() {
    let project = Project::new("/games/demo");
    let path = project.resolve(&AssetId::new("sprites/hero.png")).unwrap();

    assert_eq!(path, std::path::Path::new("/games/demo/sprites/hero.png"));
}

#[test]
fn a_project_refuses_to_look_outside_itself() {
    let project = Project::new("/games/demo");

    assert!(matches!(
        project.resolve(&AssetId::new("../../etc/passwd")),
        Err(AssetError::Escapes(_))
    ));
    assert!(matches!(
        project.resolve(&AssetId::new("/etc/passwd")),
        Err(AssetError::Escapes(_))
    ));
    assert!(matches!(
        project.resolve(&AssetId::default()),
        Err(AssetError::Empty)
    ));
}

#[test]
fn a_project_names_a_path_it_contains() {
    let project = Project::new("/games/demo");
    let id = project.id_of("/games/demo/sprites/hero.png").unwrap();

    assert_eq!(id, AssetId::new("sprites/hero.png"));
    assert_eq!(project.id_of("/elsewhere/hero.png"), None);
}

#[test]
fn a_store_loads_each_asset_once() {
    let mut store: AssetStore<String> = AssetStore::new();
    let mut loads = 0;

    let id = AssetId::new("hero.png");
    let first = store
        .load_with(&id, |_| {
            loads += 1;
            Ok::<_, ()>("pixels".to_owned())
        })
        .unwrap();

    let second = store
        .load_with(&AssetId::new("./hero.png"), |_| {
            loads += 1;
            Ok::<_, ()>("pixels".to_owned())
        })
        .unwrap();

    assert_eq!(loads, 1, "le second chargement aurait du venir du cache");
    assert_eq!(first, second);
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(first).unwrap(), "pixels");
}

#[test]
fn a_failed_load_leaves_nothing_behind() {
    let mut store: AssetStore<String> = AssetStore::new();
    let id = AssetId::new("missing.png");

    assert!(
        store
            .load_with(&id, |_| Err::<String, _>("absent"))
            .is_err()
    );
    assert!(!store.contains(&id));

    let handle = store
        .load_with(&id, |_| Ok::<_, ()>("enfin".to_owned()))
        .unwrap();
    assert_eq!(store.get(handle).unwrap(), "enfin");
}

#[test]
fn reloading_an_asset_keeps_existing_handles_valid() {
    let mut store: AssetStore<String> = AssetStore::new();
    let id = AssetId::new("hero.png");

    let handle = store.insert(id.clone(), "avant".to_owned());
    store.insert(id.clone(), "apres".to_owned());

    assert_eq!(store.len(), 1, "le rechargement a duplique l'asset");
    assert_eq!(
        store.get(handle).unwrap(),
        "apres",
        "un handle existant pointe encore vers l'ancienne version"
    );
}

#[test]
fn a_store_remembers_where_each_asset_came_from() {
    let mut store: AssetStore<u32> = AssetStore::new();
    let a = store.insert(AssetId::new("a.png"), 1);
    let b = store.insert(AssetId::new("b.png"), 2);

    assert_eq!(store.id_of(a).unwrap(), &AssetId::new("a.png"));
    assert_eq!(store.id_of(b).unwrap(), &AssetId::new("b.png"));
    assert_eq!(store.by_id(&AssetId::new("b.png")), Some(&2));
    assert_eq!(store.by_id(&AssetId::new("nope.png")), None);
}

#[derive(Reflect, Default, Debug, PartialEq)]
struct Decor {
    texture: AssetId,
    solid: bool,
}

#[test]
fn an_asset_field_is_reflected_as_an_asset_not_a_string() {
    assert_eq!(
        Decor::type_info().field("texture").unwrap().kind,
        ValueKind::Asset
    );
}

#[test]
fn an_asset_field_survives_a_scene_round_trip() {
    let mut registry = TypeRegistry::new();
    registry.register::<Decor>();

    let mut scene = Scene::new("level_1");
    scene.add(&Decor {
        texture: AssetId::new("sprites/hero.png"),
        solid: true,
    });

    let text = scene.to_toml_string();
    assert!(
        text.contains(r#"texture = "sprites/hero.png""#),
        "un asset doit s'ecrire comme son chemin : {text}"
    );

    let read = Scene::parse(&text, &registry).unwrap();
    let mut world = World::new();
    let ids = read.spawn_into(&mut world, &registry).unwrap();

    assert_eq!(
        world.get::<Decor>(ids[0]).unwrap().texture,
        AssetId::new("sprites/hero.png")
    );
}

#[test]
fn a_hand_written_string_still_loads_into_an_asset_field() {
    let mut registry = TypeRegistry::new();
    registry.register::<Decor>();

    let mut world = World::new();
    let value = Value::Struct(
        [(
            "texture".to_owned(),
            Value::Str("sprites/hero.png".to_owned()),
        )]
        .into_iter()
        .collect(),
    );

    let id = registry.spawn_into(&mut world, "Decor", &value).unwrap();
    assert_eq!(
        world.get::<Decor>(id).unwrap().texture,
        AssetId::new("sprites/hero.png")
    );
}

#[test]
fn a_project_finds_its_root_from_a_subfolder() {
    let root = std::env::temp_dir().join("raster-project-test");
    let nested = root.join("levels/forest");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(Project::MARKER), "").unwrap();

    let found = Project::discover(&nested).unwrap();
    assert_eq!(
        found.root().canonicalize().unwrap(),
        root.canonicalize().unwrap()
    );

    assert!(Project::discover(std::env::temp_dir().join("raster-nowhere")).is_none());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_project_reads_an_asset_it_contains() {
    let root = std::env::temp_dir().join("raster-project-read");
    std::fs::create_dir_all(root.join("sprites")).unwrap();
    std::fs::write(root.join("sprites/hero.png"), b"not really a png").unwrap();

    let project = Project::new(&root);
    let id = AssetId::new("sprites/hero.png");

    assert!(project.exists(&id));
    assert_eq!(project.read(&id).unwrap(), b"not really a png");

    let missing = AssetId::new("sprites/absent.png");
    assert!(!project.exists(&missing));
    assert!(matches!(project.read(&missing), Err(AssetError::Io { .. })));

    std::fs::remove_dir_all(&root).ok();
}

/// Un type deliberement non-`Copy` et non-`Clone`, comme une texture GPU.
struct NotCopy(#[allow(dead_code)] u32);

#[test]
fn a_handle_is_copiable_even_when_the_asset_is_not() {
    let mut store: AssetStore<NotCopy> = AssetStore::new();
    let handle = store.insert(AssetId::new("a.png"), NotCopy(1));

    let copie = handle;
    assert_eq!(handle, copie);
    assert!(store.get(copie).is_some());
}

#[test]
fn an_asset_resolves_from_the_project_root_whatever_the_working_directory() {
    let root = std::env::temp_dir().join("raster-project-cwd");
    std::fs::create_dir_all(root.join("levels")).unwrap();
    std::fs::write(root.join(Project::MARKER), "").unwrap();
    std::fs::create_dir_all(root.join("sprites")).unwrap();
    std::fs::write(root.join("sprites/hero.png"), b"pixels").unwrap();

    // Trouve depuis un sous-dossier : c'est ce qui rend l'identifiant d'un
    // asset independant du dossier de lancement.
    let project = Project::discover(root.join("levels")).unwrap();
    assert_eq!(
        project.read(&AssetId::new("sprites/hero.png")).unwrap(),
        b"pixels"
    );

    std::fs::remove_dir_all(&root).ok();
}
