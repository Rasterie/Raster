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
    /// Deliberately not snapped to the pixel grid. Sprites snap; the camera
    /// does not. Snapping both makes scrolling stutter, snapping neither makes
    /// it blurry — this pair is the most common thing 2D engines get wrong.
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

    /// The matrix mapping world space to clip space.
    ///
    /// Clip space runs from -1 to 1 with Y upwards, while the engine works in
    /// pixels with Y downwards — hence the negated Y scale.
    #[must_use]
    pub fn view_projection(self) -> Mat3 {
        let zoom = self.zoom.max(1) as f32;
        let half = self.resolution * 0.5 / zoom;

        // La camera n'est pas accrochee a la grille, mais la matrice l'est au
        // demi-pixel pres : sans cela, une resolution impaire decalerait tout
        // le rendu d'un demi-pixel et brouillerait chaque sprite.
        let offset = self.position;

        Mat3::new(
            Vec2::new(1.0 / half.x, 0.0),
            Vec2::new(0.0, -1.0 / half.y),
            Vec2::new(-offset.x / half.x, offset.y / half.y),
        )
    }

    /// The area of the world this camera can see.
    ///
    /// The renderer culls against it, so anything outside costs nothing.
    #[must_use]
    pub fn visible_area(self) -> Rect {
        let zoom = self.zoom.max(1) as f32;
        let size = self.resolution / zoom;
        Rect::from_center_size(self.position, size)
    }

    /// Converts a point on screen to a point in the world.
    ///
    /// `screen` is in window pixels; `window` is the window's size. Used for
    /// mouse picking.
    #[must_use]
    pub fn screen_to_world(self, screen: Vec2, window: Vec2) -> Vec2 {
        let scale = self.window_scale(window) as f32;
        let zoom = self.zoom.max(1) as f32;

        // Retire le letterboxing avant de convertir, sinon un clic dans les
        // bandes noires donnerait des coordonnees hors du monde.
        let scaled = self.resolution * scale;
        let margin = (window - scaled) * 0.5;
        let inside = (screen - margin) / scale;

        self.position + (inside - self.resolution * 0.5) / zoom
    }

    /// The integer factor by which the low-resolution image is scaled up to
    /// fill `window`.
    ///
    /// Always a whole number: a 3.7× scale makes some pixels wider than others,
    /// and there is no version of that which looks acceptable.
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

    /// Moves the camera towards `target`, smoothly, in world units per second.
    ///
    /// Frame-rate independent: the same motion at 30fps and 240fps. A naive
    /// `lerp(position, target, 0.1)` per frame is not, and produces a camera
    /// that follows faster on a better machine.
    pub fn follow(&mut self, target: Vec2, smoothing: f32, dt: f32) {
        if smoothing <= 0.0 {
            self.position = target;
            return;
        }
        let t = 1.0 - (-dt / smoothing).exp();
        self.position = self.position.lerp(target, t);
    }
}
