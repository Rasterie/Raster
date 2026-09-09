use raster_editor::{Command, History, Target};

/// Un monde jouet : une liste de nombres.
#[derive(Debug, Default)]
struct Scene {
    values: Vec<i32>,
}

impl Target for Scene {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Ajoute une valeur. Ne fusionne jamais : deux ajouts sont deux gestes.
#[derive(Debug)]
struct Add(i32);

impl Command for Add {
    fn apply(&mut self, world: &mut dyn Target) {
        scene(world).values.push(self.0);
    }
    fn revert(&mut self, world: &mut dyn Target) {
        scene(world).values.pop();
    }
    fn label(&self) -> String {
        format!("Ajoute {}", self.0)
    }
}

/// Deplace le dernier element. Fusionne : un glissement est une seule entree.
#[derive(Debug)]
struct Move {
    from: i32,
    to: i32,
}

impl Command for Move {
    fn apply(&mut self, world: &mut dyn Target) {
        if let Some(last) = scene(world).values.last_mut() {
            *last = self.to;
        }
    }
    fn revert(&mut self, world: &mut dyn Target) {
        if let Some(last) = scene(world).values.last_mut() {
            *last = self.from;
        }
    }
    fn label(&self) -> String {
        format!("Deplace vers {}", self.to)
    }
    fn merge(&mut self, other: &dyn Command) -> bool {
        // Absorbe la destination du suivant, en gardant l'origine du premier.
        if let Some(next) = downcast_move(other) {
            self.to = next.to;
            return true;
        }
        false
    }
}

/// `dyn Command` n'est pas `Any` : on reconnait par l'etiquette, ce qui suffit
/// pour un test.
fn downcast_move(command: &dyn Command) -> Option<Move> {
    let label = command.label();
    let value = label.strip_prefix("Deplace vers ")?;
    Some(Move {
        from: 0,
        to: value.parse().ok()?,
    })
}

/// La commande retrouve sa cible par transtypage, sans `unsafe`.
fn scene(world: &mut dyn Target) -> &mut Scene {
    raster_editor::target_as::<Scene>(world).expect("la cible est une Scene")
}

#[test]
fn a_command_applies_when_pushed() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);

    assert_eq!(scene.values, [1]);
    assert_eq!(history.depth(), 1);
}

#[test]
fn undoing_puts_things_back() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    history.push(Box::new(Add(2)), &mut scene);
    assert_eq!(scene.values, [1, 2]);

    assert!(history.undo(&mut scene));
    assert_eq!(scene.values, [1]);
    assert!(history.undo(&mut scene));
    assert_eq!(scene.values, [] as [i32; 0]);
}

#[test]
fn undoing_an_empty_history_does_nothing() {
    let mut scene = Scene::default();
    let mut history = History::new();

    assert!(!history.undo(&mut scene));
    assert!(!history.can_undo());
}

#[test]
fn redoing_reapplies_what_was_undone() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    history.undo(&mut scene);
    assert!(history.can_redo());

    assert!(history.redo(&mut scene));
    assert_eq!(scene.values, [1]);
    assert!(!history.can_redo());
}

#[test]
fn a_new_command_erases_what_could_be_redone() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    history.undo(&mut scene);
    assert!(history.can_redo());

    // Refaire apres une nouvelle action n'aurait pas de sens.
    history.push(Box::new(Add(9)), &mut scene);
    assert!(!history.can_redo());
    assert_eq!(scene.values, [9]);
}

#[test]
fn a_drag_collapses_into_one_entry() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(0)), &mut scene);
    // Un glissement : trois pas rapproches.
    for to in [10, 20, 30] {
        history.push(Box::new(Move { from: 0, to }), &mut scene);
    }

    assert_eq!(scene.values, [30]);
    assert_eq!(
        history.depth(),
        2,
        "le glissement doit tenir en une entree, pas trois"
    );

    // Une seule annulation remet a l'origine du glissement.
    history.undo(&mut scene);
    assert_eq!(scene.values, [0]);
}

#[test]
fn breaking_the_window_separates_two_gestures() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(0)), &mut scene);
    history.push(Box::new(Move { from: 0, to: 10 }), &mut scene);
    history.break_merge();
    history.push(Box::new(Move { from: 10, to: 20 }), &mut scene);

    assert_eq!(
        history.depth(),
        3,
        "deux gestes distincts font deux entrees"
    );
}

#[test]
fn commands_that_do_not_merge_never_collapse() {
    let mut scene = Scene::default();
    let mut history = History::new();

    for i in 0..3 {
        history.push(Box::new(Add(i)), &mut scene);
    }

    assert_eq!(history.depth(), 3);
}

#[test]
fn the_history_forgets_the_oldest_beyond_its_limit() {
    let mut scene = Scene::default();
    let mut history = History::new().with_limit(3);

    for i in 0..5 {
        history.push(Box::new(Add(i)), &mut scene);
    }

    assert_eq!(history.depth(), 3, "la limite doit etre respectee");
    // Les deux plus anciennes sont parties, les trois recentes restent.
    assert_eq!(history.labels(), ["Ajoute 2", "Ajoute 3", "Ajoute 4"]);
}

#[test]
fn labels_say_what_would_be_undone() {
    let mut scene = Scene::default();
    let mut history = History::new();

    assert_eq!(history.undo_label(), None);

    history.push(Box::new(Add(7)), &mut scene);
    assert_eq!(history.undo_label().as_deref(), Some("Ajoute 7"));

    history.undo(&mut scene);
    assert_eq!(history.redo_label().as_deref(), Some("Ajoute 7"));
}

#[test]
fn a_fresh_history_is_clean() {
    assert!(!History::new().is_dirty());
}

#[test]
fn a_change_makes_the_project_dirty() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    assert!(history.is_dirty());

    history.mark_saved();
    assert!(!history.is_dirty());
}

#[test]
fn undoing_back_to_the_saved_state_is_clean_again() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    history.mark_saved();

    history.push(Box::new(Add(2)), &mut scene);
    assert!(history.is_dirty());

    // Revenir a l'etat enregistre doit rendre le projet propre.
    history.undo(&mut scene);
    assert!(!history.is_dirty(), "annuler jusqu'a l'enregistrement");
}

#[test]
fn clearing_resets_everything() {
    let mut scene = Scene::default();
    let mut history = History::new();

    history.push(Box::new(Add(1)), &mut scene);
    history.clear();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert!(!history.is_dirty());
}

#[test]
fn the_saved_mark_follows_what_the_limit_forgets() {
    let mut scene = Scene::default();
    let mut history = History::new().with_limit(3);

    history.push(Box::new(Add(1)), &mut scene);
    history.mark_saved();

    // Deborde la limite : la plus ancienne est oubliee, et la marque doit
    // suivre, sinon elle designerait la mauvaise entree.
    for i in 2..=5 {
        history.push(Box::new(Add(i)), &mut scene);
    }

    assert_eq!(history.depth(), 3);
    assert!(history.is_dirty(), "quatre ajouts depuis l'enregistrement");

    // La divergence est ici : en annulant tout, la marque doit avoir suivi
    // l'oubli. Sinon elle designe une entree qui n'existe plus, et le projet
    // se croit enregistre une entree trop tot.
    let mut proprete = Vec::new();
    while history.can_undo() {
        history.undo(&mut scene);
        proprete.push((history.depth(), history.is_dirty()));
    }

    assert_eq!(
        proprete,
        [(2, true), (1, true), (0, false)],
        "la marque n'a pas suivi ce que la limite a oublie"
    );
}
