//! Ouvre une fenetre et l'efface a une couleur qui varie dans le temps.
//!
//! Sert de preuve que la fenetre et le GPU fonctionnent : un test automatise ne
//! peut pas verifier qu'une image apparait a l'ecran.

use raster_render::{App, Color, Gpu, WindowConfig};

#[derive(Default)]
struct Demo {
    elapsed: f32,
    /// Images et duree depuis la derniere mesure, plutot que depuis le
    /// demarrage : une moyenne cumulee inclurait le temps d'initialisation et
    /// masquerait toute variation.
    frames_depuis_mesure: u32,
    temps_depuis_mesure: f32,
}

impl App for Demo {
    fn init(&mut self, gpu: &mut Gpu) {
        let (width, height) = gpu.size();
        println!("surface {width}x{height}, format {:?}", gpu.format());
        println!("mode de presentation : {:?}", gpu.present_mode());
    }

    fn update(&mut self, _input: &mut raster_input::Input, time: &raster_core::Time) {
        let dt = time.raw_delta;
        self.elapsed += dt;
        self.frames_depuis_mesure += 1;
        self.temps_depuis_mesure += dt;

        if self.temps_depuis_mesure >= 1.0 {
            let ips = f64::from(self.frames_depuis_mesure) / f64::from(self.temps_depuis_mesure);
            println!("{ips:.0} images/s");
            self.frames_depuis_mesure = 0;
            self.temps_depuis_mesure = 0.0;
        }
    }

    fn render(&mut self, gpu: &mut Gpu) {
        let Some(mut frame) = gpu.begin_frame() else {
            return;
        };

        let pulse = f64::from(self.elapsed.sin().abs());
        frame.clear(Color {
            r: pulse * 0.2,
            g: pulse * 0.1,
            b: pulse * 0.35,
            a: 1.0,
        });

        gpu.end_frame(frame);
    }

    fn resized(&mut self, width: u32, height: u32) {
        println!("redimensionne : {width}x{height}");
    }
}

fn main() {
    let config = WindowConfig {
        title: "Raster — fenetre".to_owned(),
        width: 960,
        height: 540,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, Demo::default()) {
        eprintln!("erreur : {e}");
        std::process::exit(1);
    }
}
