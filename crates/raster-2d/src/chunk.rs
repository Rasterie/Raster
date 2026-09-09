use crate::TileId;
use raster_math::{IRect, IVec2};

/// A fixed block of tiles, the unit a world is stored and streamed in.
///
/// Un monde peut etre trop grand pour tenir d'un bloc, et ce que personne ne
/// regarde ne doit rien couter.
#[derive(Debug, Clone)]
pub struct Chunk {
    tiles: Vec<TileId>,
    /// Si quelque chose a change depuis la derniere reconstruction.
    dirty: bool,
}

impl Chunk {
    /// Tuiles sur un cote d'un chunk : 1024 tuiles, 2 Ko. Plus petit alourdit
    /// la comptabilite, plus grand alourdit chaque reconstruction.
    pub const SIZE: i32 = 32;
    pub const AREA: usize = (Self::SIZE * Self::SIZE) as usize;

    #[must_use]
    pub fn new() -> Self {
        Self {
            tiles: vec![TileId::EMPTY; Self::AREA],
            dirty: true,
        }
    }

    /// Une tuile par sa position locale ; vide hors du chunk, car les lectures
    /// de voisinage debordent tout le temps.
    #[must_use]
    pub fn get(&self, local: IVec2) -> TileId {
        Self::index(local).map_or(TileId::EMPTY, |i| self.tiles[i])
    }

    /// Pose une tuile ; rend `false` si rien n'a change, ce qui evite une
    /// reconstruction inutile quand on peint sur l'existant.
    pub fn set(&mut self, local: IVec2, tile: TileId) -> bool {
        let Some(i) = Self::index(local) else {
            return false;
        };
        if self.tiles[i] == tile {
            return false;
        }
        self.tiles[i] = tile;
        self.dirty = true;
        true
    }

    /// Whether the chunk needs its geometry rebuilt.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the chunk as rebuilt.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Force une reconstruction, apres qu'un voisin a change.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Si toutes les cellules sont vides : un tel chunk peut etre libere.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.iter().all(|t| t.is_empty())
    }

    /// How many cells hold something.
    #[must_use]
    pub fn filled(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_solid()).count()
    }

    /// Chaque tuile avec sa position locale, ligne par ligne.
    pub fn tiles(&self) -> impl Iterator<Item = (IVec2, TileId)> {
        self.tiles.iter().enumerate().map(|(i, tile)| {
            let i = i as i32;
            (IVec2::new(i % Self::SIZE, i / Self::SIZE), *tile)
        })
    }

    /// Replaces every cell.
    pub fn fill(&mut self, tile: TileId) {
        self.tiles.fill(tile);
        self.dirty = true;
    }

    /// The area a chunk at `coord` covers, in world tile coordinates.
    #[must_use]
    pub fn bounds(coord: IVec2) -> IRect {
        IRect::from_position_size(coord * Self::SIZE, IVec2::splat(Self::SIZE))
    }

    /// Le chunk d'une tuile. Division euclidienne : la tuile -1 appartient au
    /// chunk -1, la troncature les replierait sur le chunk 0.
    #[must_use]
    pub fn coord_of(tile: IVec2) -> IVec2 {
        tile.div_euclid(IVec2::splat(Self::SIZE))
    }

    /// Where a world tile sits inside its chunk, always in 0..SIZE.
    #[must_use]
    pub fn local_of(tile: IVec2) -> IVec2 {
        tile.rem_euclid(IVec2::splat(Self::SIZE))
    }

    #[inline]
    fn index(local: IVec2) -> Option<usize> {
        if local.x < 0 || local.y < 0 || local.x >= Self::SIZE || local.y >= Self::SIZE {
            return None;
        }
        Some((local.y * Self::SIZE + local.x) as usize)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
