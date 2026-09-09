use crate::font::{self, ADVANCE, GLYPH_HEIGHT, LINE_HEIGHT};
use raster_math::Vec2;

/// Where text sits relative to the position it is drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Centre,
    Right,
}

/// The size a string occupies, in pixels, at scale 1.
///
/// Mesure avant de dessiner : sans cela rien ne se centre ni ne se cadre.
#[must_use]
pub fn measure(text: &str) -> Vec2 {
    let mut widest = 0u32;
    let mut lines = 0u32;

    for line in text.split('\n') {
        widest = widest.max(line_width(line));
        lines += 1;
    }

    Vec2::new(
        widest as f32,
        (lines * LINE_HEIGHT).saturating_sub(1) as f32,
    )
}

/// The width of one line, without the trailing space of the last glyph.
#[must_use]
pub fn line_width(line: &str) -> u32 {
    let n = line.chars().count() as u32;
    if n == 0 { 0 } else { n * ADVANCE - 1 }
}

/// The number of lines a string draws as.
#[must_use]
pub fn line_count(text: &str) -> usize {
    text.split('\n').count()
}

/// One glyph, placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub c: char,
    /// Le coin haut-gauche du glyphe, en pixels.
    pub at: Vec2,
}

/// Walks a string, handing back where each glyph goes.
///
/// Les espaces et les retours a la ligne avancent le curseur sans rien rendre :
/// l'appelant dessine tout ce qu'il recoit.
pub fn layout(text: &str, at: Vec2, align: Align, scale: f32) -> Vec<Placed> {
    let mut placed = Vec::with_capacity(text.len());
    let scale = scale.max(0.0);

    for (row, line) in text.split('\n').enumerate() {
        let width = line_width(line) as f32 * scale;
        let x = match align {
            Align::Left => at.x,
            Align::Centre => at.x - width / 2.0,
            Align::Right => at.x - width,
        };
        let y = at.y + row as f32 * LINE_HEIGHT as f32 * scale;

        for (column, c) in line.chars().enumerate() {
            if c == ' ' {
                continue;
            }
            placed.push(Placed {
                c,
                at: Vec2::new(x + column as f32 * ADVANCE as f32 * scale, y),
            });
        }
    }

    placed
}

/// The pixels of a glyph, as offsets from its corner.
///
/// Un moteur qui n'a pas encore d'atlas dessine les lettres pixel par pixel :
/// a 5x7, c'est trente-cinq tests par lettre, moins que d'assembler une texture.
pub fn glyph_pixels(c: char) -> Vec<(u32, u32)> {
    let mut pixels = Vec::new();
    for y in 0..GLYPH_HEIGHT {
        for x in 0..font::GLYPH_WIDTH {
            if font::pixel(c, x, y) {
                pixels.push((x, y));
            }
        }
    }
    pixels
}

/// Whether every character in a string can be drawn.
#[must_use]
pub fn is_drawable(text: &str) -> bool {
    text.chars()
        .all(|c| c == '\n' || c == ' ' || font::glyph(c).is_some())
}

/// Replaces what the font cannot draw, so a label never comes out blank.
#[must_use]
pub fn sanitise(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c == '\n' || c == ' ' || font::glyph(c).is_some() {
                c
            } else {
                '?'
            }
        })
        .collect()
}
