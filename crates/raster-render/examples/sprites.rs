//! Dessine des sprites generes a la main et verifie le rendu pixel-perfect.
//!
//! La grille en damier rend visible tout defaut d'echelle : si un pixel est
//! plus large qu'un autre, cela se voit immediatement.

use raster_math::{Rect, Vec2};
use raster_render::{
    App, Camera, Colour, Gpu, Layer, SpriteBatch, SpriteDraw, Texture, WindowConfig,
};

/// Resolution interne du jeu. La fenetre est plus grande et l'image est
/// agrandie par un facteur entier.
const LARGEUR: u32 = 320;
const HAUTEUR: u32 = 180;

struct Demo {
    batch: Option<SpriteBatch>,
    textures: Vec<Texture>,
    camera: Camera,
    temps: f32,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            batch: None,
            textures: Vec::new(),
            camera: Camera::new(LARGEUR, HAUTEUR),
            temps: 0.0,
        }
    }
}

/// Un damier : chaque case fait un pixel, donc toute mise a l'echelle non
/// entiere produirait un moire immediatement visible.
fn damier(taille: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((taille * taille * 4) as usize);
    for y in 0..taille {
        for x in 0..taille {
            let clair = (x + y) % 2 == 0;
            pixels.extend_from_slice(if clair {
                &[220, 220, 230, 255]
            } else {
                &[40, 40, 60, 255]
            });
        }
    }
    pixels
}

/// Un disque plein, pour verifier le canal alpha et le decoupage.
fn disque(taille: u32, couleur: [u8; 3]) -> Vec<u8> {
    let centre = taille as f32 / 2.0;
    let rayon = centre - 0.5;
    let mut pixels = Vec::with_capacity((taille * taille * 4) as usize);

    for y in 0..taille {
        for x in 0..taille {
            let dx = x as f32 + 0.5 - centre;
            let dy = y as f32 + 0.5 - centre;
            let dedans = dx * dx + dy * dy <= rayon * rayon;
            if dedans {
                pixels.extend_from_slice(&[couleur[0], couleur[1], couleur[2], 255]);
            } else {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    pixels
}

impl App for Demo {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();

        self.textures = vec![
            Texture::from_rgba(gpu, layout, &damier(16), 16, 16),
            Texture::from_rgba(gpu, layout, &disque(16, [230, 90, 70]), 16, 16),
            Texture::placeholder(gpu, layout),
        ];
        self.batch = Some(batch);

        println!("resolution interne : {LARGEUR}x{HAUTEUR}");
        let (w, h) = gpu.size();
        println!(
            "fenetre {w}x{h}, facteur d'agrandissement {}",
            self.camera.window_scale(Vec2::new(w as f32, h as f32))
        );
    }

    fn update(&mut self, dt: f32) {
        self.temps += dt;

        // La camera derive en sous-pixels : c'est ce qui teste le couple
        // sprites accroches / camera libre.
        self.camera.position = Vec2::new((self.temps * 12.0).sin() * 30.0, 0.0);
    }

    fn render(&mut self, gpu: &mut Gpu) {
        let Some(batch) = self.batch.as_mut() else {
            return;
        };
        let Some(mut frame) = gpu.begin_frame() else {
            return;
        };

        frame.clear(raster_render::Color {
            r: 0.04,
            g: 0.04,
            b: 0.07,
            a: 1.0,
        });

        // Une rangee de damiers au fond.
        for i in -6..7 {
            batch.draw(
                0,
                SpriteDraw {
                    position: Vec2::new(i as f32 * 20.0 - 8.0, -50.0),
                    layer: Layer::BACKGROUND,
                    ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
                },
            );
        }

        // Des disques qui tournent, teintes differemment.
        for i in 0..8 {
            let angle = self.temps + i as f32 * std::f32::consts::TAU / 8.0;
            let position = Vec2::from_angle(angle) * 45.0 - Vec2::splat(8.0);
            let teinte = i as f32 / 8.0;

            batch.draw(
                1,
                SpriteDraw {
                    position,
                    tint: Colour::rgb(1.0, 0.4 + teinte * 0.6, 0.3 + teinte * 0.7),
                    ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
                },
            );
        }

        // Un sprite retourne, et un demi-sprite : verifient le decoupage.
        batch.draw(
            1,
            SpriteDraw {
                position: Vec2::new(-70.0, 40.0),
                flip_x: true,
                ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
            },
        );
        batch.draw(
            0,
            SpriteDraw {
                position: Vec2::new(50.0, 40.0),
                size: Vec2::new(8.0, 16.0),
                source: Rect::new(0.0, 0.0, 8.0, 16.0),
                ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
            },
        );

        // Le damier de texture manquante, au premier plan.
        batch.draw(
            2,
            SpriteDraw {
                position: Vec2::new(-8.0, 60.0),
                layer: Layer::FOREGROUND,
                ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
            },
        );

        // Deux sprites loin hors du champ : sans elimination ils atteindraient
        // le GPU, et le releve ci-dessous le dirait.
        for x in [-2000.0, 2000.0] {
            batch.draw(0, SpriteDraw::new(Vec2::new(x, 0.0), Vec2::splat(16.0)));
        }

        batch.flush(gpu, &mut frame, self.camera, &self.textures);

        // Une fois par seconde environ, un releve de ce que la frame a coute.
        if self.temps.fract() < 0.016 {
            let s = batch.stats();
            println!(
                "{} sprites, {} appels de dessin, {} elimines",
                s.sprites, s.draw_calls, s.culled
            );
        }

        gpu.end_frame(frame);
    }
}

fn main() {
    let config = WindowConfig {
        title: "Raster — sprites".to_owned(),
        width: LARGEUR * 3,
        height: HAUTEUR * 3,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, Demo::default()) {
        eprintln!("erreur : {e}");
        std::process::exit(1);
    }
}
