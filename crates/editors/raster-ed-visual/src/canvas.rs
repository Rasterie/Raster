use raster_math::{IVec2, Rect, Vec2};

/// The surface every Visual editor draws on.
///
/// Distinct du viewport de scene : celui-ci travaille en pixels d'image, sur
/// une grille finie, et le zoom y va jusqu'a voir un pixel comme un carre.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    /// La taille de l'image, en pixels.
    pub size: IVec2,
    /// Le coin haut-gauche visible, en pixels d'image.
    pub origin: Vec2,
    /// Combien de pixels d'ecran fait un pixel d'image.
    pub zoom: u32,
    pub show_grid: bool,
    /// Le damier sous les zones transparentes.
    pub show_checker: bool,
    pub show_rulers: bool,
}

impl Canvas {
    /// Au-dela, un pixel occupe la moitie de l'ecran.
    pub const MAX_ZOOM: u32 = 64;

    /// La grille n'apparait qu'a partir de la : plus bas, elle mangerait
    /// l'image au lieu de la decouper.
    pub const GRID_FROM: u32 = 4;

    /// Le cote d'une case du damier, en pixels d'ecran.
    pub const CHECKER: f32 = 8.0;

    #[must_use]
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            size: IVec2::new(width.max(1), height.max(1)),
            origin: Vec2::ZERO,
            zoom: 8,
            show_grid: true,
            show_checker: true,
            show_rulers: false,
        }
    }

    /// Turns a point on screen into a pixel of the image.
    ///
    /// Le pixel contenant le point : `floor`, pas `round`, sinon la moitie
    /// droite d'un pixel peindrait le suivant.
    #[must_use]
    pub fn to_pixel(&self, area: Rect, at: Vec2) -> IVec2 {
        let local = (at - area.position) / self.scale() + self.origin;
        IVec2::new(local.x.floor() as i32, local.y.floor() as i32)
    }

    /// The screen position of a pixel's top-left corner.
    #[must_use]
    pub fn to_screen(&self, area: Rect, pixel: IVec2) -> Vec2 {
        let local = Vec2::new(pixel.x as f32, pixel.y as f32) - self.origin;
        area.position + local * self.scale()
    }

    /// The screen rectangle one pixel covers.
    #[must_use]
    pub fn pixel_rect(&self, area: Rect, pixel: IVec2) -> Rect {
        let at = self.to_screen(area, pixel);
        Rect::new(at.x, at.y, self.scale(), self.scale())
    }

    #[must_use]
    pub fn scale(&self) -> f32 {
        self.zoom.max(1) as f32
    }

    /// Whether a pixel is inside the image.
    #[must_use]
    pub fn contains(&self, pixel: IVec2) -> bool {
        pixel.x >= 0 && pixel.y >= 0 && pixel.x < self.size.x && pixel.y < self.size.y
    }

    /// The pixels visible in `area`, clamped to the image.
    ///
    /// Ce qu'un dessin parcourt : sans cette borne, une image de seize pixels
    /// zoomee ferait boucler sur des milliers de cases vides.
    #[must_use]
    pub fn visible_pixels(&self, area: Rect) -> (IVec2, IVec2) {
        let scale = self.scale();
        let first = IVec2::new(
            self.origin.x.floor().max(0.0) as i32,
            self.origin.y.floor().max(0.0) as i32,
        );
        let last = IVec2::new(
            ((self.origin.x + area.size.x / scale).ceil() as i32).min(self.size.x),
            ((self.origin.y + area.size.y / scale).ceil() as i32).min(self.size.y),
        );
        (first, last)
    }

    /// Zooms, keeping the pixel under the cursor where it is.
    pub fn zoom_at(&mut self, area: Rect, at: Vec2, by: i32) {
        let before = (at - area.position) / self.scale() + self.origin;

        let next = if by > 0 {
            self.zoom.saturating_mul(2)
        } else {
            self.zoom / 2
        };
        self.zoom = next.clamp(1, Self::MAX_ZOOM);

        let after = (at - area.position) / self.scale() + self.origin;
        self.origin += before - after;
    }

    /// Moves the view by a screen offset.
    pub fn pan(&mut self, by: Vec2) {
        self.origin -= by / self.scale();
    }

    /// Centres the image in `area`, at a zoom that fits.
    ///
    /// Ce qu'un editeur fait a l'ouverture : montrer l'image entiere, aussi
    /// grande que possible sans la couper.
    pub fn fit(&mut self, area: Rect) {
        let by_width = area.size.x / self.size.x.max(1) as f32;
        let by_height = area.size.y / self.size.y.max(1) as f32;

        // Zoom entier : un demi-pixel rendrait l'image floue.
        let fitting = by_width.min(by_height).floor().max(1.0) as u32;
        self.zoom = fitting.clamp(1, Self::MAX_ZOOM);
        self.centre(area);
    }

    /// Centres the image without changing the zoom.
    pub fn centre(&mut self, area: Rect) {
        let visible = area.size / self.scale();
        self.origin = Vec2::new(
            (self.size.x as f32 - visible.x) / 2.0,
            (self.size.y as f32 - visible.y) / 2.0,
        );
    }

    /// Whether the pixel grid should be drawn at this zoom.
    #[must_use]
    pub fn grid_visible(&self) -> bool {
        self.show_grid && self.zoom >= Self::GRID_FROM
    }

    /// The image's rectangle on screen.
    #[must_use]
    pub fn image_rect(&self, area: Rect) -> Rect {
        let at = self.to_screen(area, IVec2::ZERO);
        Rect::new(
            at.x,
            at.y,
            self.size.x as f32 * self.scale(),
            self.size.y as f32 * self.scale(),
        )
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new(32, 32)
    }
}

/// Whether a checker square at this position is the light one.
///
/// Le damier se calcule en pixels d'ecran, pas d'image : il doit rester de
/// taille constante quel que soit le zoom, sinon il devient un motif de
/// l'image elle-meme.
#[must_use]
pub fn checker_light(x: f32, y: f32) -> bool {
    let cx = (x / Canvas::CHECKER).floor() as i64;
    let cy = (y / Canvas::CHECKER).floor() as i64;
    (cx + cy).rem_euclid(2) == 0
}
