//! Un personnage qui court, saute et se cogne aux murs.
//!
//! Reunit les tuiles, la collision, les entrees et le rendu. Touches : ZQSD ou
//! les fleches pour se deplacer, Espace pour sauter, S pour descendre d'une
//! plateforme, Echap pour quitter.

use raster_2d::{Animation, Collision, Repeat, StateMachine, TileId, TileKind, Tilemap, Tileset};
use raster_input::{Action, Axis, Grace, Input};
use raster_math::{IRect, IVec2, Rect, Vec2};
use raster_physics::{Body, move_body};
use raster_render::{
    App, Camera, Colour, Gpu, Layer, RenderTarget, SpriteBatch, SpriteDraw, Texture,
    TilemapRenderer, WindowConfig,
};

const LARGEUR: u32 = 320;
const HAUTEUR: u32 = 180;
const TUILE: u32 = 16;
const COLONNES: u32 = 8;

const PIERRE: TileId = TileId(1);
const PLATEFORME: TileId = TileId(2);

/// Reglages du personnage, groupes pour etre lisibles d'un coup d'oeil.
const VITESSE: f32 = 90.0;
const GRAVITE: f32 = 700.0;
const SAUT: f32 = 240.0;
/// Au-dela, la chute traverserait trop de tuiles par frame.
const CHUTE_MAX: f32 = 500.0;

fn tileset_genere() -> Vec<u8> {
    const UP: u8 = 1 << 0;
    const RIGHT: u8 = 1 << 2;
    const DOWN: u8 = 1 << 4;
    const LEFT: u8 = 1 << 6;

    let largeur = COLONNES * TUILE;
    let mut pixels = vec![0u8; (largeur * largeur * 4) as usize];

    let mut masque_de_variante = [0u8; 47];
    let mut vus = [false; 47];
    for masque in 0..=255u8 {
        let v = raster_render::tile_variant(masque) as usize;
        if !vus[v] {
            vus[v] = true;
            masque_de_variante[v] = masque;
        }
    }

    for variante in 0..47u32 {
        let index = variante + 1;
        let (tx, ty) = ((index % COLONNES) * TUILE, (index / COLONNES) * TUILE);
        let masque = masque_de_variante[variante as usize];

        for y in 0..TUILE {
            for x in 0..TUILE {
                let contour = (y < 2 && masque & UP == 0)
                    || (y >= TUILE - 2 && masque & DOWN == 0)
                    || (x < 2 && masque & LEFT == 0)
                    || (x >= TUILE - 2 && masque & RIGHT == 0);

                let couleur = if contour {
                    [120, 140, 170, 255]
                } else {
                    [55, 60, 80, 255]
                };
                let p = (((ty + y) * largeur + tx + x) * 4) as usize;
                pixels[p..p + 4].copy_from_slice(&couleur);
            }
        }
    }
    pixels
}

/// Une planche de quatre images de 16x16, cote a cote.
///
/// Repos, deux images de marche et un saut : les jambes changent de position,
/// ce qui rend l'animation visible sans dessin elabore.
fn heros() -> Vec<u8> {
    const IMAGES: [[&str; 16]; 4] = [
        // repos
        [
            "................",
            "................",
            "....######......",
            "...########.....",
            "...##.##.##.....",
            "...########.....",
            "....######......",
            "...#########....",
            "..##.######.##..",
            "..##.######.##..",
            ".....######.....",
            ".....######.....",
            "....###..###....",
            "....###..###....",
            "...####..####...",
            "................",
        ],
        // marche 1
        [
            "................",
            "................",
            "....######......",
            "...########.....",
            "...##.##.##.....",
            "...########.....",
            "....######......",
            "..##########....",
            ".##..######.....",
            ".##..######.###.",
            ".....######.###.",
            ".....######.....",
            "...####...###...",
            "..####.....###..",
            "..###.......###.",
            "................",
        ],
        // marche 2
        [
            "................",
            "................",
            "....######......",
            "...########.....",
            "...##.##.##.....",
            "...########.....",
            "....######......",
            "...#########....",
            "..##.######.##..",
            "..##.######.##..",
            ".....######.....",
            ".....######.....",
            "....######......",
            "...###..###.....",
            "..###....####...",
            "................",
        ],
        // saut
        [
            "................",
            "................",
            "....######......",
            "...########.....",
            "...##.##.##.....",
            "...########.....",
            "....######......",
            ".##########.##..",
            ".##..######..##.",
            ".....######.....",
            ".....######.....",
            "....##....##....",
            "...###....###...",
            "..###......###..",
            "................",
            "................",
        ],
    ];

    let largeur = 16 * IMAGES.len() as u32;
    let mut pixels = vec![0u8; (largeur * 16 * 4) as usize];

    for (i, image) in IMAGES.iter().enumerate() {
        let ox = i as u32 * 16;
        for (y, ligne) in image.iter().enumerate() {
            for (x, c) in ligne.chars().enumerate() {
                if c != '#' {
                    continue;
                }
                let p = (((y as u32) * largeur + ox + x as u32) * 4) as usize;
                pixels[p..p + 4].copy_from_slice(&[235, 215, 180, 255]);
            }
        }
    }
    pixels
}

/// Les etats du personnage et leurs transitions.
fn machine_du_heros() -> StateMachine {
    let mut m = StateMachine::new();
    m.add("repos", Animation::new([0], 6.0));
    m.add(
        "marche",
        Animation::new([1, 0, 2, 0], 10.0).with_event(0, "pas"),
    );
    m.add("saut", Animation::new([3], 6.0).with_repeat(Repeat::Once));

    // L'ordre de declaration fait la priorite : en l'air l'emporte.
    m.transition_any("saut", "en l'air");
    m.transition_any("marche", "court");
    m.transition_any("repos", "immobile");
    m
}

struct Jeu {
    map: Tilemap,
    joueur: Body,
    /// Coyote time : sauter juste apres avoir quitte une plateforme marche.
    sol: Grace,
    anim: StateMachine,
    regarde_a_gauche: bool,
    batch: Option<SpriteBatch>,
    target: Option<RenderTarget>,
    tiles: Option<TilemapRenderer>,
    textures: Vec<Texture>,
    camera: Camera,
    quitter: bool,
}

impl Default for Jeu {
    fn default() -> Self {
        let mut set = Tileset::new(TUILE);
        set.add(TileKind {
            name: "pierre".to_owned(),
            atlas: IVec2::new(1, 0),
            collision: Collision::Solid,
            autotile: true,
        });
        set.add(TileKind {
            name: "plateforme".to_owned(),
            atlas: IVec2::new(1, 0),
            collision: Collision::OneWay,
            autotile: true,
        });

        Self {
            map: Tilemap::new(set),
            joueur: Body::new(Rect::new(0.0, -40.0, 10.0, 14.0)),
            sol: Grace::default(),
            anim: machine_du_heros(),
            regarde_a_gauche: false,
            batch: None,
            target: None,
            tiles: None,
            textures: Vec::new(),
            camera: Camera::new(LARGEUR, HAUTEUR),
            quitter: false,
        }
    }
}

impl App for Jeu {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();

        let taille = COLONNES * TUILE;
        self.textures = vec![
            Texture::from_rgba(gpu, layout, &tileset_genere(), taille, taille),
            Texture::from_rgba(gpu, layout, &heros(), 64, 16),
        ];
        self.batch = Some(batch);
        self.target = Some(RenderTarget::new(gpu, LARGEUR, HAUTEUR));
        self.tiles = Some(TilemapRenderer::new(0, COLONNES).with_layer(Layer::BACKGROUND));

        // Le sol, des marches, un mur et une plateforme traversable.
        self.map.fill(IRect::new(-20, 4, 40, 3), PIERRE);
        self.map.fill(IRect::new(4, 3, 2, 1), PIERRE);
        self.map.fill(IRect::new(6, 2, 2, 1), PIERRE);
        self.map.fill(IRect::new(8, 1, 2, 1), PIERRE);
        self.map.fill(IRect::new(-9, 0, 1, 4), PIERRE);
        self.map.fill(IRect::new(-6, 1, 4, 1), PLATEFORME);

        println!("ZQSD / fleches : courir, Espace : sauter, S : descendre");
    }

    fn fixed_update(&mut self, input: &Input, dt: f32) {
        let dir = input.axis(&Axis::HORIZONTAL);
        self.joueur.velocity.x = dir * VITESSE;
        if dir != 0.0 {
            self.regarde_a_gauche = dir < 0.0;
        }

        // Traverse une plateforme en maintenant bas.
        self.joueur.drop_through = input.axis(&Axis::VERTICAL) > 0.0;

        // Le saut consomme le delai de grace, pour qu'il ne serve qu'une fois.
        if input.held(&Action::JUMP) && self.sol.active() {
            self.joueur.velocity.y = -SAUT;
            self.sol.consume();
        }

        self.joueur.velocity.y = (self.joueur.velocity.y + GRAVITE * dt).min(CHUTE_MAX);

        let contacts = move_body(&mut self.joueur, &self.map, dt);
        self.sol.update(contacts.grounded(), dt);

        let etat = if !contacts.grounded() {
            "en l'air"
        } else if dir != 0.0 {
            "court"
        } else {
            "immobile"
        };
        self.anim.update(&[etat], dt);
    }

    fn update(&mut self, input: &mut Input, time: &raster_core::Time) {
        if input.pressed(&Action::PAUSE) {
            self.quitter = true;
        }
        self.camera
            .follow(self.joueur.bounds.center(), 0.12, time.raw_delta);
    }

    fn render(&mut self, gpu: &mut Gpu) {
        let (Some(batch), Some(target), Some(tiles)) = (
            self.batch.as_mut(),
            self.target.as_ref(),
            self.tiles.as_mut(),
        ) else {
            return;
        };
        let Some(mut frame) = gpu.begin_frame() else {
            return;
        };

        target.clear(
            &mut frame,
            raster_render::Color {
                r: 0.05,
                g: 0.06,
                b: 0.10,
                a: 1.0,
            },
        );

        tiles.draw(batch, &self.map, self.camera);

        // Le sprite fait 16x16 mais la boite 10x14 : on centre l'un sur l'autre.
        let decalage = Vec2::new(-3.0, -2.0);
        let image = self.anim.index() as f32;
        batch.draw(
            1,
            SpriteDraw {
                position: self.joueur.position() + decalage,
                flip_x: self.regarde_a_gauche,
                tint: Colour::WHITE,
                layer: Layer::DEFAULT,
                // La planche fait quatre images de 16 px cote a cote.
                source: Rect::new(image * 16.0, 0.0, 16.0, 16.0),
                ..SpriteDraw::new(Vec2::ZERO, Vec2::splat(16.0))
            },
        );

        batch.flush_into(gpu, &mut frame, target.view(), self.camera, &self.textures);

        let (w, h) = gpu.size();
        target.present(&mut frame, Vec2::new(w as f32, h as f32));
        gpu.end_frame(frame);
    }

    fn should_exit(&self) -> bool {
        self.quitter
    }
}

fn main() {
    let config = WindowConfig {
        title: "Raster — plateformes".to_owned(),
        width: LARGEUR * 3,
        height: HAUTEUR * 3,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, Jeu::default()) {
        eprintln!("erreur : {e}");
        std::process::exit(1);
    }
}
