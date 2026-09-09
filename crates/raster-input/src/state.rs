use crate::{Action, Axis, Binding, Bindings, GamepadButton, Key, MouseButton};
use raster_math::Vec2;
use std::collections::{HashMap, HashSet};

/// Everything the game knows about input this frame.
///
/// La memorisation des pressions et le coyote time sont fournis ici : tout jeu
/// d'action les reimplemente, en general mal.
#[derive(Debug)]
pub struct Input {
    bindings: Bindings,

    keys: HashSet<Key>,
    mouse_buttons: HashSet<MouseButton>,
    gamepad_buttons: HashSet<GamepadButton>,

    /// What was held last frame, so a press can be told from a hold.
    previous: HashSet<Binding>,
    current: HashSet<Binding>,

    /// How long ago each action was pressed, in seconds. Absent means never.
    pressed_at: HashMap<Action, f32>,

    mouse_position: Vec2,
    mouse_delta: Vec2,
    scroll_delta: f32,

    /// Duree pendant laquelle une pression reste consommable.
    buffer_window: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self::new(Bindings::default_layout())
    }
}

impl Input {
    /// Environ sept frames a 60 images/s : assez pour qu'un saut demande juste
    /// avant l'atterrissage parte, trop court pour surprendre plus tard.
    pub const DEFAULT_BUFFER: f32 = 0.12;

    #[must_use]
    pub fn new(bindings: Bindings) -> Self {
        Self {
            bindings,
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),
            gamepad_buttons: HashSet::new(),
            previous: HashSet::new(),
            current: HashSet::new(),
            pressed_at: HashMap::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: 0.0,
            buffer_window: Self::DEFAULT_BUFFER,
        }
    }

    #[must_use]
    pub fn bindings(&self) -> &Bindings {
        &self.bindings
    }

    pub fn bindings_mut(&mut self) -> &mut Bindings {
        &mut self.bindings
    }

    pub fn set_buffer_window(&mut self, seconds: f32) {
        self.buffer_window = seconds.max(0.0);
    }

    // --- interrogation ------------------------------------------------------

    /// Whether an action is held right now.
    #[must_use]
    pub fn held(&self, action: &Action) -> bool {
        self.bindings
            .bindings_for(action)
            .iter()
            .any(|b| self.current.contains(b))
    }

    /// Whether an action started this frame.
    #[must_use]
    pub fn pressed(&self, action: &Action) -> bool {
        let bindings = self.bindings.bindings_for(action);
        bindings.iter().any(|b| self.current.contains(b))
            && !bindings.iter().any(|b| self.previous.contains(b))
    }

    /// Whether an action stopped this frame.
    #[must_use]
    pub fn released(&self, action: &Action) -> bool {
        let bindings = self.bindings.bindings_for(action);
        bindings.iter().any(|b| self.previous.contains(b))
            && !bindings.iter().any(|b| self.current.contains(b))
    }

    /// Si l'action a ete pressee dans la fenetre. Lire ne consomme pas — voir
    /// [`Input::consume_buffered`].
    #[must_use]
    pub fn buffered(&self, action: &Action) -> bool {
        self.pressed_at
            .get(action)
            .is_some_and(|age| *age <= self.buffer_window)
    }

    /// Consomme une pression memorisee, pour qu'elle ne parte qu'une fois.
    pub fn consume_buffered(&mut self, action: &Action) -> bool {
        if self.buffered(action) {
            self.pressed_at.remove(action);
            return true;
        }
        false
    }

    /// An axis from a pair of actions, in -1..1.
    #[must_use]
    pub fn axis(&self, axis: &Axis) -> f32 {
        let negative = f32::from(u8::from(self.held(&axis.negative)));
        let positive = f32::from(u8::from(self.held(&axis.positive)));
        positive - negative
    }

    /// Deux axes en direction, normalisee : sans cela la diagonale irait 41 %
    /// plus vite.
    #[must_use]
    pub fn direction(&self) -> Vec2 {
        Vec2::new(self.axis(&Axis::HORIZONTAL), self.axis(&Axis::VERTICAL)).normalized()
    }

    #[must_use]
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    #[must_use]
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    #[must_use]
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // --- alimentation par la plateforme ------------------------------------

    pub fn key_down(&mut self, key: Key) {
        self.keys.insert(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.keys.remove(&key);
    }

    pub fn mouse_down(&mut self, button: MouseButton) {
        self.mouse_buttons.insert(button);
    }

    pub fn mouse_up(&mut self, button: MouseButton) {
        self.mouse_buttons.remove(&button);
    }

    pub fn gamepad_down(&mut self, button: GamepadButton) {
        self.gamepad_buttons.insert(button);
    }

    pub fn gamepad_up(&mut self, button: GamepadButton) {
        self.gamepad_buttons.remove(&button);
    }

    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.mouse_delta += position - self.mouse_position;
        self.mouse_position = position;
    }

    pub fn add_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// Relache tout, quand la fenetre perd le focus : sinon une touche
    /// maintenue le reste pour toujours.
    pub fn release_all(&mut self) {
        self.keys.clear();
        self.mouse_buttons.clear();
        self.gamepad_buttons.clear();
        self.current.clear();
    }

    /// Advances one frame. Called by the engine before gameplay runs.
    pub fn begin_frame(&mut self, dt: f32) {
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current.clear();

        for key in &self.keys {
            self.current.insert(Binding::Key(*key));
        }
        for button in &self.mouse_buttons {
            self.current.insert(Binding::MouseButton(*button));
        }
        for button in &self.gamepad_buttons {
            self.current.insert(Binding::GamepadButton(*button));
        }

        // Vieillit les pressions et oublie les trop anciennes.
        self.pressed_at.retain(|_, age| {
            *age += dt;
            *age <= self.buffer_window
        });

        // Apres le vieillissement, pour qu'une pression neuve ait un age nul.
        let actions: Vec<Action> = self.bindings.actions().cloned().collect();
        for action in actions {
            if self.pressed(&action) {
                self.pressed_at.insert(action, 0.0);
            }
        }

        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }
}

/// Depuis combien de temps une condition a cesse d'etre vraie.
///
/// Le coyote time : sauter juste apres avoir quitte une plateforme marche
/// encore. Separe d'[`Input`] car la condition porte sur le monde.
#[derive(Debug, Clone, Copy)]
pub struct Grace {
    window: f32,
    since: f32,
}

impl Grace {
    /// Environ six frames a 60 images/s.
    pub const DEFAULT: f32 = 0.1;

    #[must_use]
    pub fn new(window: f32) -> Self {
        Self {
            window: window.max(0.0),
            // Commence expire : rien n'a encore ete vrai.
            since: f32::INFINITY,
        }
    }

    /// Updates with whether the condition holds this frame.
    pub fn update(&mut self, condition: bool, dt: f32) {
        if condition {
            self.since = 0.0;
        } else {
            self.since += dt;
        }
    }

    /// Si la condition tient, ou a tenu assez recemment.
    #[must_use]
    pub fn active(&self) -> bool {
        self.since <= self.window
    }

    /// Termine le delai, pour qu'il ne serve qu'une fois.
    pub fn consume(&mut self) {
        self.since = f32::INFINITY;
    }
}

impl Default for Grace {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}
