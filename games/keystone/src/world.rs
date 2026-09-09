use raster_core::asset::AssetId;
use raster_core::reflect::Reflect;
use raster_math::{Rect, Vec2};

/// La taille d'une tuile, en pixels.
pub const TILE: f32 = 16.0;

/// The player.
#[derive(Reflect, Default, Debug)]
pub struct Player {
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: u32,
    /// Le temps restant d'invulnerabilite, en secondes.
    pub invulnerable: f32,
    /// Vers ou le personnage regarde : -1 a gauche, 1 a droite.
    pub facing: f32,
    pub on_ground: bool,
    pub texture: AssetId,
}

impl Player {
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 12.0, 14.0)
    }
}

/// What an enemy does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    /// Marche, fait demi-tour aux murs et aux bords.
    #[default]
    Walker,
    /// Reste sur place et blesse au contact.
    Spike,
    /// Va et vient entre deux hauteurs.
    Flyer,
}

/// An enemy.
#[derive(Reflect, Default, Debug)]
pub struct Enemy {
    pub position: Vec2,
    pub velocity: Vec2,
    /// Le type, ecrit comme un entier : la reflexion ne porte pas encore les
    /// enums de jeu, et une scene doit rester lisible.
    pub kind: u32,
    pub alive: bool,
    /// L'amplitude d'un `Flyer`, en pixels.
    pub range: f32,
    /// Le point de depart, pour un mouvement qui va et vient.
    pub origin: Vec2,
    pub texture: AssetId,
}

impl Enemy {
    #[must_use]
    pub fn kind(&self) -> Kind {
        match self.kind {
            1 => Kind::Spike,
            2 => Kind::Flyer,
            _ => Kind::Walker,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 14.0, 14.0)
    }
}

/// The key that opens a room's door.
#[derive(Reflect, Default, Debug)]
pub struct Key {
    pub position: Vec2,
    pub taken: bool,
    pub texture: AssetId,
}

impl Key {
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 10.0, 10.0)
    }
}

/// The way out, which needs a key.
#[derive(Reflect, Default, Debug)]
pub struct Door {
    pub position: Vec2,
    pub texture: AssetId,
}

impl Door {
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 16.0, 24.0)
    }
}

/// One screen of the game.
#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    /// Les tuiles, ligne par ligne : `#` solide, `=` plateforme, `.` vide.
    pub tiles: Vec<String>,
    pub spawn: Vec2,
}

impl Room {
    #[must_use]
    pub fn width(&self) -> usize {
        self.tiles.first().map_or(0, |row| row.chars().count())
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    /// The character at a tile, `'.'` outside the room.
    ///
    /// Hors salle est du vide, jamais une erreur : un ennemi qui marche
    /// jusqu'au bord interroge des cases qui n'existent pas.
    #[must_use]
    pub fn at(&self, x: i32, y: i32) -> char {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return '.';
        };
        self.tiles
            .get(y)
            .and_then(|row| row.chars().nth(x))
            .unwrap_or('.')
    }

    /// Whether a tile blocks movement from every side.
    #[must_use]
    pub fn solid(&self, x: i32, y: i32) -> bool {
        self.at(x, y) == '#'
    }

    /// Whether a tile blocks only from above.
    #[must_use]
    pub fn platform(&self, x: i32, y: i32) -> bool {
        self.at(x, y) == '='
    }

    /// Whether something standing at `at` has ground under it.
    #[must_use]
    pub fn ground_under(&self, at: Vec2) -> bool {
        let x = (at.x / TILE).floor() as i32;
        let y = (at.y / TILE).floor() as i32;
        self.solid(x, y) || self.platform(x, y)
    }
}
