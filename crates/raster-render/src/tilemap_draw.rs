use crate::{Camera, Layer, SpriteBatch, SpriteDraw};
use raster_2d::{Neighbours, Tilemap, neighbours_of};
use raster_math::{IVec2, Rect, Vec2};

/// Draws the visible part of a tilemap. Reutilise le batcher : ~286 tuiles
/// visibles, 13 Ko par frame, qu'un cache n'ameliorerait pas.
pub struct TilemapRenderer {
    /// Which texture in the game's list holds the tileset.
    texture: usize,
    layer: Layer,
    /// How many tiles across the tileset texture is, for turning an atlas
    /// coordinate into pixels.
    columns: u32,
    stats: TileStats,
}

/// What the last draw cost.
#[derive(Debug, Clone, Copy, Default)]
pub struct TileStats {
    pub tiles: u32,
    pub chunks: u32,
}

impl TilemapRenderer {
    /// A renderer for a tileset packed `columns` tiles across.
    ///
    /// # Panics
    ///
    /// If `columns` is zero.
    #[must_use]
    pub fn new(texture: usize, columns: u32) -> Self {
        assert!(columns > 0, "a tileset needs at least one column");

        Self {
            texture,
            layer: Layer::BACKGROUND,
            columns,
            stats: TileStats::default(),
        }
    }

    #[must_use]
    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Met en file les tuiles visibles, en parcourant les chunks.
    pub fn draw(&mut self, batch: &mut SpriteBatch, map: &Tilemap, camera: Camera) {
        self.stats = TileStats::default();

        let visible = camera.visible_area();
        let tile_size = map.tileset().tile_size() as f32;

        for (coord, chunk) in map.chunks_in(visible) {
            self.stats.chunks += 1;
            let origin = coord * raster_2d::Chunk::SIZE;

            for (local, id) in chunk.tiles() {
                if id.is_empty() {
                    continue;
                }

                let world = origin + local;
                let position = world.as_vec2() * tile_size;

                // Un chunk visible deborde toujours des bords de la vue.
                if !visible.intersects(Rect::from_position_size(position, Vec2::splat(tile_size))) {
                    continue;
                }

                batch.draw(
                    self.texture,
                    SpriteDraw {
                        position,
                        size: Vec2::splat(tile_size),
                        source: self.source_rect(map, world, id, tile_size),
                        layer: self.layer,
                        ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(tile_size))
                    },
                );
                self.stats.tiles += 1;
            }
        }
    }

    #[must_use]
    pub fn stats(&self) -> TileStats {
        self.stats
    }

    /// Ou se trouve l'image d'une tuile dans le tileset.
    fn source_rect(
        &self,
        map: &Tilemap,
        world: IVec2,
        id: raster_2d::TileId,
        tile_size: f32,
    ) -> Rect {
        let kind = map.tileset().kind(id);

        let atlas = if kind.autotile {
            let variant = i32::from(neighbours_of(map, world).variant());
            // Les 47 variantes se suivent, en enroulant sur la largeur.
            let index = kind.atlas.y * i32::try_from(self.columns).unwrap_or(i32::MAX)
                + kind.atlas.x
                + variant;
            let columns = i32::try_from(self.columns).unwrap_or(1).max(1);
            IVec2::new(index % columns, index / columns)
        } else {
            kind.atlas
        };

        Rect::from_position_size(atlas.as_vec2() * tile_size, Vec2::splat(tile_size))
    }
}

/// Which of a tile's neighbours are the same kind, exposed for a tool that
/// needs to show it.
#[must_use]
pub fn tile_neighbours(map: &Tilemap, tile: IVec2) -> Neighbours {
    neighbours_of(map, tile)
}

/// La variante qu'un masque de voisinage selectionne, parmi 47.
#[must_use]
pub fn tile_variant(mask: u8) -> u8 {
    Neighbours(mask).variant()
}
