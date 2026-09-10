use raster_editor::keys::{self, Editor};
use raster_input::{Input, Key};

/// Une entree avec les touches deja enfoncees : `begin_frame` echantillonne
/// l'etat courant, donc l'ordre compte.
fn frame(keys: &[Key]) -> Input {
    let mut input = Input::new(keys::bindings());
    for key in keys {
        input.key_down(*key);
    }
    input.begin_frame(1.0 / 60.0);
    input
}

#[test]
fn ctrl_z_undoes() {
    let shortcuts = keys::read(&frame(&[Key::Control, Key::Z]));

    assert!(shortcuts.undo);
    assert!(
        !shortcuts.redo,
        "annuler et refaire ne doivent pas partir ensemble"
    );
}

#[test]
fn z_alone_does_nothing() {
    // Sans modificateur, Z n'est pas un raccourci : ce serait une catastrophe
    // dans un champ de saisie.
    let shortcuts = keys::read(&frame(&[Key::Z]));

    assert!(!shortcuts.undo);
    assert!(!shortcuts.redo);
}

#[test]
fn ctrl_y_redoes() {
    let shortcuts = keys::read(&frame(&[Key::Control, Key::Y]));

    assert!(shortcuts.redo);
    assert!(!shortcuts.undo);
}

#[test]
fn ctrl_shift_z_redoes_and_never_undoes() {
    // Les deux conventions coexistent : Ctrl+Y et Ctrl+Maj+Z refont tous deux.
    let shortcuts = keys::read(&frame(&[Key::Control, Key::Shift, Key::Z]));

    assert!(shortcuts.redo, "Ctrl+Maj+Z doit refaire");
    assert!(
        !shortcuts.undo,
        "Ctrl+Maj+Z ne doit surtout pas annuler en meme temps"
    );
}

#[test]
fn ctrl_s_saves() {
    assert!(keys::read(&frame(&[Key::Control, Key::S])).save);
    assert!(
        !keys::read(&frame(&[Key::S])).save,
        "S seul n'enregistre pas"
    );
}

#[test]
fn ctrl_o_opens() {
    assert!(keys::read(&frame(&[Key::Control, Key::O])).open);
}

#[test]
fn ctrl_d_duplicates() {
    assert!(keys::read(&frame(&[Key::Control, Key::D])).duplicate);
}

#[test]
fn delete_removes_without_a_modifier() {
    assert!(keys::read(&frame(&[Key::Delete])).delete);
    assert!(
        keys::read(&frame(&[Key::Backspace])).delete,
        "les deux touches"
    );

    // Avec Ctrl, ce n'est plus une suppression : le raccourci appartient a
    // autre chose.
    assert!(!keys::read(&frame(&[Key::Control, Key::Delete])).delete);
}

#[test]
fn g_and_h_toggle_the_grid() {
    assert!(keys::read(&frame(&[Key::G])).toggle_snap);
    assert!(keys::read(&frame(&[Key::H])).toggle_grid);

    assert!(!keys::read(&frame(&[Key::Control, Key::G])).toggle_snap);
}

#[test]
fn escape_quits_with_or_without_modifiers() {
    assert!(keys::read(&frame(&[Key::Escape])).quit);
    assert!(keys::read(&frame(&[Key::Control, Key::Escape])).quit);
}

#[test]
fn nothing_pressed_asks_for_nothing() {
    let shortcuts = keys::read(&frame(&[]));

    assert_eq!(shortcuts, keys::Shortcuts::default());
}

#[test]
fn holding_the_modifier_alone_does_nothing() {
    let shortcuts = keys::read(&frame(&[Key::Control]));

    assert_eq!(shortcuts, keys::Shortcuts::default());
}

#[test]
fn the_editor_has_its_own_action_names() {
    // Distinctes de celles du jeu : c'est ce que la friction 6 demandait.
    assert_ne!(Editor::UNDO, raster_input::Action::JUMP);
    assert!(Editor::UNDO.0.starts_with("editor."));
    assert!(Editor::SAVE.0.starts_with("editor."));
}

#[test]
fn a_shortcut_fires_once_per_press() {
    let mut input = Input::new(keys::bindings());
    input.key_down(Key::Control);
    input.key_down(Key::S);
    input.begin_frame(1.0 / 60.0);
    assert!(keys::read(&input).save);

    // La touche reste enfoncee : le raccourci ne doit pas se repeter, sinon
    // maintenir Ctrl+S enregistrerait soixante fois par seconde.
    input.begin_frame(1.0 / 60.0);
    assert!(!keys::read(&input).save, "le raccourci s'est repete");
}
