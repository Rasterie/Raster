use crate::{Layers, Symmetry, symmetry};
use raster_ed_visual::Frame;
use raster_ed_visual::tools::{self, Tool};
use raster_math::IVec2;
use raster_render::Colour;

/// A stroke in progress.
///
/// Un trait est un geste, pas une suite de pixels : il faut savoir ou il a
/// commence pour qu'une ligne ou un rectangle sache ou aller, et pour qu'une
/// annulation defasse tout le geste d'un coup.
#[derive(Debug, Clone)]
pub struct Stroke {
    pub tool: Tool,
    pub colour: Colour,
    pub size: i32,
    pub symmetry: Symmetry,
    /// Ou le geste a commence.
    from: IVec2,
    /// Ou le curseur se trouve.
    to: IVec2,
    /// L'etat du calque avant le geste, pour l'annuler.
    before: Frame,
}

impl Stroke {
    /// Starts a stroke on a layer.
    ///
    /// `None` si le calque ne se modifie pas : cache ou verrouille.
    pub fn begin(
        layers: &mut Layers,
        tool: Tool,
        colour: Colour,
        size: i32,
        symmetry: Symmetry,
        at: IVec2,
    ) -> Option<Self> {
        let before = layers.current_mut()?.frame.clone();

        let stroke = Self {
            tool,
            colour,
            size: size.max(1),
            symmetry,
            from: at,
            to: at,
            before,
        };

        stroke.apply(layers);
        Some(stroke)
    }

    /// Moves the cursor, redrawing the stroke.
    pub fn advance(&mut self, layers: &mut Layers, to: IVec2) {
        // Un pinceau trace en continu ; une forme se recalcule depuis le
        // depart a chaque mouvement.
        if self.tool.is_continuous() {
            let from = self.to;
            self.to = to;
            self.paint_segment(layers, from, to);
        } else {
            self.to = to;
            self.reset(layers);
            self.apply(layers);
        }
    }

    /// The area the stroke covers, for a preview.
    #[must_use]
    pub fn bounds(&self) -> (IVec2, IVec2) {
        (
            IVec2::new(self.from.x.min(self.to.x), self.from.y.min(self.to.y)),
            IVec2::new(self.from.x.max(self.to.x), self.from.y.max(self.to.y)),
        )
    }

    /// The layer as it was before the stroke, to undo it.
    #[must_use]
    pub fn before(&self) -> &Frame {
        &self.before
    }

    #[must_use]
    pub fn start(&self) -> IVec2 {
        self.from
    }

    /// Puts the layer back as it was.
    fn reset(&self, layers: &mut Layers) {
        if let Some(layer) = layers.current_mut() {
            layer.frame = self.before.clone();
        }
    }

    /// Draws the whole stroke from its start.
    fn apply(&self, layers: &mut Layers) {
        let size = layers.size();
        let Some(layer) = layers.current_mut() else {
            return;
        };

        // La gomme efface plutot qu'elle ne peint : c'est la seule difference
        // avec un pinceau.
        let colour = if self.tool == Tool::Eraser {
            Colour::TRANSPARENT
        } else {
            self.colour
        };

        for point in symmetry::mirror(self.from, size, self.symmetry) {
            // Le point d'arrivee suit le meme miroir que le depart, sinon une
            // ligne symetrique partirait de travers.
            let to = mirrored_end(self.from, self.to, point, size, self.symmetry);

            match self.tool {
                Tool::Brush | Tool::Eraser => {
                    tools::brush(&mut layer.frame, point, self.size, colour);
                }
                Tool::Line => tools::line(&mut layer.frame, point, to, self.size, colour),
                Tool::Rectangle => tools::rectangle(&mut layer.frame, point, to, colour),
                Tool::Ellipse => tools::ellipse(&mut layer.frame, point, to, colour),
                Tool::Fill => {
                    tools::fill(&mut layer.frame, point, colour);
                }
                // La pipette et la selection ne peignent pas.
                Tool::Picker | Tool::Select => {}
            }
        }
    }

    /// Paints one segment of a continuous stroke.
    fn paint_segment(&self, layers: &mut Layers, from: IVec2, to: IVec2) {
        let size = layers.size();
        let Some(layer) = layers.current_mut() else {
            return;
        };

        let colour = if self.tool == Tool::Eraser {
            Colour::TRANSPARENT
        } else {
            self.colour
        };

        // Relie les deux positions : sans cela, un mouvement rapide laisserait
        // des trous entre deux frames.
        for point in tools::line_pixels(from, to) {
            for mirrored in symmetry::mirror(point, size, self.symmetry) {
                tools::brush(&mut layer.frame, mirrored, self.size, colour);
            }
        }
    }
}

/// Where a shape's far corner goes, once its start has been mirrored.
fn mirrored_end(
    from: IVec2,
    to: IVec2,
    mirrored_from: IVec2,
    size: IVec2,
    symmetry: Symmetry,
) -> IVec2 {
    // Le miroir applique au depart dit lequel des reflets on trace : on
    // applique le meme au point d'arrivee.
    let candidates = symmetry::mirror(to, size, symmetry);

    // Celui dont le decalage depuis le depart miroir est le plus proche du
    // decalage d'origine, au signe pres.
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    candidates
        .into_iter()
        .min_by_key(|c| {
            let cdx = c.x - mirrored_from.x;
            let cdy = c.y - mirrored_from.y;
            (cdx.abs() - dx.abs()).abs() + (cdy.abs() - dy.abs()).abs()
        })
        .unwrap_or(to)
}

/// Picks the colour under a point, through the visible layers.
///
/// A travers la pile, pas seulement le calque courant : c'est la couleur qu'on
/// voit qu'on veut prendre.
#[must_use]
pub fn pick(layers: &Layers, at: IVec2) -> Option<Colour> {
    let colour = layers.flatten().get(at)?;
    (colour.to_array()[3] > 0.0).then_some(colour)
}
