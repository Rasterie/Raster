use raster_ed_visual::Frame;
use raster_math::IVec2;
use raster_render::Colour;

/// How a neighbouring frame shows through while animating.
///
/// Ce qui permet de dessiner une frame en voyant la precedente : sans cela,
/// une animation se construit a l'aveugle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Onion {
    /// Combien de frames avant sont montrees.
    pub before: usize,
    /// Et combien apres.
    pub after: usize,
    /// L'opacite de la plus proche ; les suivantes s'effacent.
    pub opacity: f32,
    pub enabled: bool,
}

impl Default for Onion {
    fn default() -> Self {
        Self {
            before: 1,
            after: 0,
            opacity: 0.35,
            enabled: false,
        }
    }
}

impl Onion {
    /// The opacity a frame `distance` away should be drawn at.
    ///
    /// Zero quand elle est hors de portee : chaque frame plus loin s'efface,
    /// sinon une longue animation deviendrait une bouillie grise.
    #[must_use]
    pub fn opacity_at(&self, distance: i32) -> f32 {
        if !self.enabled || distance == 0 {
            return 0.0;
        }

        let reach = if distance < 0 {
            self.before as i32
        } else {
            self.after as i32
        };

        if distance.abs() > reach {
            return 0.0;
        }

        self.opacity / distance.abs() as f32
    }

    /// The tint a frame is drawn with: cold before, warm after.
    ///
    /// La couleur dit le sens du temps : sans elle, on ne sait pas si la
    /// silhouette qu'on voit vient d'avant ou d'apres.
    #[must_use]
    pub fn tint(distance: i32) -> Colour {
        if distance < 0 {
            Colour::rgb(0.45, 0.60, 1.0)
        } else {
            Colour::rgb(1.0, 0.55, 0.40)
        }
    }
}

/// How the frames of a spritesheet are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    Row,
    Column,
    /// Une grille de `columns` de large.
    Grid,
}

/// Packs frames into one image.
///
/// # Errors
///
/// If there is nothing to pack, or the frames disagree on their size.
pub fn pack(frames: &[Frame], layout: Layout, columns: usize) -> Result<Frame, SheetError> {
    let first = frames.first().ok_or(SheetError::Empty)?;
    let (w, h) = (first.width(), first.height());

    if frames.iter().any(|f| f.width() != w || f.height() != h) {
        return Err(SheetError::Mismatched);
    }

    let columns = match layout {
        Layout::Row => frames.len(),
        Layout::Column => 1,
        Layout::Grid => columns.max(1).min(frames.len()),
    };
    let rows = frames.len().div_ceil(columns);

    let mut sheet = Frame::new(w * columns as i32, h * rows as i32);

    for (index, frame) in frames.iter().enumerate() {
        let col = (index % columns) as i32;
        let row = (index / columns) as i32;

        for y in 0..h {
            for x in 0..w {
                if let Some(colour) = frame.get(IVec2::new(x, y)) {
                    sheet.set(IVec2::new(col * w + x, row * h + y), colour);
                }
            }
        }
    }

    Ok(sheet)
}

/// Cuts a spritesheet back into frames.
///
/// # Errors
///
/// If the sheet does not divide evenly.
pub fn unpack(
    sheet: &Frame,
    frame_width: i32,
    frame_height: i32,
) -> Result<Vec<Frame>, SheetError> {
    if frame_width <= 0 || frame_height <= 0 {
        return Err(SheetError::Empty);
    }
    if sheet.width() % frame_width != 0 || sheet.height() % frame_height != 0 {
        return Err(SheetError::Mismatched);
    }

    let columns = sheet.width() / frame_width;
    let rows = sheet.height() / frame_height;
    let mut frames = Vec::with_capacity((columns * rows) as usize);

    for row in 0..rows {
        for col in 0..columns {
            let mut frame = Frame::new(frame_width, frame_height);

            for y in 0..frame_height {
                for x in 0..frame_width {
                    let from = IVec2::new(col * frame_width + x, row * frame_height + y);
                    if let Some(colour) = sheet.get(from) {
                        frame.set(IVec2::new(x, y), colour);
                    }
                }
            }

            frames.push(frame);
        }
    }

    Ok(frames)
}

/// Why a spritesheet could not be built or read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetError {
    Empty,
    /// Les frames n'ont pas toutes la meme taille, ou la feuille ne se divise
    /// pas en frames entieres.
    Mismatched,
}

impl std::fmt::Display for SheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "aucune frame a assembler"),
            Self::Mismatched => write!(f, "les frames ne s'assemblent pas"),
        }
    }
}

impl std::error::Error for SheetError {}
