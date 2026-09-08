use crate::TileId;
use raster_math::{IRect, IVec2};

/// A fixed block of tiles, the unit a world is stored and streamed in.
///
/// Chunking rather than one huge array: a Terraria-like world is far too large
/// to hold at once, and the parts nobody is looking at should cost nothing.
#[derive(Debug, Clone)]
pub struct Chunk {
    tiles: Vec<TileId>,
    /// Whether anything changed since the mesh was last built.
    ///
    /// Rebuilding every chunk every frame would be wasteful; rebuilding none
    /// would show stale tiles. This flag is what makes editing a tile cheap.
    dirty: bool,
}

impl Chunk {
    /// Tiles along one edge of a chunk.
    ///
    /// 32 is a compromise measured in two directions: smaller chunks mean more
    /// bookkeeping per tile, larger ones mean rebuilding more geometry when a
    /// single tile changes. At 32×32 a chunk is 1024 tiles and 2 KB.
    pub const SIZE: i32 = 32;
    pub const AREA: usize = (Self::SIZE * Self::SIZE) as usize;

    #[must_use]
    pub fn new() -> Self {
        Self {
            tiles: vec![TileId::EMPTY; Self::AREA],
            // Neuf mais marque a reconstruire : un chunk vide n'a pas encore
            // de maillage.
            dirty: true,
        }
    }

    /// A tile by its position within the chunk.
    ///
    /// Returns [`TileId::EMPTY`] outside the chunk rather than panicking:
    /// neighbour lookups routinely run off the edge, and every caller checking
    /// bounds first would be noise.
    #[must_use]
    pub fn get(&self, local: IVec2) -> TileId {
        Self::index(local).map_or(TileId::EMPTY, |i| self.tiles[i])
    }

    /// Sets a tile, returning whether anything changed.
    ///
    /// The answer matters: writing the same tile back should not mark the chunk
    /// for a rebuild, and painting often writes over what is already there.
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

    /// Forces a rebuild — after a neighbour changed, which can alter this
    /// chunk's autotiling along the shared edge.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Whether every cell is empty.
    ///
    /// Worth knowing: an empty chunk needs no geometry and can be dropped from
    /// memory entirely.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.iter().all(|t| t.is_empty())
    }

    /// How many cells hold something.
    #[must_use]
    pub fn filled(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_solid()).count()
    }

    /// Every tile with its local position, row by row.
    ///
    /// Row-major so that reads walk memory forwards, which is what the mesh
    /// builder wants.
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

    /// Which chunk a world tile belongs to.
    ///
    /// Euclidean division, so tile -1 belongs to chunk -1 rather than chunk 0.
    /// Plain division truncates towards zero and would fold the two cells
    /// either side of the origin into one chunk.
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
