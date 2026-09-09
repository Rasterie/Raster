use crate::text::{self, Align};
use raster_math::{Rect, Vec2};
use raster_render::{Colour, Layer, SpriteBatch, SpriteDraw};

/// Draws text and rectangles over the sprite batcher.
///
/// Tout part d'une texture blanche d'un pixel, etiree et teintee : une lettre
/// est faite de rectangles, un cadre aussi, et le batcher les regroupe tous.
#[derive(Debug, Clone, Copy)]
pub struct Painter {
    /// L'indice de la texture blanche dans la liste passee au batcher.
    white: usize,
    /// La couche ou l'interface se dessine, au-dessus du jeu.
    layer: Layer,
    scale: f32,
}

impl Painter {
    /// L'interface se dessine au-dessus de tout le reste.
    pub const LAYER: Layer = Layer::UI;

    #[must_use]
    pub fn new(white: usize) -> Self {
        Self {
            white,
            layer: Self::LAYER,
            scale: 1.0,
        }
    }

    /// Un texte a l'echelle 2 ou 3 reste net : le pixel art n'interpole pas.
    #[must_use]
    pub fn scaled(mut self, scale: f32) -> Self {
        self.scale = scale.max(1.0);
        self
    }

    #[must_use]
    pub fn on_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Fills a rectangle.
    pub fn rect(&self, batch: &mut SpriteBatch, at: Rect, colour: Colour) {
        if at.size.x <= 0.0 || at.size.y <= 0.0 {
            return;
        }

        batch.draw(
            self.white,
            SpriteDraw {
                position: at.position,
                size: at.size,
                tint: colour,
                layer: self.layer,
                ..SpriteDraw::new(at.position, at.size)
            },
        );
    }

    /// Draws a one-pixel outline, inside the rectangle.
    ///
    /// A l'interieur : un cadre qui deborde ferait grossir chaque element d'un
    /// pixel de chaque cote, et les alignements ne tomberaient plus juste.
    pub fn outline(&self, batch: &mut SpriteBatch, at: Rect, colour: Colour) {
        let (w, h) = (at.size.x, at.size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let t = self.scale;
        self.rect(batch, Rect::new(at.position.x, at.position.y, w, t), colour);
        self.rect(
            batch,
            Rect::new(at.position.x, at.position.y + h - t, w, t),
            colour,
        );
        self.rect(batch, Rect::new(at.position.x, at.position.y, t, h), colour);
        self.rect(
            batch,
            Rect::new(at.position.x + w - t, at.position.y, t, h),
            colour,
        );
    }

    /// Draws text, returning the area it covered.
    ///
    /// Chaque pixel allume devient un rectangle : a 5x7 une lettre coute au
    /// plus trente-cinq quads, que le batcher regroupe en un seul appel.
    pub fn text(
        &self,
        batch: &mut SpriteBatch,
        at: Vec2,
        content: &str,
        align: Align,
        colour: Colour,
    ) -> Rect {
        let scale = self.scale;

        for placed in text::layout(content, at, align, scale) {
            for (x, y) in text::glyph_pixels(placed.c) {
                self.rect(
                    batch,
                    Rect::new(
                        placed.at.x + x as f32 * scale,
                        placed.at.y + y as f32 * scale,
                        scale,
                        scale,
                    ),
                    colour,
                );
            }
        }

        self.text_bounds(at, content, align)
    }

    /// The area text would cover, without drawing it.
    ///
    /// Ce qu'il faut pour placer un cadre autour d'un libelle avant de savoir
    /// s'il tient.
    #[must_use]
    pub fn text_bounds(&self, at: Vec2, content: &str, align: Align) -> Rect {
        let size = text::measure(content) * self.scale;
        let x = match align {
            Align::Left => at.x,
            Align::Centre => at.x - size.x / 2.0,
            Align::Right => at.x - size.x,
        };
        Rect::new(x, at.y, size.x, size.y)
    }

    /// The size text occupies at this painter's scale.
    #[must_use]
    pub fn measure(&self, content: &str) -> Vec2 {
        text::measure(content) * self.scale
    }
}
