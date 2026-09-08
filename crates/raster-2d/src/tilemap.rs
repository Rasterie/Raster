use crate::{Chunk, Collision, TileId, Tileset};
use raster_math::{IRect, IVec2, Rect, Vec2};
use std::collections::HashMap;

/// The tile containing the last point strictly inside a boundary at `edge`.
///
/// A rectangle ending exactly on a tile boundary stops at the tile before it.
fn exclusive_index(edge: f32) -> i32 {
    let floor = edge.floor();
    if (edge - floor).abs() < f32::EPSILON {
        floor as i32 - 1
    } else {
        floor as i32
    }
}

/// A world of tiles, unbounded in every direction.
///
/// Chunks exist only where something was placed, so an empty world costs
/// nothing and a player can walk as far as they like. This is what a
/// Terraria-like needs and what a fixed-size grid cannot give.
#[derive(Debug)]
pub struct Tilemap {
    chunks: HashMap<IVec2, Chunk>,
    tileset: Tileset,
}

impl Tilemap {
    #[must_use]
    pub fn new(tileset: Tileset) -> Self {
        Self {
            chunks: HashMap::new(),
            tileset,
        }
    }

    #[must_use]
    pub fn tileset(&self) -> &Tileset {
        &self.tileset
    }

    pub fn tileset_mut(&mut self) -> &mut Tileset {
        &mut self.tileset
    }

    /// The tile at a world coordinate.
    ///
    /// Empty where no chunk exists — an unbuilt part of the world reads as a
    /// hole, which is exactly what it is.
    #[must_use]
    pub fn get(&self, tile: IVec2) -> TileId {
        self.chunks
            .get(&Chunk::coord_of(tile))
            .map_or(TileId::EMPTY, |c| c.get(Chunk::local_of(tile)))
    }

    /// Places a tile, creating its chunk if needed.
    ///
    /// Returns whether anything changed.
    pub fn set(&mut self, tile: IVec2, id: TileId) -> bool {
        let coord = Chunk::coord_of(tile);

        // Ne pas creer un chunk pour y ecrire du vide : un joueur qui casse
        // une tuile inexistante ne doit pas faire grandir la carte.
        if id.is_empty() && !self.chunks.contains_key(&coord) {
            return false;
        }

        let changed = self
            .chunks
            .entry(coord)
            .or_default()
            .set(Chunk::local_of(tile), id);

        if changed {
            self.mark_neighbours(tile);
        }
        changed
    }

    /// Removes a tile.
    pub fn clear(&mut self, tile: IVec2) -> bool {
        self.set(tile, TileId::EMPTY)
    }

    /// How a tile behaves for collision.
    #[must_use]
    pub fn collision(&self, tile: IVec2) -> Collision {
        self.tileset.collision(self.get(tile))
    }

    /// Whether a tile blocks movement from every direction.
    #[must_use]
    pub fn is_solid(&self, tile: IVec2) -> bool {
        self.collision(tile) == Collision::Solid
    }

    /// The world-space rectangle a tile covers.
    #[must_use]
    pub fn tile_bounds(&self, tile: IVec2) -> Rect {
        let size = self.tileset.tile_size() as f32;
        Rect::from_position_size(tile.as_vec2() * size, Vec2::splat(size))
    }

    /// Which tile contains a world-space point.
    ///
    /// Floors rather than truncating, so a point at -0.5 lands in tile -1.
    #[must_use]
    pub fn tile_at(&self, position: Vec2) -> IVec2 {
        IVec2::from_vec2_floor(position / self.tileset.tile_size() as f32)
    }

    /// Every tile coordinate a world-space rectangle touches.
    ///
    /// This is what collision uses: rather than testing every tile in the
    /// world, a body only looks at the handful it overlaps.
    #[must_use]
    pub fn tiles_in(&self, area: Rect) -> IRect {
        let size = self.tileset.tile_size() as f32;
        let min = self.tile_at(area.min());

        /*
          Le coin superieur est exclusif : un rectangle qui s'arrete pile sur
          une frontiere ne doit pas inclure la tuile d'apres, sinon un corps
          colle a un mur declencherait une collision avec celle d'en face.

          On ne peut pas simplement soustraire f32::EPSILON : cette constante
          est l'ecart entre deux flottants autour de 1,0, et vers 16,0 l'ecart
          representable est seize fois plus grand — la soustraction serait
          absorbee. On travaille donc sur l'indice, en reculant d'une tuile
          quand le bord tombe exactement sur une frontiere.
        */
        let far = area.max() / size;
        let max = IVec2::new(exclusive_index(far.x), exclusive_index(far.y));

        IRect::from_corners(min, max + IVec2::ONE)
    }

    /// Fills a rectangle of tiles.
    pub fn fill(&mut self, area: IRect, id: TileId) {
        for tile in area.cells() {
            self.set(tile, id);
        }
    }

    /// Every chunk that exists, with its coordinate.
    pub fn chunks(&self) -> impl Iterator<Item = (IVec2, &Chunk)> {
        self.chunks.iter().map(|(coord, chunk)| (*coord, chunk))
    }

    /// Chunks overlapping a world-space area, for drawing only what is visible.
    pub fn chunks_in(&self, area: Rect) -> impl Iterator<Item = (IVec2, &Chunk)> {
        let tiles = self.tiles_in(area);
        let min = Chunk::coord_of(tiles.min());
        let max = Chunk::coord_of(tiles.max() - IVec2::ONE);

        (min.y..=max.y)
            .flat_map(move |y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
            .filter_map(|coord| self.chunks.get(&coord).map(|chunk| (coord, chunk)))
    }

    /// Chunks needing their geometry rebuilt.
    pub fn dirty_chunks(&self) -> impl Iterator<Item = (IVec2, &Chunk)> {
        self.chunks().filter(|(_, chunk)| chunk.is_dirty())
    }

    /// Marks every chunk as rebuilt.
    pub fn clear_dirty(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.clear_dirty();
        }
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// How many cells hold a tile.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.chunks.values().map(Chunk::filled).sum()
    }

    /// Drops chunks that hold nothing.
    ///
    /// A world where things get destroyed accumulates empty chunks that cost
    /// memory and rebuild time for no visible reason.
    pub fn prune(&mut self) -> usize {
        let before = self.chunks.len();
        self.chunks.retain(|_, chunk| !chunk.is_empty());
        before - self.chunks.len()
    }

    /// Marks the chunks around a tile as dirty when it sits on a boundary.
    ///
    /// Autotiling looks at neighbours, so a tile on a chunk edge changes how
    /// the tile across the border is drawn. Without this, seams appear exactly
    /// where two chunks meet — a bug that only shows up on chunk boundaries
    /// and is therefore easy to miss.
    fn mark_neighbours(&mut self, tile: IVec2) {
        let local = Chunk::local_of(tile);
        let coord = Chunk::coord_of(tile);
        let last = Chunk::SIZE - 1;

        for offset in IVec2::NEIGHBOURS {
            let touches_edge = (offset.x < 0 && local.x == 0)
                || (offset.x > 0 && local.x == last)
                || (offset.y < 0 && local.y == 0)
                || (offset.y > 0 && local.y == last);

            if touches_edge && let Some(neighbour) = self.chunks.get_mut(&(coord + offset)) {
                neighbour.mark_dirty();
            }
        }
    }
}
