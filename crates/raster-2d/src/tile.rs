use raster_math::IVec2;

/// Which tile occupies a cell. Deux octets : ce qu'une tuile signifie vit dans
/// le tileset, pas dans chaque cellule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TileId(pub u16);

impl TileId {
    /// L'absence de tuile. Zero, pour qu'un chunk neuf soit vide sans travail.
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
    /// Ne bloque que par le haut : on traverse en sautant, on atterrit dessus.
    OneWay,
}

/// Ce qu'est une tuile, une fois par type plutot que par cellule.
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

/// Every kind of tile a world can contain, indexe par [`TileId`].
#[derive(Debug, Clone, Default)]
pub struct Tileset {
    kinds: Vec<TileKind>,
    /// La taille d'une tuile en pixels, carree.
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
            // L'emplacement zero est toujours le vide.
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

    /// Ce qu'est une tuile ; un identifiant inconnu se comporte comme un trou.
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
        // Le vide occupant toujours l'emplacement zero.
        self.kinds.len() <= 1
    }

    /// Cherche un type par nom. Lineaire : pour le chargement, pas la frame.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<TileId> {
        self.kinds
            .iter()
            .position(|k| k.name == name)
            .and_then(|i| u16::try_from(i).ok())
            .map(TileId)
    }
}
