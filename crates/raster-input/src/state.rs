use crate::{Action, Axis, Binding, Bindings, GamepadButton, Key, MouseButton};
use raster_math::Vec2;
use std::collections::{HashMap, HashSet};

/// Everything the game knows about input this frame.
///
/// Two conveniences are built in rather than left to gameplay code, because
/// every action game reimplements them and usually badly: input buffering and
/// coyote time. Together they are most of the difference between controls that
/// feel responsive and controls that feel broken.
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

    /// How long a press stays available to be consumed.
    buffer_window: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self::new(Bindings::default_layout())
    }
}

impl Input {
    /// The default buffer window.
    ///
    /// Roughly seven frames at 60fps: long enough that a jump pressed just
    /// before landing still fires, short enough that a stale press does not
    /// surprise the player later.
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

    /// Whether an action was pressed within the buffer window.
    ///
    /// This is what lets a jump pressed a few frames before landing still
    /// fire. Reading it does not consume it — see
    /// [`Input::consume_buffered`].
    #[must_use]
    pub fn buffered(&self, action: &Action) -> bool {
        self.pressed_at
            .get(action)
            .is_some_and(|age| *age <= self.buffer_window)
    }

    /// Takes a buffered press, so it cannot fire twice.
    ///
    /// A jump that consumed its buffered press must not jump again on the next
    /// frame from the same press.
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

    /// Two axes as a direction, normalised so diagonal movement is not faster.
    ///
    /// Holding right and down without this would move 41% faster than holding
    /// right alone, which is the classic bug.
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

    /// Clears every held input.
    ///
    /// Called when the window loses focus: without it, a key held while
    /// alt-tabbing stays held forever, because the release event goes to
    /// another window.
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

        // Vieillit les pressions memorisees, et oublie celles trop anciennes
        // pour que la table ne grandisse pas indefiniment.
        self.pressed_at.retain(|_, age| {
            *age += dt;
            *age <= self.buffer_window
        });

        // Enregistre les nouvelles pressions. Fait apres le vieillissement,
        // pour qu'une pression de cette frame ait bien un age nul.
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

/// Tracks how long ago a condition was last true.
///
/// The mechanism behind coyote time: a jump pressed shortly after walking off
/// a ledge should still work. Kept separate from [`Input`] because the
/// condition is about the world — being on the ground — not about the
/// keyboard.
#[derive(Debug, Clone, Copy)]
pub struct Grace {
    window: f32,
    since: f32,
}

impl Grace {
    /// A window of roughly six frames at 60fps: forgiving enough to feel fair,
    /// short enough that nobody notices the ground was not there.
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

    /// Whether the condition holds, or held recently enough to still count.
    #[must_use]
    pub fn active(&self) -> bool {
        self.since <= self.window
    }

    /// Ends the grace period, so it cannot be used twice.
    ///
    /// A jump that used its coyote time must not jump again from the same
    /// window.
    pub fn consume(&mut self) {
        self.since = f32::INFINITY;
    }
}

impl Default for Grace {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}
