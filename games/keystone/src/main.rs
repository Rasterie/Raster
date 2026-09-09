//! Keystone : trois salles, une clef par salle, une porte au bout.
//!
//! Touches : ZQSD, WASD ou les fleches pour se deplacer, Espace pour sauter,
//! Entree pour valider, Echap pour la pause.

use keystone::game::{Game, Screen};
use keystone::rooms;
use keystone::save;
use keystone::world::TILE;
use raster_audio::{Audio, Bus, Output, Play, Sound};
use raster_core::asset::AssetId;
use raster_input::{Action, Axis, Input};
use raster_math::Vec2;
use raster_render::{
    App, Camera, Colour, Gpu, Layer, RenderTarget, SpriteBatch, SpriteDraw, Texture, TextureLayout,
    WindowConfig,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 176;

/// Les indices dans la liste de textures.
const SOLID: usize = 0;
const PLATFORM: usize = 1;
const PLAYER: usize = 2;
const FOE: usize = 3;
const KEY: usize = 4;
const DOOR: usize = 5;
const HEART: usize = 6;
const GLYPH: usize = 7;

struct Keystone {
    game: Game,
    batch: Option<SpriteBatch>,
    target: Option<RenderTarget>,
    textures: Vec<Texture>,
    camera: Camera,
    audio: Option<Audio>,
    output: Option<Output>,
    scratch: Vec<f32>,
    sounds: Vec<raster_core::asset::Handle<Sound>>,
    quit: bool,
}

/// Les sons, dans l'ordre ou ils sont ranges.
const S_JUMP: usize = 0;
const S_HURT: usize = 1;
const S_STOMP: usize = 2;
const S_KEY: usize = 3;
const S_DOOR: usize = 4;

impl Keystone {
    fn new() -> Self {
        Self {
            game: Game::new(),
            batch: None,
            target: None,
            textures: Vec::new(),
            camera: Camera::new(WIDTH, HEIGHT),
            audio: None,
            output: None,
            scratch: Vec::new(),
            sounds: Vec::new(),
            quit: false,
        }
    }

    fn play(&mut self, sound: usize, bus: Bus) {
        if let (Some(audio), Some(&handle)) = (self.audio.as_mut(), self.sounds.get(sound)) {
            audio.play(handle, Play::new().on(bus));
        }
    }
}

impl App for Keystone {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();

        self.textures = vec![
            block(gpu, layout, [92, 84, 112], [60, 54, 78]),
            block(gpu, layout, [128, 108, 84], [96, 80, 62]),
            figure(gpu, layout, [232, 224, 208]),
            figure(gpu, layout, [206, 92, 88]),
            gem(gpu, layout, [238, 200, 96]),
            door(gpu, layout),
            gem(gpu, layout, [216, 96, 96]),
            glyphs(gpu, layout),
        ];

        self.batch = Some(batch);
        self.target = Some(RenderTarget::new(gpu, WIDTH, HEIGHT));
        // La salle tient dans l'ecran : la camera reste fixe, centree dessus.
        self.camera.position = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);

        // Le son : absent, le jeu tourne quand meme.
        if let Ok(output) = Output::new() {
            let mut audio = output.audio();
            let rate = audio.sample_rate();
            self.sounds = vec![
                audio.insert(AssetId::new("jump"), tone(430.0, 0.10, rate, 0.20)),
                audio.insert(AssetId::new("hurt"), tone(180.0, 0.22, rate, 0.24)),
                audio.insert(AssetId::new("stomp"), tone(300.0, 0.09, rate, 0.22)),
                audio.insert(AssetId::new("key"), tone(880.0, 0.16, rate, 0.18)),
                audio.insert(AssetId::new("door"), tone(620.0, 0.30, rate, 0.20)),
            ];
            self.audio = Some(audio);
            self.output = Some(output);
        }
    }

    fn fixed_update(&mut self, input: &Input, dt: f32) {
        let axis = input.axis(&Axis::HORIZONTAL);
        let events = self
            .game
            .step(axis < 0.0, axis > 0.0, input.held(&Action::JUMP), dt);

        if events.jumped {
            self.play(S_JUMP, Bus::Sfx);
        }
        if events.hurt {
            self.play(S_HURT, Bus::Sfx);
        }
        if events.stomped {
            self.play(S_STOMP, Bus::Sfx);
        }
        if events.took_key {
            self.play(S_KEY, Bus::Sfx);
        }
        if events.cleared_room || events.won {
            self.play(S_DOOR, Bus::Sfx);
        }

        // La partie se sauvegarde en franchissant une salle, pas a chaque pas.
        if events.cleared_room || events.won {
            let _ = save::save(&self.game.progress, save::default_path());
        }
    }

    fn update(&mut self, input: &mut Input, _time: &raster_core::Time) {
        let confirm = input.pressed(&Action::JUMP) || input.pressed(&Action::INTERACT);

        match self.game.screen {
            Screen::Title => {
                if confirm {
                    match save::load(save::default_path()) {
                        Ok(progress) if progress.room > 0 => self.game.resume(progress),
                        _ => self.game.start(),
                    }
                }
                if input.pressed(&Action::PAUSE) {
                    self.quit = true;
                }
            }
            Screen::Playing => {
                if input.pressed(&Action::PAUSE) {
                    self.game.screen = Screen::Paused;
                }
            }
            Screen::Paused => {
                if input.pressed(&Action::PAUSE) || confirm {
                    self.game.screen = Screen::Playing;
                }
            }
            Screen::Dead => {
                if confirm {
                    self.game.respawn();
                }
            }
            Screen::Won => {
                if confirm {
                    let _ = std::fs::remove_file(save::default_path());
                    self.game = Game::new();
                }
            }
        }

        if let (Some(output), Some(audio)) = (self.output.as_mut(), self.audio.as_mut()) {
            output.pump(audio, &mut self.scratch);
        }
    }

    fn render(&mut self, gpu: &mut Gpu) {
        let (Some(batch), Some(target)) = (self.batch.as_mut(), self.target.as_ref()) else {
            return;
        };
        let Some(mut frame) = gpu.begin_frame() else {
            return;
        };

        target.clear(
            &mut frame,
            raster_render::Color {
                r: 0.055,
                g: 0.05,
                b: 0.085,
                a: 1.0,
            },
        );

        let game = &self.game;

        if game.screen == Screen::Title {
            title(batch);
        } else {
            draw_room(batch, game);
            draw_actors(batch, game);
            draw_hearts(batch, game);

            match game.screen {
                Screen::Paused => banner(batch, 1),
                Screen::Dead => banner(batch, 2),
                Screen::Won => banner(batch, 3),
                _ => {}
            }
        }

        batch.flush_into(gpu, &mut frame, target.view(), self.camera, &self.textures);

        let (w, h) = gpu.size();
        target.present(&mut frame, Vec2::new(w as f32, h as f32));
        gpu.end_frame(frame);
    }

    fn should_exit(&self) -> bool {
        self.quit
    }
}

fn draw_room(batch: &mut SpriteBatch, game: &Game) {
    let room = game.room();

    for y in 0..room.height() as i32 {
        for x in 0..room.width() as i32 {
            let texture = if room.solid(x, y) {
                SOLID
            } else if room.platform(x, y) {
                PLATFORM
            } else {
                continue;
            };

            batch.draw(
                texture,
                SpriteDraw {
                    layer: Layer(-10),
                    ..SpriteDraw::new(
                        Vec2::new(x as f32 * TILE, y as f32 * TILE),
                        Vec2::splat(TILE),
                    )
                },
            );
        }
    }
}

fn draw_actors(batch: &mut SpriteBatch, game: &Game) {
    if let Some(key) = game.key {
        batch.draw(KEY, SpriteDraw::new(key, Vec2::splat(10.0)));
    }

    batch.draw(DOOR, SpriteDraw::new(game.door, Vec2::new(16.0, 24.0)));

    for foe in &game.foes {
        if foe.alive {
            batch.draw(FOE, SpriteDraw::new(foe.position, Vec2::splat(14.0)));
        }
    }

    // Clignote pendant l'invulnerabilite, pour que le coup se voie.
    let blink = game.invulnerable > 0.0 && (game.time * 12.0) as i32 % 2 == 0;
    if !blink {
        batch.draw(
            PLAYER,
            SpriteDraw {
                flip_x: game.facing < 0.0,
                ..SpriteDraw::new(game.player, Vec2::new(12.0, 14.0))
            },
        );
    }
}

fn draw_hearts(batch: &mut SpriteBatch, game: &Game) {
    for i in 0..game.progress.health {
        batch.draw(
            HEART,
            SpriteDraw {
                layer: Layer(10),
                ..SpriteDraw::new(Vec2::new(6.0 + i as f32 * 12.0, 6.0), Vec2::splat(10.0))
            },
        );
    }
}

/// L'ecran-titre : un mot dessine en blocs, faute de rendu de texte avant M4.
fn title(batch: &mut SpriteBatch) {
    banner(batch, 0);
}

/// Les bandeaux, dessines en blocs : `raster-ui` et le texte arrivent en M4.
fn banner(batch: &mut SpriteBatch, which: usize) {
    let (colour, rows) = match which {
        0 => (Colour::WHITE, 3),
        1 => (Colour::rgba(0.7, 0.8, 1.0, 1.0), 1),
        2 => (Colour::rgba(1.0, 0.5, 0.5, 1.0), 1),
        _ => (Colour::rgba(1.0, 0.9, 0.5, 1.0), 2),
    };

    let centre = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    for row in 0..rows {
        let width = 120.0 - row as f32 * 24.0;
        batch.draw(
            GLYPH,
            SpriteDraw {
                tint: colour,
                layer: Layer(20),
                ..SpriteDraw::new(
                    Vec2::new(centre.x - width / 2.0, centre.y - 20.0 + row as f32 * 14.0),
                    Vec2::new(width, 10.0),
                )
            },
        );
    }
}

/// Une note, avec une enveloppe pour qu'elle ne claque pas.
fn tone(frequency: f32, seconds: f32, rate: u32, volume: f32) -> Sound {
    let frames = (rate as f32 * seconds) as usize;
    let mut samples = Vec::with_capacity(frames);

    for i in 0..frames {
        let t = i as f32 / rate as f32;
        let fade = (1.0 - t / seconds).clamp(0.0, 1.0);
        let attack = (t / 0.005).min(1.0);
        samples.push((t * frequency * std::f32::consts::TAU).sin() * volume * fade * attack);
    }

    Sound::new(samples, 1, rate)
}

/// Les textures : dessinees en code, le jeu n'a pas d'assets a livrer.
fn block(gpu: &Gpu, layout: &TextureLayout, top: [u8; 3], body: [u8; 3]) -> Texture {
    let mut pixels = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16u32 {
        for x in 0..16u32 {
            let edge = y < 2 || x == 0 || x == 15;
            let c = if edge { top } else { body };
            // Un grain leger, pour que les blocs ne soient pas plats.
            let n = u8::from((x * 7 + y * 13) % 11 == 0) * 8;
            pixels.extend_from_slice(&[
                c[0].saturating_add(n),
                c[1].saturating_add(n),
                c[2].saturating_add(n),
                255,
            ]);
        }
    }
    Texture::from_rgba(gpu, layout, &pixels, 16, 16)
}

fn figure(gpu: &Gpu, layout: &TextureLayout, colour: [u8; 3]) -> Texture {
    let mut pixels = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16u32 {
        for x in 0..16u32 {
            let inside = (2..14).contains(&x) && (1..15).contains(&y);
            let eye = (4..7).contains(&y) && (x == 5 || x == 10);

            if !inside {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            } else if eye {
                pixels.extend_from_slice(&[24, 22, 32, 255]);
            } else {
                pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 255]);
            }
        }
    }
    Texture::from_rgba(gpu, layout, &pixels, 16, 16)
}

fn gem(gpu: &Gpu, layout: &TextureLayout, colour: [u8; 3]) -> Texture {
    let mut pixels = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16i32 {
        for x in 0..16i32 {
            // Un losange : la distance de Manhattan au centre.
            let d = (x - 8).abs() + (y - 8).abs();
            if d < 7 {
                let bright = u8::from(d < 4) * 30;
                pixels.extend_from_slice(&[
                    colour[0].saturating_add(bright),
                    colour[1].saturating_add(bright),
                    colour[2].saturating_add(bright),
                    255,
                ]);
            } else {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Texture::from_rgba(gpu, layout, &pixels, 16, 16)
}

fn door(gpu: &Gpu, layout: &TextureLayout) -> Texture {
    let mut pixels = Vec::with_capacity(16 * 24 * 4);
    for y in 0..24u32 {
        for x in 0..16u32 {
            let frame = !(2..14).contains(&x) || y < 2;
            let c = if frame { [150, 128, 96] } else { [48, 40, 62] };
            pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    Texture::from_rgba(gpu, layout, &pixels, 16, 24)
}

fn glyphs(gpu: &Gpu, layout: &TextureLayout) -> Texture {
    Texture::from_rgba(gpu, layout, &[255, 255, 255, 255], 1, 1)
}

fn main() {
    for room in rooms::all() {
        if let Err(e) = rooms::check(&room) {
            eprintln!("salle invalide : {e}");
            return;
        }
    }

    let config = WindowConfig {
        title: "Keystone".to_owned(),
        width: WIDTH * 3,
        height: HEIGHT * 3,
        ..WindowConfig::default()
    };

    if let Err(e) = raster_render::run(config, Keystone::new()) {
        eprintln!("keystone: {e}");
    }
}
