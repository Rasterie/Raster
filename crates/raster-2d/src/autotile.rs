use crate::{TileId, Tilemap};
use raster_math::IVec2;

/// Which of a tile's eight neighbours match it.
///
/// One bit per direction, in the order of [`IVec2::NEIGHBOURS`] — clockwise
/// from up. The bit order is part of the format, since a tileset's variants are
/// indexed by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Neighbours(pub u8);

impl Neighbours {
    pub const UP: u8 = 1 << 0;
    pub const UP_RIGHT: u8 = 1 << 1;
    pub const RIGHT: u8 = 1 << 2;
    pub const DOWN_RIGHT: u8 = 1 << 3;
    pub const DOWN: u8 = 1 << 4;
    pub const DOWN_LEFT: u8 = 1 << 5;
    pub const LEFT: u8 = 1 << 6;
    pub const UP_LEFT: u8 = 1 << 7;

    /// The four edge-sharing directions.
    pub const CARDINAL: u8 = Self::UP | Self::RIGHT | Self::DOWN | Self::LEFT;

    #[must_use]
    pub const fn has(self, direction: u8) -> bool {
        self.0 & direction != 0
    }

    /// The variant this neighbourhood selects, from 0 to 46.
    ///
    /// A tile has 256 possible neighbourhoods but only 47 distinct appearances:
    /// a corner neighbour only matters when both edges beside it are also
    /// filled. Without that reduction a tileset would need 256 drawings instead
    /// of 47, which is the difference between a feasible art task and an
    /// unreasonable one.
    #[must_use]
    pub fn variant(self) -> u8 {
        // Un voisin diagonal ne compte que si les deux cotes qui l'encadrent
        // sont remplis : sinon le coin est visible et la diagonale ne change
        // rien au dessin.
        let mut mask = self.0 & Self::CARDINAL;

        if self.has(Self::UP) && self.has(Self::RIGHT) && self.has(Self::UP_RIGHT) {
            mask |= Self::UP_RIGHT;
        }
        if self.has(Self::RIGHT) && self.has(Self::DOWN) && self.has(Self::DOWN_RIGHT) {
            mask |= Self::DOWN_RIGHT;
        }
        if self.has(Self::DOWN) && self.has(Self::LEFT) && self.has(Self::DOWN_LEFT) {
            mask |= Self::DOWN_LEFT;
        }
        if self.has(Self::LEFT) && self.has(Self::UP) && self.has(Self::UP_LEFT) {
            mask |= Self::UP_LEFT;
        }

        VARIANT_OF[mask as usize]
    }
}

/// Reads a tile's neighbourhood from the map.
///
/// Two tiles are neighbours for this purpose when they are the same kind — a
/// stone block joins other stone, not dirt.
#[must_use]
pub fn neighbours_of(map: &Tilemap, tile: IVec2) -> Neighbours {
    let id = map.get(tile);
    if id.is_empty() {
        return Neighbours(0);
    }

    let mut bits = 0u8;
    for (i, offset) in IVec2::NEIGHBOURS.iter().enumerate() {
        if map.get(tile + *offset) == id {
            bits |= 1 << i;
        }
    }
    Neighbours(bits)
}

/// Whether a tile should be autotiled at all.
#[must_use]
pub fn should_autotile(map: &Tilemap, id: TileId) -> bool {
    map.tileset().kind(id).autotile
}

/// Maps a reduced neighbourhood mask to a variant index.
///
/// Built once at compile time: 256 entries, of which 47 are distinct. Anything
/// unreachable maps to 0, which draws the isolated tile — visibly wrong rather
/// than silently plausible.
static VARIANT_OF: [u8; 256] = build_variant_table();

const fn build_variant_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut next = 0u8;
    let mut mask = 0usize;

    while mask < 256 {
        if is_reachable(mask as u8) {
            table[mask] = next;
            next += 1;
        }
        mask += 1;
    }
    table
}

/// Whether a mask can arise from the reduction in [`Neighbours::variant`].
///
/// A diagonal bit is only ever set when both adjacent edges are set, so a mask
/// with a lone diagonal never occurs and needs no variant.
const fn is_reachable(mask: u8) -> bool {
    let up = mask & Neighbours::UP != 0;
    let right = mask & Neighbours::RIGHT != 0;
    let down = mask & Neighbours::DOWN != 0;
    let left = mask & Neighbours::LEFT != 0;

    if mask & Neighbours::UP_RIGHT != 0 && !(up && right) {
        return false;
    }
    if mask & Neighbours::DOWN_RIGHT != 0 && !(right && down) {
        return false;
    }
    if mask & Neighbours::DOWN_LEFT != 0 && !(down && left) {
        return false;
    }
    if mask & Neighbours::UP_LEFT != 0 && !(left && up) {
        return false;
    }
    true
}
