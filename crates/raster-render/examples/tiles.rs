//! Un monde de tuiles qu'on parcourt, avec autotuilage.
//!
//! Touches : ZQSD ou les fleches pour se deplacer, Espace pour courir,
//! clic gauche pour poser une tuile, clic droit pour en retirer une.

use raster_2d::{Collision, TileId, TileKind, Tilemap, Tileset};
use raster_input::{Action, Axis, Binding, Input, MouseButton};
use raster_math::{IRect, IVec2, Vec2};
use raster_render::{
    App, Camera, Gpu, Layer, RenderTarget, SpriteBatch, Texture, TilemapRenderer, WindowConfig,
};

const LARGEUR: u32 = 320;
const HAUTEUR: u32 = 180;
const TUILE: u32 = 16;
/// Le tileset fait huit tuiles de large : les 47 variantes tiennent sur six
/// rangees.
const COLONNES: u32 = 8;

const PIERRE: TileId = TileId(1);
/// Le clic droit : une action propre au jeu, absente de la disposition par
/// defaut.
const RETIRER: Action = Action("retirer");

/// Dessine un tileset de 8x8 tuiles.
///
/// La premiere case est vide, puis viennent les 47 variantes d'autotuilage.
/// Chacune est un carre dont les bords sont ouverts la ou le voisin manque —
/// ce qui rend l'autotuilage visible a l'oeil.
fn tileset_genere() -> Vec<u8> {
    const UP: u8 = 1 << 0;
    const RIGHT: u8 = 1 << 2;
    const DOWN: u8 = 1 << 4;
    const LEFT: u8 = 1 << 6;

    let largeur = COLONNES * TUILE;
    let hauteur = COLONNES * TUILE;
    let mut pixels = vec![0u8; (largeur * hauteur * 4) as usize];

    // Retrouve, pour chaque variante, un masque qui la produit : c'est lui qui
    // dit de quel cote la tuile doit etre ouverte.
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
        let index = variante + 1; // la case zero reste vide
        let tx = (index % COLONNES) * TUILE;
        let ty = (index / COLONNES) * TUILE;
        let masque = masque_de_variante[variante as usize];

        for y in 0..TUILE {
            for x in 0..TUILE {
                let bord = 2;
                // Un bord n'est dessine que si le voisin de ce cote manque :
                // deux tuiles voisines se fondent alors l'une dans l'autre.
                let contour = (y < bord && masque & UP == 0)
                    || (y >= TUILE - bord && masque & DOWN == 0)
                    || (x < bord && masque & LEFT == 0)
                    || (x >= TUILE - bord && masque & RIGHT == 0);

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

struct Jeu {
    map: Tilemap,
    batch: Option<SpriteBatch>,
    target: Option<RenderTarget>,
    tiles: Option<TilemapRenderer>,
    textures: Vec<Texture>,
    camera: Camera,
    temps: f32,
    quitter: bool,
}

impl Default for Jeu {
    fn default() -> Self {
        let mut set = Tileset::new(TUILE);
        set.add(TileKind {
            name: "pierre".to_owned(),
            // La premiere variante commence juste apres la case vide.
            atlas: IVec2::new(1, 0),
            collision: Collision::Solid,
            autotile: true,
        });

        Self {
            map: Tilemap::new(set),
            batch: None,
            target: None,
            tiles: None,
            textures: Vec::new(),
            camera: Camera::new(LARGEUR, HAUTEUR),
            temps: 0.0,
            quitter: false,
        }
    }
}

impl App for Jeu {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();

        let taille = COLONNES * TUILE;
        self.textures = vec![Texture::from_rgba(
            gpu,
            layout,
            &tileset_genere(),
            taille,
            taille,
        )];
        self.batch = Some(batch);
        self.target = Some(RenderTarget::new(gpu, LARGEUR, HAUTEUR));
        self.tiles = Some(TilemapRenderer::new(0, COLONNES).with_layer(Layer::BACKGROUND));

        // La vue couvre les tuiles x de -10 a 10 et y de -6 a 5.
        // Le sol, juste sous le bas de la vue initiale.
        self.map.fill(IRect::new(-30, 4, 60, 3), PIERRE);
        // Deux plateformes flottantes.
        self.map.fill(IRect::new(-8, 0, 6, 1), PIERRE);
        self.map.fill(IRect::new(3, -2, 5, 1), PIERRE);
        // Une colonne qui touche le sol, et un bloc massif : les deux montrent
        // l'autotuilage dans des configurations differentes.
        self.map.fill(IRect::new(-13, -3, 2, 7), PIERRE);
        self.map.fill(IRect::new(11, 0, 4, 4), PIERRE);

        // Le clic droit n'est lie a rien par defaut : le jeu declare sa propre
        // action, ce qui est exactement le point du systeme d'actions.
        println!("ZQSD / fleches : se deplacer, Espace : courir");
        println!("clic gauche : poser, clic droit : retirer, Echap : quitter");
        println!(
            "{} tuiles dans {} chunks",
            self.map.tile_count(),
            self.map.chunk_count()
        );
    }

    fn update(&mut self, input: &mut Input, time: &raster_core::Time) {
        self.temps += time.delta;

        // Declare la liaison au premier passage : l'action n'existe pas dans la
        // disposition par defaut.
        if input.bindings().bindings_for(&RETIRER).is_empty() {
            input
                .bindings_mut()
                .bind(RETIRER, Binding::MouseButton(MouseButton::Right));
        }

        if input.pressed(&Action::PAUSE) {
            self.quitter = true;
        }

        // Le deplacement de la camera reste ici : c'est du confort visuel, pas
        // de la simulation.
        let direction = Vec2::new(input.axis(&Axis::HORIZONTAL), input.axis(&Axis::VERTICAL));
        let vitesse = if input.held(&Action::JUMP) {
            300.0
        } else {
            100.0
        };
        self.camera.position += direction.normalized() * vitesse * time.raw_delta;

        // Poser et retirer des tuiles : verifie que la modification a chaud
        // fonctionne, et que l'autotuilage suit.
        let fenetre = Vec2::new(LARGEUR as f32, HAUTEUR as f32) * 3.0;
        let monde = self.camera.screen_to_world(input.mouse_position(), fenetre);
        let tuile = self.map.tile_at(monde);

        if input.held(&Action::ATTACK) {
            self.map.set(tuile, PIERRE);
        }
        if input.held(&RETIRER) {
            self.map.clear(tuile);
        }
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
        batch.flush_into(gpu, &mut frame, target.view(), self.camera, &self.textures);

        let (w, h) = gpu.size();
        target.present(&mut frame, Vec2::new(w as f32, h as f32));

        if self.temps.fract() < 0.017 {
            let s = tiles.stats();
            println!(
                "camera ({:.0}, {:.0}) : {} tuiles dessinees dans {} chunks, {} au total",
                self.camera.position.x,
                self.camera.position.y,
                s.tiles,
                s.chunks,
                self.map.tile_count(),
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
        title: "Raster — tuiles".to_owned(),
        width: LARGEUR * 3,
        height: HAUTEUR * 3,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, Jeu::default()) {
        eprintln!("erreur : {e}");
        std::process::exit(1);
    }
}
