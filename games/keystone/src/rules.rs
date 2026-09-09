use raster_math::{Rect, Vec2};

/// Ce qui tue, ce qui blesse, ce qui fait avancer.
pub const MAX_HEALTH: u32 = 3;

/// Le temps d'invulnerabilite apres un coup : sans lui, traverser un ennemi
/// coute les trois coeurs d'un coup.
pub const INVULNERABLE: f32 = 1.0;

/// La vitesse verticale au-dela de laquelle un saut sur un ennemi l'ecrase.
///
/// Positif vers le bas : un joueur qui monte ne peut pas ecraser.
pub const STOMP_SPEED: f32 = 40.0;

/// La poussee vers le haut apres avoir ecrase un ennemi.
pub const STOMP_BOUNCE: f32 = 200.0;

/// The player's state that survives a room change.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    pub health: u32,
    pub keys: u32,
    pub room: usize,
    /// Les salles franchies, dans l'ordre : ce qui se sauvegarde.
    pub cleared: Vec<usize>,
}

impl Progress {
    #[must_use]
    pub fn new() -> Self {
        Self {
            health: MAX_HEALTH,
            keys: 0,
            room: 0,
            cleared: Vec::new(),
        }
    }

    #[must_use]
    pub fn alive(&self) -> bool {
        self.health > 0
    }

    /// Marks the current room cleared, without recording it twice.
    pub fn clear(&mut self, room: usize) {
        if !self.cleared.contains(&room) {
            self.cleared.push(room);
        }
    }

    #[must_use]
    pub fn has_cleared(&self, room: usize) -> bool {
        self.cleared.contains(&room)
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// What a hit does to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    /// Le joueur est encore invulnerable : rien ne se passe.
    Ignored,
    Hurt,
    Killed,
}

/// Applies damage, respecting invulnerability.
pub fn take_damage(health: &mut u32, invulnerable: &mut f32, amount: u32) -> Hit {
    if *invulnerable > 0.0 || *health == 0 {
        return Hit::Ignored;
    }

    *health = health.saturating_sub(amount);
    *invulnerable = INVULNERABLE;

    if *health == 0 { Hit::Killed } else { Hit::Hurt }
}

/// Whether the player lands on an enemy hard enough to squash it.
///
/// Il faut tomber dessus : toucher un ennemi de cote blesse, sauter dessus tue.
#[must_use]
pub fn stomps(player: Rect, velocity: Vec2, enemy: Rect) -> bool {
    if velocity.y < STOMP_SPEED {
        return false;
    }
    if !player.intersects(enemy) {
        return false;
    }

    // Les pieds doivent etre dans la moitie haute de l'ennemi, sinon un joueur
    // qui tombe le long d'un mur ecraserait ce qui passe a cote.
    let feet = player.position.y + player.size.y;
    feet <= enemy.position.y + enemy.size.y * 0.5
}

/// How an enemy that walks turns around.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Keep,
    Reverse,
}

/// Whether a walker should turn: at a wall, or at the edge of its ground.
///
/// `ahead_solid` : un mur devant. `ground_ahead` : du sol sous le pas suivant.
#[must_use]
pub fn walker_turn(ahead_solid: bool, ground_ahead: bool) -> Turn {
    if ahead_solid || !ground_ahead {
        Turn::Reverse
    } else {
        Turn::Keep
    }
}
