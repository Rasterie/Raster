use raster_math::IVec2;
use raster_render::Colour;

/// A grid of pixels: one frame, one layer.
///
/// Le modele que le sprite et l'animation partagent : une animation est une
/// suite de frames, un sprite en a une seule, et les deux editeurs
/// manipulent la meme chose.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    size: IVec2,
    /// Les pixels, ligne par ligne, en RGBA.
    pixels: Vec<Colour>,
}

impl Frame {
    /// # Panics
    ///
    /// Si une dimension est nulle ou negative : une frame vide n'a pas de sens
    /// et masquerait une erreur de calcul plus haut.
    #[must_use]
    pub fn new(width: i32, height: i32) -> Self {
        assert!(
            width > 0 && height > 0,
            "a frame cannot be {width}x{height}"
        );

        Self {
            size: IVec2::new(width, height),
            pixels: vec![Colour::TRANSPARENT; (width * height) as usize],
        }
    }

    /// A frame filled with one colour.
    #[must_use]
    pub fn filled(width: i32, height: i32, colour: Colour) -> Self {
        let mut frame = Self::new(width, height);
        frame.pixels.fill(colour);
        frame
    }

    #[must_use]
    pub fn size(&self) -> IVec2 {
        self.size
    }

    #[must_use]
    pub fn width(&self) -> i32 {
        self.size.x
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        self.size.y
    }

    /// The colour of a pixel, `None` outside the frame.
    #[must_use]
    pub fn get(&self, at: IVec2) -> Option<Colour> {
        self.index(at).map(|i| self.pixels[i])
    }

    /// Sets a pixel, returning whether it was inside.
    ///
    /// Hors cadre n'est pas une erreur : un pinceau qui deborde doit simplement
    /// ne rien poser dehors.
    pub fn set(&mut self, at: IVec2, colour: Colour) -> bool {
        match self.index(at) {
            Some(i) => {
                self.pixels[i] = colour;
                true
            }
            None => false,
        }
    }

    fn index(&self, at: IVec2) -> Option<usize> {
        if at.x < 0 || at.y < 0 || at.x >= self.size.x || at.y >= self.size.y {
            return None;
        }
        Some((at.y * self.size.x + at.x) as usize)
    }

    pub fn fill(&mut self, colour: Colour) {
        self.pixels.fill(colour);
    }

    pub fn clear(&mut self) {
        self.fill(Colour::TRANSPARENT);
    }

    #[must_use]
    pub fn pixels(&self) -> &[Colour] {
        &self.pixels
    }

    /// The pixels as RGBA bytes, ready to upload as a texture.
    #[must_use]
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixels.len() * 4);

        for pixel in &self.pixels {
            for channel in pixel.to_array() {
                // Borne avant conversion : une couleur hors [0,1] deborderait
                // en boucle, et un blanc deviendrait noir.
                out.push((channel.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }

        out
    }

    /// Reads a frame back from RGBA bytes.
    ///
    /// `None` si la longueur ne correspond pas : mieux vaut refuser qu'afficher
    /// une image decalee.
    #[must_use]
    pub fn from_rgba(width: i32, height: i32, bytes: &[u8]) -> Option<Self> {
        if width <= 0 || height <= 0 {
            return None;
        }
        let expected = (width * height) as usize * 4;
        if bytes.len() != expected {
            return None;
        }

        let mut frame = Self::new(width, height);
        for (i, chunk) in bytes.chunks_exact(4).enumerate() {
            frame.pixels[i] = Colour::rgba(
                f32::from(chunk[0]) / 255.0,
                f32::from(chunk[1]) / 255.0,
                f32::from(chunk[2]) / 255.0,
                f32::from(chunk[3]) / 255.0,
            );
        }

        Some(frame)
    }

    /// Resizes, keeping what fits in the top-left corner.
    ///
    /// Ancre en haut a gauche plutot que centree : c'est ce qu'un editeur de
    /// pixel art attend, et le centrage decalerait tout d'un demi-pixel sur
    /// une dimension impaire.
    pub fn resize(&mut self, width: i32, height: i32) {
        if width <= 0 || height <= 0 {
            return;
        }

        let mut next = Self::new(width, height);
        for y in 0..self.size.y.min(height) {
            for x in 0..self.size.x.min(width) {
                let at = IVec2::new(x, y);
                if let Some(colour) = self.get(at) {
                    next.set(at, colour);
                }
            }
        }

        *self = next;
    }

    /// Whether every pixel is transparent.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.pixels.iter().all(|p| p.to_array()[3] <= 0.0)
    }
}

/// A sequence of frames: what the animation editor edits.
#[derive(Debug, Clone, PartialEq)]
pub struct Frames {
    frames: Vec<Frame>,
    /// La frame montree, celle qu'un outil modifie.
    current: usize,
}

impl Frames {
    #[must_use]
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            frames: vec![Frame::new(width, height)],
            current: 0,
        }
    }

    #[must_use]
    pub fn current(&self) -> &Frame {
        &self.frames[self.current]
    }

    pub fn current_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.current]
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.current
    }

    pub fn select(&mut self, index: usize) {
        self.current = index.min(self.frames.len() - 1);
    }

    /// Adds an empty frame after the current one.
    pub fn insert_after(&mut self) -> usize {
        let size = self.frames[self.current].size();
        self.frames
            .insert(self.current + 1, Frame::new(size.x, size.y));
        self.current += 1;
        self.current
    }

    /// Copies the current frame, which is how an animation is drawn.
    pub fn duplicate(&mut self) -> usize {
        let copy = self.frames[self.current].clone();
        self.frames.insert(self.current + 1, copy);
        self.current += 1;
        self.current
    }

    /// Removes a frame, refusing to leave none.
    pub fn remove(&mut self, index: usize) -> bool {
        if self.frames.len() <= 1 || index >= self.frames.len() {
            return false;
        }
        self.frames.remove(index);
        self.current = self.current.min(self.frames.len() - 1);
        true
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Frame> {
        self.frames.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Frame> {
        self.frames.iter()
    }
}
