use crate::layout::inset;
use crate::text::Align;
use crate::{Id, Painter, Response, Ui};
use raster_math::{Rect, Vec2};
use raster_render::SpriteBatch;

/// Draws a label, which nothing can interact with.
pub fn label(
    ui: &Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    at: Vec2,
    text: &str,
    align: Align,
) -> Rect {
    painter.text(batch, at, text, align, ui.theme().text)
}

/// Draws a muted label, for secondary text.
pub fn hint(
    ui: &Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    at: Vec2,
    text: &str,
    align: Align,
) -> Rect {
    painter.text(batch, at, text, align, ui.theme().muted)
}

/// A clickable button.
pub fn button(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    text: &str,
) -> Response {
    button_enabled(ui, batch, painter, id, area, text, true)
}

/// A button that may be greyed out.
pub fn button_enabled(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    text: &str,
    enabled: bool,
) -> Response {
    let response = ui.interact(id, area, enabled);
    let theme = *ui.theme();
    let state = response.state();

    painter.rect(batch, area, theme.surface_for(state));
    painter.outline(
        batch,
        area,
        if response.focused {
            theme.focus
        } else {
            theme.border
        },
    );

    // Le texte s'enfonce d'un pixel quand le bouton est presse : le retour
    // visuel compte autant que la couleur.
    let sink = if response.held { painter.scale() } else { 0.0 };
    let centre = Vec2::new(
        area.position.x + area.size.x / 2.0,
        area.position.y + (area.size.y - painter.measure(text).y) / 2.0 + sink,
    );

    painter.text(batch, centre, text, Align::Centre, theme.text_for(state));
    response
}

/// A box that is either ticked or not.
pub fn checkbox(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    text: &str,
    checked: &mut bool,
) -> Response {
    let response = ui.interact(id, area, true);
    if response.clicked {
        *checked = !*checked;
    }

    let theme = *ui.theme();
    let state = response.state();
    let side = area.size.y.min(theme.row_height);
    let box_area = Rect::new(area.position.x, area.position.y, side, side);

    painter.rect(batch, box_area, theme.surface_for(state));
    painter.outline(
        batch,
        box_area,
        if response.focused {
            theme.focus
        } else {
            theme.border
        },
    );

    if *checked {
        painter.rect(batch, inset(box_area, side * 0.28), theme.accent);
    }

    painter.text(
        batch,
        Vec2::new(
            area.position.x + side + theme.gap,
            area.position.y + (side - painter.measure(text).y) / 2.0,
        ),
        text,
        Align::Left,
        theme.text_for(state),
    );

    response
}

/// A value dragged between two bounds.
///
/// Renvoie `true` si la valeur a change : un appelant peut ainsi reagir au
/// glissement sans comparer lui-meme.
pub fn slider(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    range: (f32, f32),
    value: &mut f32,
) -> bool {
    let response = ui.interact(id, area, true);
    let theme = *ui.theme();
    let (low, high) = (range.0, range.1.max(range.0 + f32::EPSILON));
    let before = *value;

    if response.held && area.size.x > 0.0 {
        let t = ((ui.pointer().at.x - area.position.x) / area.size.x).clamp(0.0, 1.0);
        *value = low + t * (high - low);
    }

    // Au clavier, un pas de un centieme de la course.
    if response.focused {
        let step = (high - low) / 100.0;
        if ui.keys().left {
            *value -= step;
        }
        if ui.keys().right {
            *value += step;
        }
    }

    *value = value.clamp(low, high);
    let t = (*value - low) / (high - low);

    painter.rect(batch, area, theme.pressed);
    painter.rect(
        batch,
        Rect::new(
            area.position.x,
            area.position.y,
            area.size.x * t,
            area.size.y,
        ),
        theme.accent,
    );
    painter.outline(
        batch,
        area,
        if response.focused {
            theme.focus
        } else {
            theme.border
        },
    );

    (*value - before).abs() > f32::EPSILON
}

/// A panel: a filled, outlined rectangle to group things in.
pub fn panel(ui: &Ui, batch: &mut SpriteBatch, painter: &Painter, area: Rect) -> Rect {
    let theme = ui.theme();
    painter.rect(batch, area, theme.surface);
    painter.outline(batch, area, theme.border);
    inset(area, theme.padding)
}

/// A bar showing a fraction, such as health.
pub fn progress(
    ui: &Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    area: Rect,
    fraction: f32,
    colour: raster_render::Colour,
) {
    let theme = ui.theme();
    let t = fraction.clamp(0.0, 1.0);

    painter.rect(batch, area, theme.pressed);
    painter.rect(
        batch,
        Rect::new(
            area.position.x,
            area.position.y,
            area.size.x * t,
            area.size.y,
        ),
        colour,
    );
    painter.outline(batch, area, theme.border);
}
