use crate::{Animation, Animator};
use std::collections::HashMap;

/// Named animations and the rules for moving between them.
///
/// Sans cela, l'animation d'un personnage devient un tas de booléens dans le
/// code de jeu, recalculé à chaque frame et faux dès qu'un cas s'ajoute.
#[derive(Debug, Default)]
pub struct StateMachine {
    states: HashMap<String, Animation>,
    /// De quel état vers quel autre, avec la condition qui le déclenche.
    transitions: Vec<Transition>,
    current: String,
    animator: Animator,
}

#[derive(Debug, Clone)]
struct Transition {
    /// `None` : depuis n'importe quel état.
    from: Option<String>,
    to: String,
    condition: String,
    /// N'accepte la transition qu'une fois l'animation terminée.
    wait_for_end: bool,
}

impl StateMachine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a state. The first one added becomes the current one.
    pub fn add(&mut self, name: impl Into<String>, animation: Animation) {
        let name = name.into();
        if self.states.is_empty() {
            self.current = name.clone();
        }
        self.states.insert(name, animation);
    }

    /// Allows moving from one state to another when `condition` holds.
    pub fn transition(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        condition: impl Into<String>,
    ) {
        self.transitions.push(Transition {
            from: Some(from.into()),
            to: to.into(),
            condition: condition.into(),
            wait_for_end: false,
        });
    }

    /// Allows moving to a state from anywhere.
    pub fn transition_any(&mut self, to: impl Into<String>, condition: impl Into<String>) {
        self.transitions.push(Transition {
            from: None,
            to: to.into(),
            condition: condition.into(),
            wait_for_end: false,
        });
    }

    /// Like [`StateMachine::transition`], but only once the animation ends.
    ///
    /// Ce qu'il faut pour qu'une attaque aille au bout avant de rendre la main.
    pub fn transition_after(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        condition: impl Into<String>,
    ) {
        self.transitions.push(Transition {
            from: Some(from.into()),
            to: to.into(),
            condition: condition.into(),
            wait_for_end: true,
        });
    }

    /// Advances the machine, returning the animation events crossed.
    ///
    /// `conditions` lists what holds this frame — the game decides what those
    /// mean.
    pub fn update(&mut self, conditions: &[&str], dt: f32) -> Vec<String> {
        if let Some(next) = self.pick(conditions)
            && next != self.current
        {
            self.current = next;
            self.animator.restart();
        }

        let Some(animation) = self.states.get(&self.current) else {
            return Vec::new();
        };

        self.animator
            .advance(animation, dt)
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    /// The state a set of conditions leads to, if any.
    ///
    /// La première transition déclarée l'emporte : l'ordre de déclaration est
    /// la priorité, ce qui évite un système de poids à régler.
    fn pick(&self, conditions: &[&str]) -> Option<String> {
        self.transitions
            .iter()
            .find(|t| {
                let from_matches = t.from.as_ref().is_none_or(|f| *f == self.current);
                let ready = !t.wait_for_end || self.animator.finished();
                from_matches && ready && conditions.contains(&t.condition.as_str())
            })
            .map(|t| t.to.clone())
    }

    /// The sprite index to draw.
    #[must_use]
    pub fn index(&self) -> u32 {
        self.states
            .get(&self.current)
            .map_or(0, |a| self.animator.index(a))
    }

    #[must_use]
    pub fn state(&self) -> &str {
        &self.current
    }

    /// Switches state immediately, ignoring transitions.
    pub fn force(&mut self, state: impl Into<String>) {
        let state = state.into();
        if self.states.contains_key(&state) && state != self.current {
            self.current = state;
            self.animator.restart();
        }
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.animator.finished()
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.animator.set_speed(speed);
    }
}
