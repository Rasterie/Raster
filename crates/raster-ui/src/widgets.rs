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

/// A single-line text field.
///
/// Renvoie `true` si le contenu a change cette frame.
pub fn text_field(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    value: &mut String,
) -> bool {
    let response = ui.interact(id, area, true);
    let theme = *ui.theme();
    let before = value.clone();

    if response.focused {
        edit(value, ui.typed(), ui.keys().backspace);
    }

    painter.rect(batch, area, theme.pressed);
    painter.outline(
        batch,
        area,
        if response.focused {
            theme.focus
        } else {
            theme.border
        },
    );

    let inner = inset(area, theme.padding);
    let baseline = area.position.y + (area.size.y - painter.measure(value).y) / 2.0;

    // Coupe par la gauche : c'est la fin qu'on tape, donc la fin qu'on regarde.
    let visible = clip_end(painter, value, inner.size.x);
    painter.text(
        batch,
        Vec2::new(inner.position.x, baseline),
        &visible,
        Align::Left,
        theme.text,
    );

    if response.focused {
        let width = painter.measure(&visible).x;
        painter.rect(
            batch,
            Rect::new(
                inner.position.x + width + 1.0,
                baseline,
                painter.scale(),
                painter.measure("A").y,
            ),
            theme.accent,
        );
    }

    *value != before
}

/// Applies typing to a field's contents.
///
/// Separee du dessin pour etre testable : ce que la police ne sait pas
/// dessiner n'entre pas, sinon l'etiquette sortirait avec des trous.
pub fn edit(value: &mut String, typed: &str, backspace: bool) {
    for c in typed.chars() {
        if crate::font::glyph(c).is_some() {
            value.push(c);
        }
    }
    if backspace {
        value.pop();
    }
}

/// Garde la fin d'un texte qui deborde de `width`.
pub fn clip_end(painter: &Painter, text: &str, width: f32) -> String {
    if painter.measure(text).x <= width {
        return text.to_owned();
    }

    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    while start < chars.len() {
        let tail: String = chars[start..].iter().collect();
        if painter.measure(&tail).x <= width {
            return tail;
        }
        start += 1;
    }
    String::new()
}

/// One of several mutually exclusive choices.
pub fn radio(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    text: &str,
    selected: bool,
) -> Response {
    let response = ui.interact(id, area, true);
    let theme = *ui.theme();
    let state = response.state();
    let side = area.size.y.min(theme.row_height);
    let mark = Rect::new(area.position.x, area.position.y, side, side);

    painter.rect(batch, mark, theme.surface_for(state));
    painter.outline(
        batch,
        mark,
        if response.focused {
            theme.focus
        } else {
            theme.border
        },
    );
    if selected {
        painter.rect(batch, inset(mark, side * 0.3), theme.accent);
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

/// A list of rows, one of which is selected.
///
/// Renvoie l'indice choisi cette frame, si un l'a ete.
pub fn list(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    items: &[&str],
    selected: usize,
) -> Option<usize> {
    let theme = *ui.theme();
    painter.rect(batch, area, theme.pressed);
    painter.outline(batch, area, theme.border);

    let inner = inset(area, theme.padding);
    let row = theme.row_height;
    let mut chosen = None;

    for (i, item) in items.iter().enumerate() {
        let y = inner.position.y + i as f32 * row;
        // Ce qui deborde n'est pas dessine : un defilement viendra avec la
        // zone de defilement.
        if y + row > inner.position.y + inner.size.y {
            break;
        }

        let line = Rect::new(inner.position.x, y, inner.size.x, row);
        let response = ui.interact(id.index(i), line, true);

        if i == selected {
            painter.rect(batch, line, theme.accent);
        } else if response.hovered {
            painter.rect(batch, line, theme.hovered);
        }

        if response.clicked {
            chosen = Some(i);
        }

        let colour = if i == selected {
            theme.pressed
        } else {
            theme.text
        };
        painter.text(
            batch,
            Vec2::new(
                line.position.x + theme.gap,
                y + (row - painter.measure(item).y) / 2.0,
            ),
            item,
            Align::Left,
            colour,
        );
    }

    chosen
}

/// A row of tabs, one of which is active.
pub fn tabs(
    ui: &mut Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    id: Id,
    area: Rect,
    labels: &[&str],
    active: usize,
) -> Option<usize> {
    if labels.is_empty() {
        return None;
    }

    let theme = *ui.theme();
    let width = area.size.x / labels.len() as f32;
    let mut chosen = None;

    for (i, label) in labels.iter().enumerate() {
        let tab = Rect::new(
            area.position.x + i as f32 * width,
            area.position.y,
            width,
            area.size.y,
        );
        let response = ui.interact(id.index(i), tab, true);

        let fond = if i == active {
            theme.surface
        } else {
            theme.surface_for(response.state())
        };
        painter.rect(batch, tab, fond);
        painter.outline(
            batch,
            tab,
            if response.focused {
                theme.focus
            } else {
                theme.border
            },
        );

        // L'onglet actif se souligne : la couleur seule ne suffit pas a le
        // distinguer pour qui ne les percoit pas.
        if i == active {
            painter.rect(
                batch,
                Rect::new(
                    tab.position.x,
                    tab.position.y + tab.size.y - painter.scale() * 2.0,
                    tab.size.x,
                    painter.scale() * 2.0,
                ),
                theme.accent,
            );
        }

        painter.text(
            batch,
            Vec2::new(
                tab.position.x + width / 2.0,
                tab.position.y + (tab.size.y - painter.measure(label).y) / 2.0,
            ),
            label,
            Align::Centre,
            theme.text,
        );

        if response.clicked {
            chosen = Some(i);
        }
    }

    chosen
}

/// Dims everything behind a modal, and returns the area to draw it in.
pub fn modal(
    ui: &Ui,
    batch: &mut SpriteBatch,
    painter: &Painter,
    screen: Rect,
    size: Vec2,
) -> Rect {
    painter.rect(
        batch,
        screen,
        raster_render::Colour::rgba(0.02, 0.02, 0.04, 0.76),
    );

    let area = crate::layout::centre(screen, size);
    panel(ui, batch, painter, area)
}
