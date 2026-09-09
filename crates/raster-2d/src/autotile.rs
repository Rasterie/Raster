use crate::{TileId, Tilemap};
use raster_math::IVec2;

/// Lesquels des huit voisins correspondent, un bit par direction dans l'ordre
/// de [`IVec2::NEIGHBOURS`] — cet ordre fait partie du format.
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

    /// La variante selectionnee, de 0 a 46.
    ///
    /// 256 voisinages pour 47 apparences : un coin ne compte que si les deux
    /// cotes qui l'encadrent sont remplis.
    #[must_use]
    pub fn variant(self) -> u8 {
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

/// Lit le voisinage d'une tuile : deux tuiles se raccordent si elles sont du
/// meme type.
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

/// Masque reduit -> indice de variante, construit a la compilation.
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

/// Si un masque peut resulter de la reduction : une diagonale isolee, non.
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
