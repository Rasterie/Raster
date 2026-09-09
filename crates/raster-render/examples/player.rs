//! Le jalon M1 : un sprite qui bouge au clavier, pilote par un acteur.
//!
//! Reunit tout ce qui precede — le modele d'acteur, l'index de composants, le
//! rendu de sprites, la camera et les entrees.
//!
//! Touches : ZQSD ou WASD ou les fleches pour se deplacer, Espace pour
//! accelerer, Echap pour quitter.

use raster_core::World;
use raster_core::actor::Component;
use raster_core::asset::{AssetId, Handle, Project};
use raster_core::reflect::Reflect;
use raster_input::{Action, Axis, Input};
use raster_math::{Rect, Vec2};
use raster_render::{
    App, Camera, Colour, Gpu, Layer, RenderTarget, SpriteBatch, SpriteDraw, Texture, Textures,
    WindowConfig,
};

const LARGEUR: u32 = 320;
const HAUTEUR: u32 = 180;

/// Ce qu'un acteur porte pour etre dessine.
#[derive(Reflect, Default, Debug)]
struct Sprite {
    /// Index dans la liste de textures du jeu.
    texture: u32,
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
}

impl Component for Sprite {}

#[derive(Reflect, Default, Debug)]
struct Player {
    #[property(component)]
    sprite: Sprite,
    position: Vec2,
    velocity: Vec2,
    #[property(min = 0.0, max = 500.0)]
    speed: f32,
}

#[derive(Reflect, Default, Debug)]
struct Decor {
    #[property(component)]
    sprite: Sprite,
    position: Vec2,
}

/// Un damier : chaque case fait un pixel, donc une mise a l'echelle non
/// entiere se verrait immediatement.
fn damier(taille: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((taille * taille * 4) as usize);
    for y in 0..taille {
        for x in 0..taille {
            let clair = (x + y).is_multiple_of(2);
            pixels.extend_from_slice(if clair {
                &[90, 90, 110, 255]
            } else {
                &[60, 60, 80, 255]
            });
        }
    }
    pixels
}

struct Jeu {
    temps: f32,
    world: World,
    batch: Option<SpriteBatch>,
    /// Le rendu passe par une cible a la resolution du jeu, agrandie ensuite
    /// d'un facteur entier. Dessiner directement dans la fenetre donnerait des
    /// pixels de largeurs inegales des que celle-ci n'est pas un multiple
    /// exact de 320x180.
    target: Option<RenderTarget>,
    textures: Option<Textures>,
    damier: Option<Handle<Texture>>,
    heros: Option<Handle<Texture>>,
    camera: Camera,
    joueur: Option<raster_core::ActorId>,
    quitter: bool,
}

impl Default for Jeu {
    fn default() -> Self {
        Self {
            temps: 0.0,
            world: World::new(),
            batch: None,
            target: None,
            textures: None,
            damier: None,
            heros: None,
            camera: Camera::new(LARGEUR, HAUTEUR),
            joueur: None,
            quitter: false,
        }
    }
}

impl App for Jeu {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();

        let projet = Project::discover(std::env::current_dir().unwrap_or_default())
            .unwrap_or_else(|| Project::new("."));
        let mut textures = Textures::new(gpu, layout, projet);

        // Le damier est genere, le heros charge par son identifiant d'asset.
        self.damier = Some(textures.insert(
            AssetId::new("<damier>"),
            Texture::from_rgba(gpu, layout, &damier(16), 16, 16),
        ));
        self.heros = Some(textures.load(
            gpu,
            layout,
            &AssetId::new("crates/raster-render/examples/assets/heros.png"),
        ));
        self.textures = Some(textures);
        self.batch = Some(batch);

        // Le batcher indexe une tranche : un handle lui donne son indice.
        let damier_id = self.damier.expect("damier").index();
        let heros_id = self.heros.expect("heros").index();
        self.target = Some(RenderTarget::new(gpu, LARGEUR, HAUTEUR));

        self.world.register_component::<Player, Sprite>();
        self.world.register_component::<Decor, Sprite>();

        // Un sol de damiers, plus quelques blocs poses.
        for i in -10..11 {
            self.world.spawn(Decor {
                sprite: Sprite {
                    texture: damier_id,
                    tint_r: 1.0,
                    tint_g: 1.0,
                    tint_b: 1.0,
                },
                position: Vec2::new(i as f32 * 16.0, 60.0),
            });
        }
        for (x, y) in [(-64.0, 44.0), (-48.0, 44.0), (48.0, 28.0), (64.0, 44.0)] {
            self.world.spawn(Decor {
                sprite: Sprite {
                    texture: damier_id,
                    tint_r: 0.7,
                    tint_g: 0.9,
                    tint_b: 1.0,
                },
                position: Vec2::new(x, y),
            });
        }

        self.joueur = Some(self.world.spawn(Player {
            sprite: Sprite {
                texture: heros_id,
                tint_r: 1.0,
                tint_g: 1.0,
                tint_b: 1.0,
            },
            position: Vec2::new(-8.0, 30.0),
            velocity: Vec2::ZERO,
            speed: 90.0,
        }));

        println!("Deplacement : ZQSD / WASD / fleches. Espace : courir. Echap : quitter.");
    }

    fn update(&mut self, input: &mut Input, time: &raster_core::Time) {
        let dt = time.delta;
        self.temps += dt;

        if input.pressed(&Action::PAUSE) {
            self.quitter = true;
        }

        let direction = Vec2::new(input.axis(&Axis::HORIZONTAL), input.axis(&Axis::VERTICAL));
        let course = if input.held(&Action::JUMP) { 2.5 } else { 1.0 };

        if let Some(id) = self.joueur
            && let Some(player) = self.world.get_mut::<Player>(id)
        {
            // Normalise pour que la diagonale n'aille pas plus vite.
            player.velocity = direction.normalized() * player.speed * course;
            player.position += player.velocity * dt;
        }

        // La camera suit sans s'accrocher a la grille : c'est ce qui rend le
        // defilement fluide pendant que les sprites restent nets.
        if let Some(id) = self.joueur
            && let Some(player) = self.world.get::<Player>(id)
        {
            self.camera
                .follow(player.position + Vec2::splat(8.0), 0.15, dt);
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
                r: 0.05,
                g: 0.05,
                b: 0.09,
                a: 1.0,
            },
        );

        // Le decor, puis le joueur par-dessus.
        for (_, decor) in self.world.iter::<Decor>() {
            batch.draw(
                decor.sprite.texture as usize,
                SpriteDraw {
                    position: decor.position,
                    tint: Colour::rgb(
                        decor.sprite.tint_r,
                        decor.sprite.tint_g,
                        decor.sprite.tint_b,
                    ),
                    layer: Layer::BACKGROUND,
                    ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
                },
            );
        }

        for (_, player) in self.world.iter::<Player>() {
            batch.draw(
                player.sprite.texture as usize,
                SpriteDraw {
                    position: player.position,
                    // Le personnage se retourne selon son sens de deplacement.
                    flip_x: player.velocity.x < 0.0,
                    source: Rect::new(0.0, 0.0, 16.0, 16.0),
                    ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
                },
            );
        }

        let textures = self.textures.as_ref().expect("textures");
        batch.flush_into(
            gpu,
            &mut frame,
            target.view(),
            self.camera,
            textures.as_slice(),
        );

        // Agrandit la cible sur la fenetre, d'un facteur entier.
        let (w, h) = gpu.size();
        target.present(&mut frame, Vec2::new(w as f32, h as f32));

        if self.temps.fract() < 0.017 {
            let s = batch.stats();
            let position = self
                .joueur
                .and_then(|id| self.world.get::<Player>(id))
                .map_or(Vec2::ZERO, |p| p.position);
            println!(
                "joueur ({:.0}, {:.0}), camera ({:.1}, {:.1}), {} sprites en {} appels, \
                 rendu {}x{} agrandi x{}",
                position.x,
                position.y,
                self.camera.position.x,
                self.camera.position.y,
                s.sprites,
                s.draw_calls,
                LARGEUR,
                HAUTEUR,
                target.scale(Vec2::new(w as f32, h as f32)),
            );
        }

        gpu.end_frame(frame);
    }

    fn should_exit(&self) -> bool {
        self.quitter
    }
}

fn main() {
    let config = WindowConfig {
        title: "Raster — un sprite qui bouge".to_owned(),
        width: LARGEUR * 3,
        height: HAUTEUR * 3,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, Jeu::default()) {
        eprintln!("erreur : {e}");
        std::process::exit(1);
    }
}
