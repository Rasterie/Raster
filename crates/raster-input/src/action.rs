use std::collections::HashMap;

/// What a game does, named by intent rather than by key.
///
/// Gameplay asks whether `Jump` is pressed, never whether Space is. That is
/// what makes rebinding and gamepad support possible without touching gameplay
/// code.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Action(pub &'static str);

impl Action {
    pub const JUMP: Self = Self("jump");
    pub const ATTACK: Self = Self("attack");
    pub const INTERACT: Self = Self("interact");
    pub const PAUSE: Self = Self("pause");
}

/// A pair of actions read as a single value from -1 to 1.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Axis {
    pub negative: Action,
    pub positive: Action,
}

impl Axis {
    pub const HORIZONTAL: Self = Self {
        negative: Action("left"),
        positive: Action("right"),
    };
    pub const VERTICAL: Self = Self {
        negative: Action("up"),
        positive: Action("down"),
    };
}

/// A physical input that can trigger an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Binding {
    Key(Key),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
}

/// A keyboard key, identified by physical position rather than by the
/// character it produces.
///
/// Position rather than character on purpose: WASD must stay under the same
/// fingers on an AZERTY keyboard, where those positions produce ZQSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    Left,
    Right,
    Up,
    Down,

    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Shift,
    Control,
    Alt,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Start,
    Select,
}

/// Which physical inputs trigger which actions.
///
/// Several bindings may map to one action — a gamepad button alongside a key —
/// and the action fires if any of them is active.
#[derive(Debug, Clone, Default)]
pub struct Bindings {
    map: HashMap<Action, Vec<Binding>>,
}

impl Bindings {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A layout that lets a game run before anyone configures anything.
    ///
    /// Both WASD and the arrow keys, because a player will try one or the
    /// other and being wrong about which is an avoidable annoyance.
    #[must_use]
    pub fn default_layout() -> Self {
        let mut bindings = Self::new();

        bindings.bind(Action("left"), Binding::Key(Key::A));
        bindings.bind(Action("left"), Binding::Key(Key::Left));
        bindings.bind(
            Action("left"),
            Binding::GamepadButton(GamepadButton::DPadLeft),
        );

        bindings.bind(Action("right"), Binding::Key(Key::D));
        bindings.bind(Action("right"), Binding::Key(Key::Right));
        bindings.bind(
            Action("right"),
            Binding::GamepadButton(GamepadButton::DPadRight),
        );

        bindings.bind(Action("up"), Binding::Key(Key::W));
        bindings.bind(Action("up"), Binding::Key(Key::Up));
        bindings.bind(Action("up"), Binding::GamepadButton(GamepadButton::DPadUp));

        bindings.bind(Action("down"), Binding::Key(Key::S));
        bindings.bind(Action("down"), Binding::Key(Key::Down));
        bindings.bind(
            Action("down"),
            Binding::GamepadButton(GamepadButton::DPadDown),
        );

        bindings.bind(Action::JUMP, Binding::Key(Key::Space));
        bindings.bind(Action::JUMP, Binding::GamepadButton(GamepadButton::South));

        bindings.bind(Action::ATTACK, Binding::MouseButton(MouseButton::Left));
        bindings.bind(Action::ATTACK, Binding::GamepadButton(GamepadButton::West));

        bindings.bind(Action::INTERACT, Binding::Key(Key::E));
        bindings.bind(
            Action::INTERACT,
            Binding::GamepadButton(GamepadButton::North),
        );

        bindings.bind(Action::PAUSE, Binding::Key(Key::Escape));
        bindings.bind(Action::PAUSE, Binding::GamepadButton(GamepadButton::Start));

        bindings
    }

    /// Adds a binding, keeping any already attached to this action.
    pub fn bind(&mut self, action: Action, binding: Binding) {
        self.map.entry(action).or_default().push(binding);
    }

    /// Replaces every binding for an action.
    pub fn rebind(&mut self, action: Action, bindings: Vec<Binding>) {
        self.map.insert(action, bindings);
    }

    pub fn clear(&mut self, action: &Action) {
        self.map.remove(action);
    }

    #[must_use]
    pub fn bindings_for(&self, action: &Action) -> &[Binding] {
        self.map.get(action).map_or(&[], Vec::as_slice)
    }

    /// Every action bound to a physical input.
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.map.keys()
    }
}
