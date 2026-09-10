use raster_render::Colour;

/// A colour in a palette, with its slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swatch {
    pub colour: Colour,
    /// Le nom du slot, pour les palettes ou les couleurs ont un role.
    pub name: Option<&'static str>,
}

impl Swatch {
    #[must_use]
    pub fn new(colour: Colour) -> Self {
        Self { colour, name: None }
    }
}

/// The colours an editor works with.
///
/// Partagee par tous les editeurs visuels : un sprite, une tuile et une
/// interface d'un meme projet doivent tirer des memes couleurs, sinon la
/// direction artistique se disperse.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub name: String,
    swatches: Vec<Swatch>,
    /// La couleur active, celle qu'un outil pose.
    primary: usize,
    /// Celle du clic droit, souvent la transparence ou le fond.
    secondary: usize,
}

impl Palette {
    /// Au-dela, une palette n'est plus une palette mais une image.
    pub const MAX: usize = 256;

    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            swatches: Vec::new(),
            primary: 0,
            secondary: 0,
        }
    }

    /// Une palette de depart : seize teintes lisibles sur fond sombre.
    #[must_use]
    pub fn default_sixteen() -> Self {
        let mut palette = Self::new("defaut");

        for [r, g, b] in [
            [0.06, 0.05, 0.09],
            [0.16, 0.15, 0.22],
            [0.30, 0.29, 0.38],
            [0.52, 0.52, 0.60],
            [0.78, 0.79, 0.84],
            [0.96, 0.96, 0.98],
            [0.72, 0.24, 0.28],
            [0.90, 0.42, 0.32],
            [0.95, 0.72, 0.32],
            [0.98, 0.90, 0.55],
            [0.40, 0.68, 0.36],
            [0.22, 0.46, 0.32],
            [0.28, 0.52, 0.78],
            [0.42, 0.76, 0.90],
            [0.52, 0.34, 0.68],
            [0.82, 0.50, 0.70],
        ] {
            palette.push(Colour::rgb(r, g, b));
        }

        palette
    }

    /// Adds a colour, up to the limit.
    ///
    /// Renvoie son indice, ou `None` si la palette est pleine.
    pub fn push(&mut self, colour: Colour) -> Option<usize> {
        if self.swatches.len() >= Self::MAX {
            return None;
        }
        self.swatches.push(Swatch::new(colour));
        Some(self.swatches.len() - 1)
    }

    /// Removes a colour, keeping the selection valid.
    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.swatches.len() {
            return false;
        }
        self.swatches.remove(index);

        // La selection suit : sans cela elle pointerait au-dela de la liste,
        // ou sur une couleur differente de celle qu'on regardait.
        self.primary = clamp_after_removal(self.primary, index, self.swatches.len());
        self.secondary = clamp_after_removal(self.secondary, index, self.swatches.len());
        true
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<Colour> {
        self.swatches.get(index).map(|s| s.colour)
    }

    pub fn set(&mut self, index: usize, colour: Colour) -> bool {
        match self.swatches.get_mut(index) {
            Some(swatch) => {
                swatch.colour = colour;
                true
            }
            None => false,
        }
    }

    #[must_use]
    pub fn swatches(&self) -> &[Swatch] {
        &self.swatches
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.swatches.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.swatches.is_empty()
    }

    pub fn select(&mut self, index: usize) {
        if index < self.swatches.len() {
            self.primary = index;
        }
    }

    pub fn select_secondary(&mut self, index: usize) {
        if index < self.swatches.len() {
            self.secondary = index;
        }
    }

    /// The colour a tool paints with.
    #[must_use]
    pub fn primary(&self) -> Colour {
        self.get(self.primary).unwrap_or(Colour::TRANSPARENT)
    }

    #[must_use]
    pub fn secondary(&self) -> Colour {
        self.get(self.secondary).unwrap_or(Colour::TRANSPARENT)
    }

    #[must_use]
    pub fn primary_index(&self) -> usize {
        self.primary
    }

    /// Swaps the two selected colours.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.primary, &mut self.secondary);
    }

    /// The closest colour in the palette, for quantising an imported image.
    ///
    /// Distance dans l'espace RGB : approximatif, mais suffisant pour ranger
    /// un pixel dans une palette de seize teintes.
    #[must_use]
    pub fn nearest(&self, colour: Colour) -> Option<usize> {
        let [r, g, b, _] = colour.to_array();

        self.swatches
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, other)| {
                distance(a.colour, r, g, b).total_cmp(&distance(other.colour, r, g, b))
            })
            .map(|(i, _)| i)
    }
}

/// Un indice apres la suppression de `removed`, borne a la nouvelle taille.
fn clamp_after_removal(index: usize, removed: usize, len: usize) -> usize {
    let shifted = if index > removed { index - 1 } else { index };
    shifted.min(len.saturating_sub(1))
}

fn distance(colour: Colour, r: f32, g: f32, b: f32) -> f32 {
    let [cr, cg, cb, _] = colour.to_array();
    (cr - r).powi(2) + (cg - g).powi(2) + (cb - b).powi(2)
}

impl Default for Palette {
    fn default() -> Self {
        Self::default_sixteen()
    }
}
