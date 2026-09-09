use raster_math::IVec2;

/// Which tile occupies a cell.
///
/// Two bytes rather than an enum with data: a large world holds millions of
/// these, and every byte is multiplied by that. What a tile *means* — its
/// collision, its texture, whether it can be mined — lives once in the tileset,
/// not once per cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TileId(pub u16);

impl TileId {
    /// The absence of a tile. Zero so that a freshly allocated chunk is empty
    /// without having to fill it.
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_solid(self) -> bool {
        !self.is_empty()
    }
}

/// How a tile behaves when something walks into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Collision {
    /// Nothing stops here.
    #[default]
    None,
    /// Blocks from every direction.
    Solid,
    /// Blocks from above only, so a character can jump up through it and land
    /// on top. Every platformer needs this and it cannot be expressed as a
    /// plain solid.
    OneWay,
}

/// What a tile is, held once per kind rather than once per cell.
#[derive(Debug, Clone)]
pub struct TileKind {
    pub name: String,
    /// Where the tile sits in the tileset texture, in tiles rather than pixels.
    pub atlas: IVec2,
    pub collision: Collision,
    /// Whether neighbouring tiles of the same kind should join up visually.
    pub autotile: bool,
}

impl Default for TileKind {
    fn default() -> Self {
        Self {
            name: String::new(),
            atlas: IVec2::ZERO,
            collision: Collision::None,
            autotile: false,
        }
    }
}

/// Every kind of tile a world can contain.
///
/// Indexed by [`TileId`], so a lookup is an array access rather than a hash.
#[derive(Debug, Clone, Default)]
pub struct Tileset {
    kinds: Vec<TileKind>,
    /// The size of one tile in pixels. Square, because a non-square tile grid
    /// complicates every calculation downstream for no benefit anyone has asked
    /// for.
    tile_size: u32,
}

impl Tileset {
    /// A tileset whose tiles are `tile_size` pixels square.
    ///
    /// # Panics
    ///
    /// If `tile_size` is zero.
    #[must_use]
    pub fn new(tile_size: u32) -> Self {
        assert!(tile_size > 0, "a tile cannot be zero pixels");

        Self {
            // L'emplacement zero est toujours le vide : cela evite un decalage
            // d'indice partout ailleurs.
            kinds: vec![TileKind {
                name: "empty".to_owned(),
                ..TileKind::default()
            }],
            tile_size,
        }
    }

    /// Adds a kind and returns the id that refers to it.
    ///
    /// # Panics
    ///
    /// If the tileset already holds 65 535 kinds.
    pub fn add(&mut self, kind: TileKind) -> TileId {
        let id = u16::try_from(self.kinds.len()).expect("more than 65 535 tile kinds");
        self.kinds.push(kind);
        TileId(id)
    }

    /// What a tile is, or the empty kind for an id this tileset does not know.
    ///
    /// Returning the empty kind rather than `None` keeps callers simple: an
    /// unknown tile behaves as a hole, which is both harmless and visible.
    #[must_use]
    pub fn kind(&self, id: TileId) -> &TileKind {
        self.kinds.get(id.0 as usize).unwrap_or(&self.kinds[0])
    }

    #[must_use]
    pub fn collision(&self, id: TileId) -> Collision {
        self.kind(id).collision
    }

    #[must_use]
    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        // L'emplacement zero est toujours present, donc un tileset n'est
        // « vide » que s'il ne contient que celui-la.
        self.kinds.len() <= 1
    }

    /// Finds a kind by name. Linear, and meant for loading rather than for the
    /// frame loop.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<TileId> {
        self.kinds
            .iter()
            .position(|k| k.name == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(TileId)
    }
}
