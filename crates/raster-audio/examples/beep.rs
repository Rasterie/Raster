//! Verifie que du son sort vraiment : une gamme, panoramiquee de gauche a
//! droite, puis un son qui s'eloigne.
//!
//! `cargo run -p raster-audio --example beep`

use raster_audio::{Bus, Output, Play, Sound, Spatial};
use raster_core::asset::AssetId;
use raster_math::Vec2;
use std::time::{Duration, Instant};

/// Une note, avec une enveloppe simple pour qu'elle ne claque pas.
fn note(frequency: f32, seconds: f32, sample_rate: u32) -> Sound {
    let frames = (sample_rate as f32 * seconds) as usize;
    let mut samples = Vec::with_capacity(frames);

    for i in 0..frames {
        let t = i as f32 / sample_rate as f32;
        let phase = t * frequency * std::f32::consts::TAU;

        // Attaque et extinction courtes : une onde coupee net claque.
        let fade = 0.01;
        let enveloppe = (t / fade)
            .min(1.0)
            .min((seconds - t) / fade)
            .clamp(0.0, 1.0);

        samples.push(phase.sin() * 0.3 * enveloppe);
    }

    Sound::new(samples, 1, sample_rate)
}

fn main() {
    let mut output = match Output::new() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("pas de sortie audio : {e}");
            return;
        }
    };

    println!(
        "sortie : {} Hz, {} canaux",
        output.sample_rate(),
        output.channels()
    );

    let mut audio = output.audio();
    let rate = audio.sample_rate();

    // Une gamme de do majeur.
    let notes: Vec<_> = [261.63, 293.66, 329.63, 349.23, 392.0, 440.0, 493.88, 523.25]
        .iter()
        .enumerate()
        .map(|(i, &f)| audio.insert(AssetId::new(format!("note_{i}.wav")), note(f, 0.35, rate)))
        .collect();

    audio.set_spatial(Spatial::new(0.0, 400.0));

    let mut scratch = Vec::new();
    let mut creux = usize::MAX;
    let debut = Instant::now();
    let mut jouees = 0;

    println!("gamme panoramiquee de gauche a droite...");

    while jouees < notes.len() {
        let attendu = Duration::from_millis(400 * jouees as u64);
        if debut.elapsed() >= attendu {
            // De -1 a 1 au fil de la gamme.
            let pan = (jouees as f32 / (notes.len() - 1) as f32) * 2.0 - 1.0;
            audio.play(notes[jouees], Play::new().pan(pan).on(Bus::Music));
            jouees += 1;
        }

        output.pump(&mut audio, &mut scratch);
        creux = creux.min(output.queued());
        std::thread::sleep(Duration::from_millis(5));
    }

    println!("un son qui s'eloigne... (creux du tampon : {creux} echantillons)");

    let eloigne = audio
        .play(notes[0], Play::new().looping(true).at(Vec2::ZERO))
        .expect("voix libre");

    let depart = Instant::now();
    while depart.elapsed() < Duration::from_secs(3) {
        let t = depart.elapsed().as_secs_f32() / 3.0;
        audio.move_voice(eloigne, Vec2::new(t * 400.0, 0.0));

        output.pump(&mut audio, &mut scratch);
        creux = creux.min(output.queued());
        std::thread::sleep(Duration::from_millis(5));
    }

    println!("creux du tampon sur toute la duree : {creux} echantillons");
    println!("interruptions pendant la lecture : {}", output.starvation());
    audio.stop_all();

    // Laisse le tampon se vider avant de rendre la main.
    std::thread::sleep(Duration::from_millis(300));
    println!("fini");
}
