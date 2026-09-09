use crate::{Id, Theme};
use raster_math::{Rect, Vec2};
use std::collections::HashMap;

/// What the pointer is doing this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pointer {
    pub at: Vec2,
    /// Le bouton principal est enfonce.
    pub down: bool,
    /// Il vient d'etre enfonce cette frame.
    pub pressed: bool,
    /// Il vient d'etre relache.
    pub released: bool,
}

/// What the keyboard did this frame, for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Keys {
    pub next: bool,
    pub previous: bool,
    pub confirm: bool,
    pub cancel: bool,
    pub left: bool,
    pub right: bool,
}

/// The state a UI keeps between frames.
///
/// Mode retenu : le survol, le focus et la capture survivent d'une frame a
/// l'autre, sinon un glissement de curseur s'interromprait des que la souris
/// quitte le widget.
pub struct Ui {
    theme: Theme,
    pointer: Pointer,
    keys: Keys,

    hovered: Option<Id>,
    /// Le widget qui garde le pointeur meme s'il sort : un curseur qu'on tire
    /// continue de suivre la souris.
    captured: Option<Id>,
    focused: Option<Id>,
    /// Les widgets vus cette frame, dans l'ordre : la navigation clavier les
    /// parcourt dans cet ordre.
    order: Vec<Id>,
    /// L'etat que chaque widget garde, comme la position d'un ascenseur.
    memory: HashMap<Id, f32>,
    /// Ce qui a ete survole avant, pour ne garder que le dernier — donc le
    /// plus haut dans l'ordre de dessin.
    hover_candidate: Option<Id>,
}

impl Ui {
    #[must_use]
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            pointer: Pointer::default(),
            keys: Keys::default(),
            hovered: None,
            captured: None,
            focused: None,
            order: Vec::new(),
            memory: HashMap::new(),
            hover_candidate: None,
        }
    }

    /// Starts a frame with fresh input.
    pub fn begin(&mut self, pointer: Pointer, keys: Keys) {
        self.pointer = pointer;
        self.keys = keys;
        self.order.clear();
        self.hover_candidate = None;
    }

    /// Ends a frame, settling hover and keyboard navigation.
    pub fn end(&mut self) {
        self.hovered = self.hover_candidate;

        // La capture se libere en fin de frame : la liberer a l'ouverture
        // priverait le widget de l'evenement qui conclut son propre clic.
        if self.pointer.released {
            self.captured = None;
        }

        if self.keys.next {
            self.move_focus(1);
        } else if self.keys.previous {
            self.move_focus(-1);
        }

        // Un focus sur un widget disparu ne mene nulle part.
        if self.focused.is_some_and(|id| !self.order.contains(&id)) {
            self.focused = None;
        }
    }

    fn move_focus(&mut self, by: isize) {
        if self.order.is_empty() {
            return;
        }

        let next = match self
            .focused
            .and_then(|id| self.order.iter().position(|o| *o == id))
        {
            Some(current) => {
                let n = self.order.len() as isize;
                ((current as isize + by).rem_euclid(n)) as usize
            }
            // Sans focus, `suivant` prend le premier et `precedent` le dernier.
            None if by > 0 => 0,
            None => self.order.len() - 1,
        };

        self.focused = Some(self.order[next]);
    }

    /// Registers a widget and reports what is happening to it.
    ///
    /// Le dernier a revendiquer une position l'emporte : les widgets dessines
    /// en dernier sont au-dessus.
    pub fn interact(&mut self, id: Id, area: Rect, enabled: bool) -> Response {
        if enabled {
            self.order.push(id);
        }

        let inside = area.contains(self.pointer.at);
        let mut captured = self.captured == Some(id);

        if enabled && inside && self.captured.is_none() {
            self.hover_candidate = Some(id);
        }

        let hovered = enabled && (captured || (inside && self.hover_candidate == Some(id)));

        if hovered && self.pointer.pressed {
            self.captured = Some(id);
            self.focused = Some(id);
            // Pris en compte des cette frame : sans cela un bouton ne
            // s'enfoncerait qu'a la frame suivante.
            captured = true;
        }

        let focused = self.focused == Some(id) && enabled;
        let held = captured && self.pointer.down;

        // Un clic ne compte que relache sur le widget qui l'a commence : sortir
        // avant de relacher annule, comme partout ailleurs.
        let clicked = enabled && captured && self.pointer.released && inside;
        let activated = clicked || (focused && self.keys.confirm);

        Response {
            id,
            area,
            hovered,
            held,
            clicked: activated,
            focused,
            enabled,
        }
    }

    #[must_use]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    #[must_use]
    pub fn pointer(&self) -> Pointer {
        self.pointer
    }

    #[must_use]
    pub fn keys(&self) -> Keys {
        self.keys
    }

    #[must_use]
    pub fn focused(&self) -> Option<Id> {
        self.focused
    }

    pub fn focus(&mut self, id: Id) {
        self.focused = Some(id);
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    #[must_use]
    pub fn hovered(&self) -> Option<Id> {
        self.hovered
    }

    #[must_use]
    pub fn captured(&self) -> Option<Id> {
        self.captured
    }

    /// The value a widget remembers, or `default` the first time.
    pub fn remember(&mut self, id: Id, default: f32) -> f32 {
        *self.memory.entry(id).or_insert(default)
    }

    pub fn store(&mut self, id: Id, value: f32) {
        self.memory.insert(id, value);
    }

    /// Le nombre de widgets vus cette frame.
    #[must_use]
    pub fn count(&self) -> usize {
        self.order.len()
    }
}

/// What happened to a widget this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Response {
    pub id: Id,
    pub area: Rect,
    pub hovered: bool,
    /// Le bouton est enfonce sur ce widget.
    pub held: bool,
    /// Il vient d'etre active, a la souris ou au clavier.
    pub clicked: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl Response {
    #[must_use]
    pub fn state(&self) -> crate::State {
        use crate::State;
        if !self.enabled {
            State::Disabled
        } else if self.held {
            State::Pressed
        } else if self.hovered {
            State::Hovered
        } else {
            State::Idle
        }
    }
}
