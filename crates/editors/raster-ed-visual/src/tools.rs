use crate::Frame;
use raster_math::IVec2;
use raster_render::Colour;

/// What a tool does to a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Brush,
    Eraser,
    /// Remplit une zone de meme couleur.
    Fill,
    /// Prend la couleur sous le curseur.
    Picker,
    Line,
    Rectangle,
    Ellipse,
    /// Selectionne une zone, sans rien peindre.
    Select,
}

impl Tool {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Brush => "pinceau",
            Self::Eraser => "gomme",
            Self::Fill => "remplissage",
            Self::Picker => "pipette",
            Self::Line => "ligne",
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Select => "selection",
        }
    }

    /// Whether the tool paints continuously as the cursor moves.
    ///
    /// Un pinceau suit le curseur ; une ligne attend qu'on relache pour savoir
    /// ou elle va.
    #[must_use]
    pub fn is_continuous(self) -> bool {
        matches!(self, Self::Brush | Self::Eraser)
    }

    /// Whether it changes pixels at all.
    #[must_use]
    pub fn paints(self) -> bool {
        !matches!(self, Self::Picker | Self::Select)
    }
}

/// Paints a square brush, centred on a pixel.
///
/// Un pinceau de taille paire n'a pas de centre exact : il deborde d'un pixel
/// vers le bas et la droite, comme partout ailleurs.
pub fn brush(frame: &mut Frame, at: IVec2, size: i32, colour: Colour) {
    let size = size.max(1);
    let half = (size - 1) / 2;

    for dy in 0..size {
        for dx in 0..size {
            frame.set(IVec2::new(at.x - half + dx, at.y - half + dy), colour);
        }
    }
}

/// Draws a line between two pixels, without gaps.
///
/// Bresenham : une interpolation flottante laisserait des trous des que la
/// pente depasse un, ce qui se voit immediatement en pixel art.
pub fn line(frame: &mut Frame, from: IVec2, to: IVec2, size: i32, colour: Colour) {
    for at in line_pixels(from, to) {
        brush(frame, at, size, colour);
    }
}

/// The pixels a line covers.
#[must_use]
pub fn line_pixels(from: IVec2, to: IVec2) -> Vec<IVec2> {
    let mut out = Vec::new();

    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };

    let (mut x, mut y) = (from.x, from.y);
    let mut error = dx + dy;

    loop {
        out.push(IVec2::new(x, y));
        if x == to.x && y == to.y {
            break;
        }

        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }

    out
}

/// Fills every pixel connected to `at` that shares its colour.
///
/// Par quatre voisins, pas huit : deux zones qui ne se touchent qu'en diagonale
/// sont deux zones, comme dans tout editeur de pixel art.
pub fn fill(frame: &mut Frame, at: IVec2, colour: Colour) -> usize {
    let Some(target) = frame.get(at) else {
        return 0;
    };
    if same(target, colour) {
        return 0;
    }

    let mut painted = 0;
    let mut queue = vec![at];

    while let Some(pixel) = queue.pop() {
        let Some(current) = frame.get(pixel) else {
            continue;
        };
        if !same(current, target) {
            continue;
        }

        frame.set(pixel, colour);
        painted += 1;

        queue.push(IVec2::new(pixel.x + 1, pixel.y));
        queue.push(IVec2::new(pixel.x - 1, pixel.y));
        queue.push(IVec2::new(pixel.x, pixel.y + 1));
        queue.push(IVec2::new(pixel.x, pixel.y - 1));
    }

    painted
}

/// Draws a rectangle outline.
pub fn rectangle(frame: &mut Frame, from: IVec2, to: IVec2, colour: Colour) {
    let (x0, x1) = (from.x.min(to.x), from.x.max(to.x));
    let (y0, y1) = (from.y.min(to.y), from.y.max(to.y));

    for x in x0..=x1 {
        frame.set(IVec2::new(x, y0), colour);
        frame.set(IVec2::new(x, y1), colour);
    }
    for y in y0..=y1 {
        frame.set(IVec2::new(x0, y), colour);
        frame.set(IVec2::new(x1, y), colour);
    }
}

/// Fills a rectangle.
pub fn rectangle_filled(frame: &mut Frame, from: IVec2, to: IVec2, colour: Colour) {
    let (x0, x1) = (from.x.min(to.x), from.x.max(to.x));
    let (y0, y1) = (from.y.min(to.y), from.y.max(to.y));

    for y in y0..=y1 {
        for x in x0..=x1 {
            frame.set(IVec2::new(x, y), colour);
        }
    }
}

/// Draws an ellipse inscribed in the rectangle the two corners define.
pub fn ellipse(frame: &mut Frame, from: IVec2, to: IVec2, colour: Colour) {
    let (x0, x1) = (from.x.min(to.x), from.x.max(to.x));
    let (y0, y1) = (from.y.min(to.y), from.y.max(to.y));

    let cx = (x0 + x1) as f32 / 2.0;
    let cy = (y0 + y1) as f32 / 2.0;
    let rx = ((x1 - x0) as f32 / 2.0).max(0.5);
    let ry = ((y1 - y0) as f32 / 2.0).max(0.5);

    for y in y0..=y1 {
        for x in x0..=x1 {
            let nx = (x as f32 - cx) / rx;
            let ny = (y as f32 - cy) / ry;
            let d = nx * nx + ny * ny;

            // La bordure : dedans, mais pas trop loin du contour.
            if d <= 1.0 && d >= inner_ratio(rx, ry) {
                frame.set(IVec2::new(x, y), colour);
            }
        }
    }
}

/// L'epaisseur du contour, en fraction du rayon : constante en pixels quelle
/// que soit la taille, sinon une grande ellipse aurait un bord epais.
fn inner_ratio(rx: f32, ry: f32) -> f32 {
    let r = rx.min(ry).max(1.0);
    let inner = (r - 1.0) / r;
    (inner * inner).max(0.0)
}

/// Whether two colours are the same for filling purposes.
///
/// Deux transparents sont egaux quelle que soit leur teinte : un pixel efface
/// n'a pas de couleur visible.
#[must_use]
pub fn same(a: Colour, b: Colour) -> bool {
    let (x, y) = (a.to_array(), b.to_array());

    if x[3] <= 0.0 && y[3] <= 0.0 {
        return true;
    }
    x.iter()
        .zip(y.iter())
        .all(|(p, q)| (p - q).abs() < 1.0 / 512.0)
}
