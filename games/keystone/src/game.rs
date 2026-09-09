use crate::rooms;
use crate::rules::{self, Hit, Progress};
use crate::world::{Kind, Room, TILE};
use raster_math::{Rect, Vec2};

/// Ce que l'ecran montre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Playing,
    Paused,
    Dead,
    Won,
}

/// Un ennemi en jeu.
#[derive(Debug, Clone)]
pub struct Foe {
    pub position: Vec2,
    pub velocity: Vec2,
    pub kind: Kind,
    pub alive: bool,
    pub origin: Vec2,
    pub range: f32,
}

impl Foe {
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, 14.0, 14.0)
    }
}

/// Ce que la frame vient de produire, pour que l'appelant joue les sons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Events {
    pub jumped: bool,
    pub hurt: bool,
    pub stomped: bool,
    pub took_key: bool,
    pub cleared_room: bool,
    pub died: bool,
    pub won: bool,
}

/// The game: rooms, the player, and what is happening.
pub struct Game {
    pub screen: Screen,
    pub progress: Progress,
    pub rooms: Vec<Room>,
    pub player: Vec2,
    pub velocity: Vec2,
    pub facing: f32,
    pub on_ground: bool,
    pub invulnerable: f32,
    pub foes: Vec<Foe>,
    pub key: Option<Vec2>,
    pub door: Vec2,
    /// Le temps ecoule, pour les animations et le clignotement.
    pub time: f32,
}

/// La physique, en pixels par seconde.
pub const SPEED: f32 = 90.0;
pub const GRAVITY: f32 = 700.0;
pub const JUMP: f32 = 245.0;
pub const MAX_FALL: f32 = 500.0;
const FOE_SPEED: f32 = 34.0;
const FLYER_SPEED: f32 = 45.0;

impl Game {
    #[must_use]
    pub fn new() -> Self {
        let rooms = rooms::all();
        let mut game = Self {
            screen: Screen::Title,
            progress: Progress::new(),
            rooms,
            player: Vec2::ZERO,
            velocity: Vec2::ZERO,
            facing: 1.0,
            on_ground: false,
            invulnerable: 0.0,
            foes: Vec::new(),
            key: None,
            door: Vec2::ZERO,
            time: 0.0,
        };
        game.enter_room(0);
        game
    }

    #[must_use]
    pub fn room(&self) -> &Room {
        &self.rooms[self.progress.room.min(self.rooms.len() - 1)]
    }

    /// Places the player, the enemies, the key and the door for a room.
    pub fn enter_room(&mut self, index: usize) {
        self.progress.room = index.min(self.rooms.len() - 1);
        let room = self.rooms[self.progress.room].clone();

        self.player = room.spawn;
        self.velocity = Vec2::ZERO;
        self.on_ground = false;
        self.progress.keys = 0;

        // Les habitants de chaque salle : poses ici plutot que dans les tuiles,
        // car ce sont des acteurs et non du decor.
        let (foes, key, door) = populate(self.progress.room, &room);
        self.foes = foes;
        self.key = Some(key);
        self.door = door;
    }

    /// Starts a new game from the first room.
    pub fn start(&mut self) {
        self.progress = Progress::new();
        self.enter_room(0);
        self.screen = Screen::Playing;
    }

    /// Resumes from saved progress.
    pub fn resume(&mut self, progress: Progress) {
        let room = progress.room;
        self.progress = progress;
        self.enter_room(room);
        self.screen = Screen::Playing;
    }

    /// Advances one fixed step.
    ///
    /// `left`, `right` et `jump` sont l'etat des commandes ; le rendu et le son
    /// se lisent dans ce que la methode renvoie.
    pub fn step(&mut self, left: bool, right: bool, jump: bool, dt: f32) -> Events {
        let mut events = Events::default();
        if self.screen != Screen::Playing {
            return events;
        }

        self.time += dt;
        self.invulnerable = (self.invulnerable - dt).max(0.0);

        // Le joueur
        let dir = f32::from(right) - f32::from(left);
        self.velocity.x = dir * SPEED;
        if dir != 0.0 {
            self.facing = dir.signum();
        }

        if jump && self.on_ground {
            self.velocity.y = -JUMP;
            self.on_ground = false;
            events.jumped = true;
        }

        self.velocity.y = (self.velocity.y + GRAVITY * dt).min(MAX_FALL);

        let room = self.rooms[self.progress.room].clone();
        let (position, grounded) = move_against(&room, self.player, self.velocity, dt, 12.0, 14.0);
        self.player = position;
        self.on_ground = grounded;
        if grounded && self.velocity.y > 0.0 {
            self.velocity.y = 0.0;
        }

        // Tomber hors de la salle tue, sans quoi le joueur chuterait sans fin.
        if self.player.y > room.height() as f32 * TILE + TILE {
            self.progress.health = 0;
            events.died = true;
            self.screen = Screen::Dead;
            return events;
        }

        self.step_foes(&room, dt);
        self.resolve_contacts(&mut events);

        if self.progress.health == 0 {
            events.died = true;
            self.screen = Screen::Dead;
        }

        events
    }

    fn step_foes(&mut self, room: &Room, dt: f32) {
        for foe in &mut self.foes {
            if !foe.alive {
                continue;
            }

            match foe.kind {
                Kind::Spike => {}
                Kind::Walker => {
                    let dir = if foe.velocity.x >= 0.0 { 1.0 } else { -1.0 };
                    foe.velocity.x = dir * FOE_SPEED;

                    let next = foe.position.x + foe.velocity.x * dt;
                    // Le point que le prochain pas atteindrait, pieds compris.
                    let ahead = Vec2::new(
                        if dir > 0.0 { next + 14.0 } else { next },
                        foe.position.y + 7.0,
                    );
                    let under = Vec2::new(ahead.x, foe.position.y + 15.0);

                    let solid = room.solid(
                        (ahead.x / TILE).floor() as i32,
                        (ahead.y / TILE).floor() as i32,
                    );
                    let ground = room.ground_under(under);

                    if rules::walker_turn(solid, ground) == rules::Turn::Reverse {
                        foe.velocity.x = -foe.velocity.x;
                    } else {
                        foe.position.x = next;
                    }
                }
                Kind::Flyer => {
                    // Va et vient autour de son point de depart.
                    let dir = if foe.velocity.x >= 0.0 { 1.0 } else { -1.0 };
                    foe.velocity.x = dir * FLYER_SPEED;
                    foe.position.x += foe.velocity.x * dt;

                    if (foe.position.x - foe.origin.x).abs() > foe.range {
                        foe.velocity.x = -foe.velocity.x;
                        // Ramene dans les bornes : sans cela un pas trop grand
                        // laisserait l'ennemi coince dehors, a osciller.
                        foe.position.x = foe
                            .position
                            .x
                            .clamp(foe.origin.x - foe.range, foe.origin.x + foe.range);
                    }
                }
            }
        }
    }

    fn resolve_contacts(&mut self, events: &mut Events) {
        let player = Rect::new(self.player.x, self.player.y, 12.0, 14.0);

        for foe in &mut self.foes {
            if !foe.alive || !player.intersects(foe.bounds()) {
                continue;
            }

            // Un pic ne s'ecrase pas : sauter dessus blesse comme le reste.
            if foe.kind != Kind::Spike && rules::stomps(player, self.velocity, foe.bounds()) {
                foe.alive = false;
                self.velocity.y = -rules::STOMP_BOUNCE;
                events.stomped = true;
                continue;
            }

            match rules::take_damage(&mut self.progress.health, &mut self.invulnerable, 1) {
                Hit::Hurt => {
                    events.hurt = true;
                    // Repousse a l'oppose de l'ennemi, pour degager du contact.
                    let away = (player.center().x - foe.bounds().center().x).signum();
                    self.velocity = Vec2::new(away * 120.0, -140.0);
                }
                Hit::Killed => events.hurt = true,
                Hit::Ignored => {}
            }
        }

        if let Some(key) = self.key
            && player.intersects(Rect::new(key.x, key.y, 10.0, 10.0))
        {
            self.key = None;
            self.progress.keys += 1;
            events.took_key = true;
        }

        if self.progress.keys > 0
            && player.intersects(Rect::new(self.door.x, self.door.y, 16.0, 24.0))
        {
            self.progress.clear(self.progress.room);
            events.cleared_room = true;

            if self.progress.room + 1 >= self.rooms.len() {
                self.screen = Screen::Won;
                events.won = true;
            } else {
                let next = self.progress.room + 1;
                self.enter_room(next);
            }
        }
    }

    /// Respawns in the current room, keeping cleared rooms.
    pub fn respawn(&mut self) {
        self.progress.health = rules::MAX_HEALTH;
        self.invulnerable = 0.0;
        let room = self.progress.room;
        self.enter_room(room);
        self.screen = Screen::Playing;
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// Moves a box against the room's tiles, axis by axis.
///
/// Axe par axe : deplacer les deux d'un coup ferait accrocher un coin sur un
/// sol plat.
fn move_against(
    room: &Room,
    position: Vec2,
    velocity: Vec2,
    dt: f32,
    width: f32,
    height: f32,
) -> (Vec2, bool) {
    let mut at = position;

    at.x += velocity.x * dt;
    if let Some(fixed) = push_out_x(room, at, width, height, velocity.x) {
        at.x = fixed;
    }

    at.y += velocity.y * dt;
    let mut grounded = false;
    if let Some(fixed) = push_out_y(room, at, width, height, velocity.y) {
        at.y = fixed;
        grounded = velocity.y > 0.0;
    }

    (at, grounded)
}

/// Les cases qu'une boite recouvre sur un axe.
fn span(low: f32, high: f32) -> std::ops::RangeInclusive<i32> {
    (low / TILE).floor() as i32..=((high - 0.01) / TILE).floor() as i32
}

fn push_out_x(room: &Room, at: Vec2, width: f32, height: f32, vx: f32) -> Option<f32> {
    if vx == 0.0 {
        return None;
    }

    for y in span(at.y, at.y + height) {
        let x = if vx > 0.0 {
            ((at.x + width - 0.01) / TILE).floor() as i32
        } else {
            (at.x / TILE).floor() as i32
        };

        if room.solid(x, y) {
            return Some(if vx > 0.0 {
                x as f32 * TILE - width
            } else {
                (x + 1) as f32 * TILE
            });
        }
    }
    None
}

fn push_out_y(room: &Room, at: Vec2, width: f32, height: f32, vy: f32) -> Option<f32> {
    if vy == 0.0 {
        return None;
    }

    for x in span(at.x, at.x + width) {
        let y = if vy > 0.0 {
            ((at.y + height - 0.01) / TILE).floor() as i32
        } else {
            (at.y / TILE).floor() as i32
        };

        // Une plateforme n'arrete qu'en descendant : on la traverse par en bas.
        let blocks =
            room.solid(x, y) || (vy > 0.0 && room.platform(x, y) && crosses(at.y + height, y));

        if blocks {
            return Some(if vy > 0.0 {
                y as f32 * TILE - height
            } else {
                (y + 1) as f32 * TILE
            });
        }
    }
    None
}

/// Si les pieds viennent de franchir le haut de la tuile : sans ce test, une
/// plateforme arreterait un joueur deja passe dessous.
fn crosses(feet: f32, tile_y: i32) -> bool {
    let top = tile_y as f32 * TILE;
    feet >= top && feet <= top + TILE * 0.5
}

/// Les ennemis, la clef et la porte de chaque salle.
fn populate(index: usize, room: &Room) -> (Vec<Foe>, Vec2, Vec2) {
    let at = |c: f32, r: f32| Vec2::new(c * TILE, r * TILE);

    match index {
        0 => (Vec::new(), at(15.0, 2.0), at(17.0, 8.0)),
        1 => (
            vec![
                walker(on_ground(13.0, 10.0)),
                Foe {
                    position: at(6.0, 9.0),
                    velocity: Vec2::ZERO,
                    kind: Kind::Spike,
                    alive: true,
                    origin: at(6.0, 9.0),
                    range: 0.0,
                },
            ],
            at(10.0, 3.0),
            at(17.0, 8.0),
        ),
        _ => (
            vec![
                walker(on_ground(9.0, 10.0)),
                Foe {
                    position: at(10.0, 6.0),
                    velocity: Vec2::new(FLYER_SPEED, 0.0),
                    kind: Kind::Flyer,
                    alive: true,
                    origin: at(10.0, 6.0),
                    range: 3.0 * TILE,
                },
            ],
            at(15.0, 1.0),
            at(1.0, room.height() as f32 - 3.0),
        ),
    }
}

/// Pose un ennemi de 14 px de haut sur le dessus de la ligne `row`.
fn on_ground(column: f32, row: f32) -> Vec2 {
    Vec2::new(column * TILE, row * TILE - 14.0)
}

fn walker(at: Vec2) -> Foe {
    Foe {
        position: at,
        velocity: Vec2::new(FOE_SPEED, 0.0),
        kind: Kind::Walker,
        alive: true,
        origin: at,
        range: 0.0,
    }
}
