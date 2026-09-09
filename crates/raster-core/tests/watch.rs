use raster_core::asset::{AssetId, Project, Watcher};
use std::time::Duration;

/// Un dossier de projet jetable, nettoye a la fin du test.
struct Bac {
    root: std::path::PathBuf,
}

impl Bac {
    fn new(nom: &str) -> Self {
        let root = std::env::temp_dir().join(format!("raster-watch-{nom}"));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(Project::MARKER), "").unwrap();
        Self { root }
    }

    fn project(&self) -> Project {
        Project::new(&self.root)
    }

    fn write(&self, nom: &str, contenu: &[u8]) {
        let path = self.root.join(nom);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contenu).unwrap();
    }

    fn remove(&self, nom: &str) {
        std::fs::remove_file(self.root.join(nom)).unwrap();
    }
}

impl Drop for Bac {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
fn an_untouched_asset_never_reports_a_change() {
    let bac = Bac::new("stable");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));

    assert!(watcher.sweep(&project).is_empty());
    assert!(watcher.sweep(&project).is_empty());
}

#[test]
fn a_rewritten_asset_is_reported_once() {
    let bac = Bac::new("rewrite");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));

    bac.write("hero.png", b"des pixels differents");

    assert_eq!(watcher.sweep(&project), [AssetId::new("hero.png")]);
    assert!(
        watcher.sweep(&project).is_empty(),
        "un changement doit etre signale une seule fois"
    );
}

#[test]
fn a_rewrite_of_the_same_length_is_still_noticed() {
    let bac = Bac::new("same-size");
    bac.write("hero.png", b"aaaa");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));

    // Meme taille : seule la date distingue les deux versions.
    std::thread::sleep(Duration::from_millis(20));
    bac.write("hero.png", b"bbbb");

    assert_eq!(watcher.sweep(&project), [AssetId::new("hero.png")]);
}

#[test]
fn a_rewrite_the_clock_cannot_separate_is_noticed_by_its_size() {
    let bac = Bac::new("fast");
    bac.write("hero.png", b"aa");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));

    // Repose la date d'origine : ce que ferait un systeme de fichiers dont la
    // granularite est trop grossiere pour separer deux enregistrements. Sans
    // la taille dans l'empreinte, le changement passerait inapercu.
    let path = bac.root.join("hero.png");
    let avant = std::fs::metadata(&path).unwrap().modified().unwrap();
    bac.write("hero.png", b"un contenu bien plus long");
    // En ecriture : Windows refuse `set_modified` sur un fichier ouvert en
    // lecture seule.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(avant)
        .unwrap();

    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        avant,
        "la date n'a pas pu etre reposee : le test ne prouverait rien"
    );
    assert_eq!(watcher.sweep(&project), [AssetId::new("hero.png")]);
}

#[test]
fn deleting_an_asset_is_a_change() {
    let bac = Bac::new("delete");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));

    bac.remove("hero.png");
    assert_eq!(watcher.sweep(&project), [AssetId::new("hero.png")]);
}

#[test]
fn an_asset_that_appears_later_is_a_change() {
    let bac = Bac::new("appear");
    let project = bac.project();

    let mut watcher = Watcher::new();
    // Surveiller un fichier absent est permis : il apparaitra comme un
    // changement des qu'il existera.
    watcher.watch(&project, &AssetId::new("hero.png"));
    assert!(watcher.sweep(&project).is_empty());

    bac.write("hero.png", b"pixels");
    assert_eq!(watcher.sweep(&project), [AssetId::new("hero.png")]);
}

#[test]
fn several_changes_come_back_in_a_stable_order() {
    let bac = Bac::new("order");
    for nom in ["c.png", "a.png", "b.png"] {
        bac.write(nom, b"x");
    }

    let project = bac.project();
    let mut watcher = Watcher::new();
    for nom in ["c.png", "a.png", "b.png"] {
        watcher.watch(&project, &AssetId::new(nom));
    }

    for nom in ["c.png", "a.png", "b.png"] {
        bac.write(nom, b"contenu modifie");
    }

    assert_eq!(
        watcher.sweep(&project),
        [
            AssetId::new("a.png"),
            AssetId::new("b.png"),
            AssetId::new("c.png")
        ]
    );
}

#[test]
fn a_sweep_only_runs_once_per_interval() {
    let bac = Bac::new("interval");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::with_interval(Duration::from_secs(3600));
    watcher.watch(&project, &AssetId::new("hero.png"));

    // Le premier appel balaie : rien ne s'est encore ecoule.
    assert!(watcher.changed(&project).is_empty());

    bac.write("hero.png", b"pixels modifies");

    assert!(
        watcher.changed(&project).is_empty(),
        "l'intervalle n'est pas ecoule, aucun balayage ne devrait avoir lieu"
    );
    assert_eq!(
        watcher.sweep(&project),
        [AssetId::new("hero.png")],
        "un balayage force voit le changement"
    );
}

#[test]
fn a_short_interval_lets_the_sweep_run_again() {
    let bac = Bac::new("short");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::with_interval(Duration::from_millis(1));
    watcher.watch(&project, &AssetId::new("hero.png"));
    watcher.changed(&project);

    bac.write("hero.png", b"pixels modifies");
    std::thread::sleep(Duration::from_millis(5));

    assert_eq!(watcher.changed(&project), [AssetId::new("hero.png")]);
}

#[test]
fn an_asset_can_stop_being_watched() {
    let bac = Bac::new("forget");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("hero.png"));
    assert!(watcher.is_watching(&AssetId::new("hero.png")));
    assert_eq!(watcher.len(), 1);

    watcher.forget(&AssetId::new("hero.png"));
    bac.write("hero.png", b"pixels modifies");

    assert!(watcher.sweep(&project).is_empty());
    assert!(watcher.is_empty());
}

#[test]
fn changing_the_interval_keeps_what_is_watched() {
    let bac = Bac::new("reinterval");
    bac.write("hero.png", b"pixels");

    let project = bac.project();
    let mut watcher = Watcher::with_interval(Duration::from_secs(3600));
    watcher.watch(&project, &AssetId::new("hero.png"));
    watcher.changed(&project);

    bac.write("hero.png", b"pixels modifies");
    watcher.set_interval(Duration::from_millis(1));
    std::thread::sleep(Duration::from_millis(5));

    assert_eq!(
        watcher.changed(&project),
        [AssetId::new("hero.png")],
        "changer l'intervalle a perdu l'etat surveille"
    );
    assert_eq!(watcher.interval(), Duration::from_millis(1));
}

#[test]
fn an_asset_outside_the_project_never_reports_a_change() {
    let bac = Bac::new("escape");
    let project = bac.project();

    let mut watcher = Watcher::new();
    watcher.watch(&project, &AssetId::new("../../etc/passwd"));

    assert!(watcher.sweep(&project).is_empty());
}

/// Ce que fait un cache de textures, sans GPU : charger, surveiller,
/// recharger ce qui a change derriere les handles existants.
#[test]
fn a_store_and_a_watcher_reload_an_asset_behind_a_live_handle() {
    use raster_core::asset::AssetStore;

    let bac = Bac::new("store");
    bac.write("hero.png", b"avant");

    let project = bac.project();
    let mut store: AssetStore<Vec<u8>> = AssetStore::new();
    let mut watcher = Watcher::new();
    let id = AssetId::new("hero.png");

    let handle = store
        .load_with(&id, |id| project.read(id))
        .expect("chargement initial");
    watcher.watch(&project, &id);
    assert_eq!(store.get(handle).unwrap(), b"avant");

    bac.write("hero.png", b"apres modification");

    let changed = watcher.sweep(&project);
    assert_eq!(changed, std::slice::from_ref(&id));

    for id in &changed {
        let bytes = project.read(id).unwrap();
        store.insert(id.clone(), bytes);
    }

    assert_eq!(
        store.get(handle).unwrap(),
        b"apres modification",
        "le handle pointe encore vers l'ancienne version"
    );
    assert_eq!(store.len(), 1, "le rechargement a duplique l'asset");
}

#[test]
fn an_asset_that_becomes_unreadable_keeps_its_previous_version() {
    use raster_core::asset::AssetStore;

    let bac = Bac::new("broken");
    bac.write("hero.png", b"valide");

    let project = bac.project();
    let mut store: AssetStore<Vec<u8>> = AssetStore::new();
    let mut watcher = Watcher::new();
    let id = AssetId::new("hero.png");

    let handle = store.load_with(&id, |id| project.read(id)).unwrap();
    watcher.watch(&project, &id);

    bac.remove("hero.png");

    // Un asset supprime est signale, mais son rechargement echoue : l'ancienne
    // version reste a l'ecran plutot que de laisser un trou.
    assert_eq!(watcher.sweep(&project), std::slice::from_ref(&id));
    assert!(project.read(&id).is_err());
    assert_eq!(store.get(handle).unwrap(), b"valide");

    // Et le fichier revenu se recharge normalement.
    bac.write("hero.png", b"revenu");
    assert_eq!(watcher.sweep(&project), std::slice::from_ref(&id));
    store.insert(id.clone(), project.read(&id).unwrap());
    assert_eq!(store.get(handle).unwrap(), b"revenu");
}

#[test]
fn an_asset_missing_at_first_load_is_picked_up_when_it_appears() {
    use raster_core::asset::AssetStore;

    let bac = Bac::new("late");
    let project = bac.project();
    let mut store: AssetStore<Vec<u8>> = AssetStore::new();
    let mut watcher = Watcher::new();
    let id = AssetId::new("hero.png");

    // Le fichier n'existe pas encore : le cache reserve quand meme une entree,
    // sans quoi le handle rendu au jeu ne pourrait jamais etre mis a jour.
    let handle = match store.load_with(&id, |id| project.read(id)) {
        Ok(handle) => handle,
        Err(_) => store.insert(id.clone(), b"damier".to_vec()),
    };
    watcher.watch(&project, &id);
    assert_eq!(store.get(handle).unwrap(), b"damier");

    bac.write("hero.png", b"enfin la");

    assert_eq!(watcher.sweep(&project), std::slice::from_ref(&id));
    store.insert(id.clone(), project.read(&id).unwrap());

    assert_eq!(
        store.get(handle).unwrap(),
        b"enfin la",
        "le handle rendu avant l'apparition du fichier n'a pas suivi"
    );
}
