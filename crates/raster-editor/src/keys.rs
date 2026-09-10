use raster_input::{Action, Binding, Bindings, Key};

/// The editor's own actions.
///
/// Distinctes de celles du jeu : un editeur n'a pas de saut, et emprunter les
/// touches de gameplay rendait les raccourcis incomprehensibles. Voir
/// `docs/friction.md`, entree 6.
pub struct Editor;

impl Editor {
    pub const UNDO: Action = Action::new("editor.undo");
    pub const REDO: Action = Action::new("editor.redo");
    pub const SAVE: Action = Action::new("editor.save");
    pub const OPEN: Action = Action::new("editor.open");
    pub const DELETE: Action = Action::new("editor.delete");
    pub const DUPLICATE: Action = Action::new("editor.duplicate");
    /// Bascule l'aimantation a la grille.
    pub const SNAP: Action = Action::new("editor.snap");
    /// Montre ou cache la grille.
    pub const GRID: Action = Action::new("editor.grid");
    /// Le modificateur : ce qui distingue annuler de refaire.
    pub const MODIFIER: Action = Action::new("editor.modifier");
    /// Le second modificateur, pour refaire.
    pub const SHIFT: Action = Action::new("editor.shift");
    pub const QUIT: Action = Action::new("editor.quit");
    /// Lance ou arrete la partie.
    pub const PLAY: Action = Action::new("editor.play");
    /// Met la partie en pause.
    pub const PAUSE: Action = Action::new("editor.pause");
}

/// The editor's key bindings.
///
/// Les memes que partout : Ctrl+Z, Ctrl+S, Suppr. Un editeur qui invente ses
/// raccourcis force a les reapprendre.
#[must_use]
pub fn bindings() -> Bindings {
    let mut bindings = Bindings::new();

    bindings.bind(Editor::MODIFIER, Binding::Key(Key::Control));
    bindings.bind(Editor::SHIFT, Binding::Key(Key::Shift));

    bindings.bind(Editor::UNDO, Binding::Key(Key::Z));
    bindings.bind(Editor::REDO, Binding::Key(Key::Y));
    bindings.bind(Editor::SAVE, Binding::Key(Key::S));
    bindings.bind(Editor::OPEN, Binding::Key(Key::O));
    bindings.bind(Editor::DUPLICATE, Binding::Key(Key::D));

    bindings.bind(Editor::DELETE, Binding::Key(Key::Delete));
    bindings.bind(Editor::DELETE, Binding::Key(Key::Backspace));

    bindings.bind(Editor::SNAP, Binding::Key(Key::G));
    bindings.bind(Editor::GRID, Binding::Key(Key::H));
    bindings.bind(Editor::QUIT, Binding::Key(Key::Escape));
    bindings.bind(Editor::PLAY, Binding::Key(Key::F5));
    bindings.bind(Editor::PAUSE, Binding::Key(Key::F6));

    bindings
}

/// What the editor was asked to do this frame.
///
/// Lue en une fois : un raccourci a modificateur se decide en regardant
/// plusieurs touches ensemble, pas une par une.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shortcuts {
    pub undo: bool,
    pub redo: bool,
    pub save: bool,
    pub open: bool,
    pub delete: bool,
    pub duplicate: bool,
    pub toggle_snap: bool,
    pub toggle_grid: bool,
    pub quit: bool,
    pub play: bool,
    pub pause: bool,
}

/// Reads the shortcuts from the input.
#[must_use]
pub fn read(input: &raster_input::Input) -> Shortcuts {
    let ctrl = input.held(&Editor::MODIFIER);
    let shift = input.held(&Editor::SHIFT);

    Shortcuts {
        // Ctrl+Maj+Z refait aussi : les deux conventions coexistent.
        undo: ctrl && !shift && input.pressed(&Editor::UNDO),
        redo: ctrl && (input.pressed(&Editor::REDO) || (shift && input.pressed(&Editor::UNDO))),
        save: ctrl && input.pressed(&Editor::SAVE),
        open: ctrl && input.pressed(&Editor::OPEN),
        duplicate: ctrl && input.pressed(&Editor::DUPLICATE),
        // Sans modificateur : ce sont des touches d'action directe.
        delete: !ctrl && input.pressed(&Editor::DELETE),
        toggle_snap: !ctrl && input.pressed(&Editor::SNAP),
        toggle_grid: !ctrl && input.pressed(&Editor::GRID),
        quit: input.pressed(&Editor::QUIT),
        play: input.pressed(&Editor::PLAY),
        pause: input.pressed(&Editor::PAUSE),
    }
}
