use raster_core::ActorId;
use raster_math::{Rect, Vec2};

/// What the viewport is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Select,
    /// Un rectangle de selection est en cours.
    BoxSelect,
    /// Un acteur est en cours de deplacement.
    Move,
    /// La vue est en cours de deplacement.
    Pan,
}

/// The scene view: what is visible, what is selected, what is being dragged.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Le centre de la vue, en pixels du monde.
    pub camera: Vec2,
    /// Le zoom, entier : un demi-pixel rendrait le pixel art flou.
    pub zoom: u32,
    pub mode: Mode,
    /// Ce qui est selectionne, dans l'ordre de selection.
    selection: Vec<ActorId>,
    /// Ou un glissement a commence, en pixels du monde.
    drag_from: Option<Vec2>,
    /// La position du curseur au dernier mouvement.
    cursor: Vec2,
    pub grid: f32,
    pub snap: bool,
    pub show_grid: bool,
}

impl Viewport {
    /// Seize pixels : la taille d'une tuile, donc le pas naturel.
    pub const DEFAULT_GRID: f32 = 16.0;

    /// Le zoom maximum : au-dela, un pixel occupe tout l'ecran.
    pub const MAX_ZOOM: u32 = 16;

    #[must_use]
    pub fn new() -> Self {
        Self {
            camera: Vec2::ZERO,
            zoom: 2,
            mode: Mode::Select,
            selection: Vec::new(),
            drag_from: None,
            cursor: Vec2::ZERO,
            grid: Self::DEFAULT_GRID,
            snap: true,
            show_grid: true,
        }
    }

    /// Turns a point in the panel into a point in the world.
    #[must_use]
    pub fn to_world(&self, area: Rect, at: Vec2) -> Vec2 {
        let centre = area.position + area.size * 0.5;
        self.camera + (at - centre) / self.zoom.max(1) as f32
    }

    /// And back, for drawing.
    #[must_use]
    pub fn to_screen(&self, area: Rect, world: Vec2) -> Vec2 {
        let centre = area.position + area.size * 0.5;
        centre + (world - self.camera) * self.zoom.max(1) as f32
    }

    /// The world area the panel shows.
    #[must_use]
    pub fn visible(&self, area: Rect) -> Rect {
        let size = area.size / self.zoom.max(1) as f32;
        Rect::new(
            self.camera.x - size.x / 2.0,
            self.camera.y - size.y / 2.0,
            size.x,
            size.y,
        )
    }

    /// Zooms in or out, keeping `at` under the cursor.
    ///
    /// Sans ce point fixe, zoomer deplacerait ce qu'on regarde : c'est ce qui
    /// rend une vue penible a naviguer.
    pub fn zoom_at(&mut self, area: Rect, at: Vec2, by: i32) {
        let before = self.to_world(area, at);

        let next = if by > 0 {
            self.zoom.saturating_mul(2)
        } else {
            self.zoom / 2
        };
        self.zoom = next.clamp(1, Self::MAX_ZOOM);

        let after = self.to_world(area, at);
        self.camera += before - after;
    }

    /// Snaps a point to the grid, if snapping is on.
    #[must_use]
    pub fn snapped(&self, at: Vec2) -> Vec2 {
        if !self.snap || self.grid <= 0.0 {
            // Toujours au pixel entier : un acteur a mi-pixel rendrait flou.
            return Vec2::new(at.x.round(), at.y.round());
        }
        Vec2::new(
            (at.x / self.grid).round() * self.grid,
            (at.y / self.grid).round() * self.grid,
        )
    }

    /// Starts a drag at a world position.
    pub fn begin_drag(&mut self, at: Vec2, mode: Mode) {
        self.drag_from = Some(at);
        self.cursor = at;
        self.mode = mode;
    }

    /// Moves the cursor, returning how far it went since the last call.
    pub fn drag_to(&mut self, at: Vec2) -> Vec2 {
        let delta = at - self.cursor;
        self.cursor = at;
        delta
    }

    /// Ends a drag, returning the rectangle it covered.
    ///
    /// Normalise : un rectangle tire vers le haut a gauche a une taille
    /// positive, sinon rien ne le croiserait.
    pub fn end_drag(&mut self) -> Option<Rect> {
        let from = self.drag_from.take()?;
        self.mode = Mode::Select;

        let min = Vec2::new(from.x.min(self.cursor.x), from.y.min(self.cursor.y));
        let max = Vec2::new(from.x.max(self.cursor.x), from.y.max(self.cursor.y));
        Some(Rect::new(min.x, min.y, max.x - min.x, max.y - min.y))
    }

    /// The rectangle a box-select currently covers.
    #[must_use]
    pub fn drag_rect(&self) -> Option<Rect> {
        let from = self.drag_from?;
        let min = Vec2::new(from.x.min(self.cursor.x), from.y.min(self.cursor.y));
        let max = Vec2::new(from.x.max(self.cursor.x), from.y.max(self.cursor.y));
        Some(Rect::new(min.x, min.y, max.x - min.x, max.y - min.y))
    }

    #[must_use]
    pub fn dragging(&self) -> bool {
        self.drag_from.is_some()
    }

    #[must_use]
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    // --- Selection ---

    /// Selects one actor, replacing what was selected.
    pub fn select(&mut self, actor: ActorId) {
        self.selection.clear();
        self.selection.push(actor);
    }

    /// Adds to the selection, or removes what was already in it.
    pub fn toggle(&mut self, actor: ActorId) {
        match self.selection.iter().position(|a| *a == actor) {
            Some(i) => {
                self.selection.remove(i);
            }
            None => self.selection.push(actor),
        }
    }

    /// Replaces the selection with everything given.
    pub fn select_all(&mut self, actors: impl IntoIterator<Item = ActorId>) {
        self.selection.clear();
        for actor in actors {
            if !self.selection.contains(&actor) {
                self.selection.push(actor);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    #[must_use]
    pub fn selection(&self) -> &[ActorId] {
        &self.selection
    }

    #[must_use]
    pub fn is_selected(&self, actor: ActorId) -> bool {
        self.selection.contains(&actor)
    }

    /// The one selected actor, if exactly one is.
    ///
    /// L'inspecteur n'en montre qu'un : a plusieurs, il n'y a pas de valeur
    /// commune a afficher.
    #[must_use]
    pub fn only_selected(&self) -> Option<ActorId> {
        match self.selection.as_slice() {
            [one] => Some(*one),
            _ => None,
        }
    }

    /// Drops selected actors that no longer exist.
    ///
    /// Une selection qui survit a une suppression pointerait dans le vide.
    pub fn retain_selection(&mut self, exists: impl Fn(ActorId) -> bool) {
        self.selection.retain(|a| exists(*a));
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// The handles shown around a selected actor.
///
/// Deplacement seulement pour l'instant : en 2D, une echelle libre casse
/// l'alignement au pixel, et une rotation aussi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    /// Le corps de l'acteur : le tirer le deplace librement.
    Body,
    /// L'axe horizontal seul.
    AxisX,
    /// L'axe vertical seul.
    AxisY,
}

/// Where the handles sit around an actor's bounds, in world pixels.
#[must_use]
pub fn handles(bounds: Rect) -> [(Handle, Rect); 3] {
    let centre = bounds.position + bounds.size * 0.5;
    let arm = 12.0;
    let thick = 3.0;

    [
        (Handle::Body, bounds),
        (
            Handle::AxisX,
            Rect::new(centre.x, centre.y - thick / 2.0, arm, thick),
        ),
        (
            Handle::AxisY,
            Rect::new(centre.x - thick / 2.0, centre.y, thick, arm),
        ),
    ]
}

/// Constrains a movement to a handle's axis.
#[must_use]
pub fn constrain(handle: Handle, delta: Vec2) -> Vec2 {
    match handle {
        Handle::Body => delta,
        Handle::AxisX => Vec2::new(delta.x, 0.0),
        Handle::AxisY => Vec2::new(0.0, delta.y),
    }
}
