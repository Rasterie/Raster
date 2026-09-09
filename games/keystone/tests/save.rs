use keystone::rules::{MAX_HEALTH, Progress};
use keystone::save::{self, SaveError};

#[test]
fn progress_survives_a_round_trip() {
    let mut avant = Progress::new();
    avant.health = 2;
    avant.keys = 1;
    avant.room = 3;
    avant.cleared = vec![0, 1, 2];

    let apres = save::from_toml(&save::to_toml(&avant)).unwrap();
    assert_eq!(apres, avant);
}

#[test]
fn a_fresh_save_reads_back_as_a_fresh_start() {
    let neuf = Progress::new();
    assert_eq!(save::from_toml(&save::to_toml(&neuf)).unwrap(), neuf);
}

#[test]
fn a_save_is_readable_by_a_human() {
    let mut progress = Progress::new();
    progress.room = 2;
    progress.cleared = vec![0, 1];

    let texte = save::to_toml(&progress);
    assert!(texte.contains("room = 2"), "{texte}");
    assert!(texte.contains("cleared = [0, 1]"), "{texte}");
}

#[test]
fn a_tampered_save_cannot_grant_more_hearts_than_the_game_has() {
    let trafique = "[progress]\nhealth = 999\nkeys = 0\nroom = 0\ncleared = []\n";
    let progress = save::from_toml(trafique).unwrap();

    assert_eq!(progress.health, MAX_HEALTH, "un fichier edite a donne l'invincibilite");
}

#[test]
fn an_empty_save_is_refused() {
    assert!(matches!(save::from_toml(""), Err(SaveError::Malformed(_))));
}

#[test]
fn a_save_with_a_bad_number_says_which_field() {
    match save::from_toml("[progress]\nhealth = beaucoup\n") {
        Err(SaveError::Malformed(what)) => assert!(what.contains("health"), "{what}"),
        other => panic!("attendu Malformed, obtenu {other:?}"),
    }
}

#[test]
fn a_save_with_a_bad_room_list_is_refused() {
    let mauvais = "[progress]\nhealth = 3\ncleared = [0, deux]\n";
    assert!(matches!(save::from_toml(mauvais), Err(SaveError::Malformed(_))));
}

#[test]
fn a_missing_field_keeps_its_default() {
    // Une sauvegarde d'une version anterieure reste chargeable.
    let partiel = "[progress]\nroom = 4\n";
    let progress = save::from_toml(partiel).unwrap();

    assert_eq!(progress.room, 4);
    assert_eq!(progress.health, MAX_HEALTH);
    assert!(progress.cleared.is_empty());
}

#[test]
fn a_save_survives_a_trip_through_a_file() {
    let path = std::env::temp_dir().join("keystone-test-save.toml");
    let mut progress = Progress::new();
    progress.room = 2;
    progress.keys = 1;

    save::save(&progress, &path).unwrap();
    assert_eq!(save::load(&path).unwrap(), progress);

    std::fs::remove_file(&path).ok();
}

#[test]
fn a_missing_save_is_an_io_error() {
    assert!(matches!(
        save::load("/keystone/does/not/exist.toml"),
        Err(SaveError::Io(_))
    ));
}
