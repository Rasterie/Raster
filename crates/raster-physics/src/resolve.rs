use crate::{Body, Contacts};
use raster_2d::{Collision, Tilemap};
use raster_math::{Rect, Vec2};

/// Moves a body through a tilemap, stopping it against solid tiles.
///
/// Deplace un axe apres l'autre : resoudre les deux ensemble rend ambigu le
/// cote d'ou vient le contact, et fait accrocher un personnage aux jointures
/// entre deux tuiles alignees.
pub fn move_body(body: &mut Body, map: &Tilemap, dt: f32) -> Contacts {
    let motion = body.velocity * dt;
    let mut contacts = Contacts::default();

    // Un pas plus long qu'une tuile traverserait un mur : on le decoupe.
    let tile = map.tileset().tile_size() as f32;
    let longest = motion.x.abs().max(motion.y.abs());
    let steps = ((longest / (tile * 0.5)).ceil() as u32).max(1);
    let step = motion / steps as f32;

    for _ in 0..steps {
        if move_axis_x(body, map, step.x) {
            contacts.left |= step.x < 0.0;
            contacts.right |= step.x > 0.0;
            body.velocity.x = 0.0;
        }
        if move_axis_y(body, map, step.y, body.drop_through) {
            contacts.above |= step.y < 0.0;
            contacts.below |= step.y > 0.0;
            body.velocity.y = 0.0;
        }
    }

    // Un corps immobile au-dessus du sol doit quand meme se savoir pose.
    if !contacts.below {
        contacts.below = grounded(body, map);
    }

    contacts
}

/// Whether a body rests on something solid.
#[must_use]
pub fn grounded(body: &Body, map: &Tilemap) -> bool {
    let probe = Rect::from_position_size(
        body.bounds.position + Vec2::new(0.0, body.bounds.size.y),
        Vec2::new(body.bounds.size.x, SKIN),
    );
    blocked(map, probe, false, 0.0)
}

/// Distance sous laquelle deux surfaces sont considerees en contact.
///
/// Sans cette marge, un corps pose sur le sol oscille entre « au sol » et « en
/// l'air » selon l'arrondi du flottant.
const SKIN: f32 = 0.01;

/// Deplace en x, rendant `true` si un mur a arrete le mouvement.
fn move_axis_x(body: &mut Body, map: &Tilemap, dx: f32) -> bool {
    if dx == 0.0 {
        return false;
    }

    let moved = body.bounds.translated(Vec2::new(dx, 0.0));
    if !blocked(map, moved, false, 0.0) {
        body.bounds = moved;
        return false;
    }

    // Colle le corps contre la tuile plutot que d'annuler le pas : sinon un
    // personnage s'arreterait a un pixel du mur.
    let tile = map.tileset().tile_size() as f32;
    body.bounds.position.x = if dx > 0.0 {
        let edge = ((body.bounds.right() + dx) / tile).floor() * tile;
        edge - body.bounds.size.x - SKIN
    } else {
        let edge = ((body.bounds.left() + dx) / tile).floor() * tile + tile;
        edge + SKIN
    };
    true
}

/// Deplace en y, rendant `true` si le sol ou un plafond a arrete le mouvement.
fn move_axis_y(body: &mut Body, map: &Tilemap, dy: f32, drop_through: bool) -> bool {
    if dy == 0.0 {
        return false;
    }

    let moved = body.bounds.translated(Vec2::new(0.0, dy));
    let falling = dy > 0.0;
    // Une plateforme ne bloque qu'en descendant, et depuis au-dessus.
    let one_way_blocks = falling && !drop_through;
    let feet = body.bounds.bottom();

    if !blocked(map, moved, one_way_blocks, feet) {
        body.bounds = moved;
        return false;
    }

    let tile = map.tileset().tile_size() as f32;
    body.bounds.position.y = if falling {
        let edge = ((body.bounds.bottom() + dy) / tile).floor() * tile;
        edge - body.bounds.size.y - SKIN
    } else {
        let edge = ((body.bounds.top() + dy) / tile).floor() * tile + tile;
        edge + SKIN
    };
    true
}

/// Whether an area overlaps anything solid.
///
/// `feet` est le bas du corps avant le mouvement : une plateforme ne bloque que
/// si le corps arrivait entierement au-dessus d'elle.
fn blocked(map: &Tilemap, area: Rect, one_way_blocks: bool, feet: f32) -> bool {
    let tile = map.tileset().tile_size() as f32;

    for cell in map.tiles_in(area).cells() {
        match map.collision(cell) {
            Collision::Solid => return true,
            Collision::OneWay if one_way_blocks => {
                let top = cell.y as f32 * tile;
                // Le corps doit venir d'au-dessus, sinon il traverse.
                if feet <= top + SKIN {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether a body overlaps anything solid where it stands.
#[must_use]
pub fn overlapping(body: &Body, map: &Tilemap) -> bool {
    blocked(map, body.bounds, false, 0.0)
}
