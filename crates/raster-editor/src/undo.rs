use std::time::{Duration, Instant};

/// Something done that can be undone.
///
/// Le patron commande : chaque modification sait se defaire et se refaire, ce
/// qui rend l'historique independant de ce qui a ete modifie.
pub trait Command: std::fmt::Debug + std::any::Any {
    /// Ce qui permet a une commande de reconnaitre sa jumelle avant de
    /// fusionner : sans cela, elle ne verrait qu'un `dyn Command`.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Applies the change.
    fn apply(&mut self, world: &mut dyn Target);

    /// Puts back what `apply` changed.
    fn revert(&mut self, world: &mut dyn Target);

    /// A short description, shown in the history.
    fn label(&self) -> String;

    /// Whether this command can absorb `other`, which came just after.
    ///
    /// Ce qui fait qu'un glissement est une seule entree d'annulation, et non
    /// une par pixel parcouru.
    fn merge(&mut self, _other: &dyn Command) -> bool {
        false
    }
}

/// What commands act on.
///
/// La pile ignore ce qu'elle modifie ; les commandes, elles, connaissent leur
/// cible et la retrouvent par `as_any_mut`. Sans ce pont, elles devraient
/// deviner le type, ce qui demanderait un `unsafe` injustifiable.
pub trait Target: std::any::Any {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// The concrete target a command works on, or `None` if it is another kind.
///
/// # Errors
///
/// Renvoie `None` plutot que paniquer : deux editeurs peuvent pousser dans la
/// meme pile, et une commande ne doit pas faire tomber celle d'un autre.
pub fn target_as<T: 'static>(target: &mut dyn Target) -> Option<&mut T> {
    target.as_any_mut().downcast_mut::<T>()
}

/// One undo history, shared by every editor.
///
/// Une seule pile plutot qu'une par panneau : annuler apres avoir change de
/// panneau doit faire ce a quoi on s'attend, pas une surprise.
pub struct History {
    done: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
    /// Quand la derniere commande a ete poussee, pour la fusion.
    last_at: Option<Instant>,
    /// Au-dela de ce delai, deux commandes ne fusionnent plus meme si elles
    /// le pourraient : une pause veut dire deux gestes.
    merge_window: Duration,
    limit: usize,
    /// La profondeur au dernier enregistrement, pour savoir si le projet a
    /// change depuis.
    saved_at: usize,
}

impl History {
    /// Cent entrees : au-dela, personne ne remonte, et chacune retient de la
    /// memoire.
    pub const DEFAULT_LIMIT: usize = 100;

    /// Une demi-seconde : au-dela, deux mouvements sont deux gestes.
    pub const DEFAULT_MERGE_WINDOW: Duration = Duration::from_millis(500);

    #[must_use]
    pub fn new() -> Self {
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            last_at: None,
            merge_window: Self::DEFAULT_MERGE_WINDOW,
            limit: Self::DEFAULT_LIMIT,
            saved_at: 0,
        }
    }

    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }

    /// Applies a command and records it.
    ///
    /// Efface ce qui avait ete annule : refaire apres une nouvelle action
    /// n'aurait pas de sens.
    pub fn push(&mut self, mut command: Box<dyn Command>, target: &mut dyn Target) {
        command.apply(target);
        self.undone.clear();

        let recent = self
            .last_at
            .is_some_and(|at| at.elapsed() < self.merge_window);

        if recent
            && let Some(last) = self.done.last_mut()
            && last.merge(command.as_ref())
        {
            self.last_at = Some(Instant::now());
            return;
        }

        self.done.push(command);
        self.last_at = Some(Instant::now());

        if self.done.len() > self.limit {
            self.done.remove(0);
            // La marque d'enregistrement suit ce qui est oublie, sinon elle
            // designerait la mauvaise entree.
            self.saved_at = self.saved_at.saturating_sub(1);
        }
    }

    /// Undoes the last command.
    pub fn undo(&mut self, target: &mut dyn Target) -> bool {
        let Some(mut command) = self.done.pop() else {
            return false;
        };
        command.revert(target);
        self.undone.push(command);
        // Une annulation coupe la fusion : le geste suivant est un geste neuf.
        self.last_at = None;
        true
    }

    /// Redoes the last undone command.
    pub fn redo(&mut self, target: &mut dyn Target) -> bool {
        let Some(mut command) = self.undone.pop() else {
            return false;
        };
        command.apply(target);
        self.done.push(command);
        self.last_at = None;
        true
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// The label of what undoing would revert.
    #[must_use]
    pub fn undo_label(&self) -> Option<String> {
        self.done.last().map(|c| c.label())
    }

    #[must_use]
    pub fn redo_label(&self) -> Option<String> {
        self.undone.last().map(|c| c.label())
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.done.len()
    }

    /// The labels of what has been done, oldest first.
    #[must_use]
    pub fn labels(&self) -> Vec<String> {
        self.done.iter().map(|c| c.label()).collect()
    }

    /// Marks the current state as saved.
    pub fn mark_saved(&mut self) {
        self.saved_at = self.done.len();
    }

    /// Whether anything changed since the last save.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.done.len() != self.saved_at
    }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
        self.last_at = None;
        self.saved_at = 0;
    }

    /// Ends the current merge window, so the next command starts a new entry.
    ///
    /// A appeler quand un glissement se termine : sans cela, le geste suivant
    /// pourrait s'y coller.
    pub fn break_merge(&mut self) {
        self.last_at = None;
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
