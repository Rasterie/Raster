use raster_math::{Mat3, Rect, Vec2};

/// An orthographic 2D camera.
///
/// Holds a position in world space and a fixed resolution in pixels. The
/// resolution is the game's, not the window's: the renderer draws at that size
/// and scales the result up by a whole number, which is what keeps pixel art
/// sharp.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// The point at the centre of the view, in world space.
    ///
    /// Volontairement pas accrochee a la grille : les sprites s'accrochent, pas
    /// la camera. Accrocher les deux fait saccader, aucun rend flou.
    pub position: Vec2,

    /// The visible area in pixels, before scaling to the window.
    pub resolution: Vec2,

    /// Integer zoom. Fractional zoom would put sprites on half-pixels.
    pub zoom: u32,
}

impl Camera {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vec2::ZERO,
            resolution: Vec2::new(width as f32, height as f32),
            zoom: 1,
        }
    }

    /// La matrice monde -> clip. Le clip monte en Y, le moteur descend, d'ou
    /// l'echelle negative.
    #[must_use]
    pub fn view_projection(self) -> Mat3 {
        let zoom = self.zoom.max(1) as f32;
        let half = self.resolution * 0.5 / zoom;

        let offset = self.position;

        Mat3::new(
            Vec2::new(1.0 / half.x, 0.0),
            Vec2::new(0.0, -1.0 / half.y),
            Vec2::new(-offset.x / half.x, offset.y / half.y),
        )
    }

    /// The area of the world this camera can see, used for culling.
    #[must_use]
    pub fn visible_area(self) -> Rect {
        let zoom = self.zoom.max(1) as f32;
        let size = self.resolution / zoom;
        Rect::from_center_size(self.position, size)
    }

    /// Convertit un point de l'ecran en point du monde, bandes comprises.
    #[must_use]
    pub fn screen_to_world(self, screen: Vec2, window: Vec2) -> Vec2 {
        let scale = self.window_scale(window) as f32;
        let zoom = self.zoom.max(1) as f32;

        let scaled = self.resolution * scale;
        let margin = (window - scaled) * 0.5;
        let inside = (screen - margin) / scale;

        self.position + (inside - self.resolution * 0.5) / zoom
    }

    /// Le facteur d'agrandissement, toujours entier : a 3,7 certains pixels
    /// seraient plus larges que d'autres.
    #[must_use]
    pub fn window_scale(self, window: Vec2) -> u32 {
        let x = (window.x / self.resolution.x).floor() as u32;
        let y = (window.y / self.resolution.y).floor() as u32;
        x.min(y).max(1)
    }

    /// Where the scaled image sits in the window, centred with letterboxing.
    #[must_use]
    pub fn viewport(self, window: Vec2) -> Rect {
        let scale = self.window_scale(window) as f32;
        let size = self.resolution * scale;
        Rect::from_position_size(((window - size) * 0.5).floor(), size)
    }

    /// Suit `target` en douceur, independamment de la frequence d'images —
    /// contrairement a un `lerp` par frame, qui suit plus vite sur une
    /// machine rapide.
    pub fn follow(&mut self, target: Vec2, smoothing: f32, dt: f32) {
        if smoothing <= 0.0 {
            self.position = target;
            return;
        }
        let t = 1.0 - (-dt / smoothing).exp();
        self.position = self.position.lerp(target, t);
    }
}
