use raster_core::asset::{AssetId, Project};
use raster_editor::browser::{Browser, Kind};

/// Un projet jetable avec quelques assets.
struct Bac {
    root: std::path::PathBuf,
}

impl Bac {
    fn new(nom: &str) -> Self {
        let root = std::env::temp_dir().join(format!("raster-browser-{nom}"));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(Project::MARKER), "").unwrap();

        for (chemin, contenu) in [
            ("sprites/hero.png", "x"),
            ("sprites/tiles.png", "x"),
            ("sprites/ui/button.png", "x"),
            ("levels/forest.scene.toml", "x"),
            ("sounds/jump.wav", "x"),
            ("notes.txt", "x"),
        ] {
            let p = root.join(chemin);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, contenu).unwrap();
        }

        // Ce qui ne doit pas paraitre dans un navigateur d'assets.
        std::fs::create_dir_all(root.join("target/debug")).unwrap();
        std::fs::write(root.join("target/debug/binaire"), "x").unwrap();
        std::fs::write(root.join(".gitignore"), "x").unwrap();

        Self { root }
    }

    fn project(&self) -> Project {
        Project::new(&self.root)
    }
}

impl Drop for Bac {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

fn browser(bac: &Bac) -> Browser {
    let mut b = Browser::new();
    b.scan(&bac.project()).unwrap();
    b
}

#[test]
fn scanning_finds_every_asset() {
    let bac = Bac::new("scan");
    let b = browser(&bac);

    let chemins: Vec<&str> = b.visible().iter().map(|e| e.id.as_str()).collect();

    assert!(chemins.contains(&"sprites/hero.png"));
    assert!(chemins.contains(&"levels/forest.scene.toml"));
    assert!(chemins.contains(&"sounds/jump.wav"));
    assert!(
        chemins.contains(&"notes.txt"),
        "un fichier inconnu se montre"
    );
}

#[test]
fn service_files_never_appear() {
    let bac = Bac::new("skip");
    let b = browser(&bac);
    let chemins: Vec<&str> = b.visible().iter().map(|e| e.id.as_str()).collect();

    assert!(
        !chemins.iter().any(|c| c.starts_with("target")),
        "target/ n'a rien a faire dans un navigateur d'assets"
    );
    assert!(!chemins.iter().any(|c| c.contains(".gitignore")));
    assert!(!chemins.iter().any(|c| c.contains(Project::MARKER)));
}

#[test]
fn each_extension_gets_its_kind() {
    assert_eq!(Kind::of(&AssetId::new("a/hero.png")), Kind::Sprite);
    assert_eq!(Kind::of(&AssetId::new("a/level.scene.toml")), Kind::Scene);
    assert_eq!(Kind::of(&AssetId::new("a/jump.wav")), Kind::Sound);
    assert_eq!(Kind::of(&AssetId::new("a/notes.txt")), Kind::Other);

    // Un TOML qui n'est pas une scene reste un fichier ordinaire.
    assert_eq!(Kind::of(&AssetId::new("raster.toml")), Kind::Other);
}

#[test]
fn folders_are_listed_with_their_depth() {
    let bac = Bac::new("depth");
    let b = browser(&bac);

    let ui = b
        .visible()
        .into_iter()
        .find(|e| e.id.as_str() == "sprites/ui")
        .expect("le dossier imbrique doit paraitre");

    assert_eq!(ui.kind, Kind::Folder);
    assert_eq!(ui.depth, 1, "un dossier dans un dossier");

    let bouton = b
        .visible()
        .into_iter()
        .find(|e| e.id.as_str() == "sprites/ui/button.png")
        .unwrap();
    assert_eq!(bouton.depth, 2);
}

#[test]
fn searching_narrows_the_list() {
    let bac = Bac::new("search");
    let mut b = browser(&bac);

    b.search("hero");
    let trouves: Vec<&str> = b.visible().iter().map(|e| e.id.as_str()).collect();

    assert_eq!(trouves, ["sprites/hero.png"]);
}

#[test]
fn searching_ignores_case() {
    let bac = Bac::new("case");
    let mut b = browser(&bac);

    b.search("HERO");
    assert_eq!(b.visible().len(), 1, "personne ne tape les majuscules");
}

#[test]
fn clearing_the_search_shows_everything_again() {
    let bac = Bac::new("clear");
    let mut b = browser(&bac);
    let total = b.visible().len();

    b.search("hero");
    assert!(b.visible().len() < total);

    b.search("   ");
    assert_eq!(b.visible().len(), total, "des espaces ne filtrent rien");
    assert_eq!(b.filter(), None);
}

#[test]
fn collapsing_a_folder_hides_what_it_holds() {
    let bac = Bac::new("collapse");
    let mut b = browser(&bac);
    let avant = b.visible().len();

    b.toggle(&AssetId::new("sprites"));
    let apres: Vec<&str> = b.visible().iter().map(|e| e.id.as_str()).collect();

    assert!(apres.contains(&"sprites"), "le dossier reste visible");
    assert!(
        !apres.contains(&"sprites/hero.png"),
        "son contenu est cache"
    );
    assert!(
        !apres.contains(&"sprites/ui/button.png"),
        "et les sous-dossiers"
    );
    assert!(apres.len() < avant);
    assert!(b.is_collapsed(&AssetId::new("sprites")));
}

#[test]
fn unfolding_brings_the_contents_back() {
    let bac = Bac::new("unfold");
    let mut b = browser(&bac);
    let avant = b.visible().len();

    b.toggle(&AssetId::new("sprites"));
    b.toggle(&AssetId::new("sprites"));

    assert_eq!(b.visible().len(), avant);
    assert!(!b.is_collapsed(&AssetId::new("sprites")));
}

#[test]
fn a_search_reaches_inside_collapsed_folders() {
    let bac = Bac::new("search-collapsed");
    let mut b = browser(&bac);

    b.toggle(&AssetId::new("sprites"));
    b.search("button");

    // Chercher sans trouver ce qui est replie serait absurde.
    let trouves: Vec<&str> = b.visible().iter().map(|e| e.id.as_str()).collect();
    assert_eq!(trouves, ["sprites/ui/button.png"]);
}

#[test]
fn assets_can_be_listed_by_kind() {
    let bac = Bac::new("kind");
    let b = browser(&bac);

    let sprites: Vec<&str> = b
        .of_kind(Kind::Sprite)
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(sprites.len(), 3);
    assert!(sprites.contains(&"sprites/ui/button.png"));

    assert_eq!(b.of_kind(Kind::Scene).len(), 1);
    assert_eq!(b.of_kind(Kind::Sound).len(), 1);
}

#[test]
fn an_empty_project_lists_nothing() {
    let root = std::env::temp_dir().join("raster-browser-vide");
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(Project::MARKER), "").unwrap();

    let mut b = Browser::new();
    b.scan(&Project::new(&root)).unwrap();

    assert!(b.is_empty());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn scanning_twice_does_not_duplicate() {
    let bac = Bac::new("twice");
    let mut b = browser(&bac);
    let avant = b.len();

    b.scan(&bac.project()).unwrap();
    assert_eq!(b.len(), avant, "un rescan a duplique les entrees");
}
