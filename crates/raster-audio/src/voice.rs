use crate::{Bus, Sound};
use raster_math::Vec2;

/// A sound currently playing.
#[derive(Debug, Clone)]
pub struct Voice {
    pub sound: usize,
    pub bus: Bus,
    /// La position de lecture, en frames.
    pub cursor: usize,
    pub volume: f32,
    /// -1 a gauche, 0 au centre, 1 a droite.
    pub pan: f32,
    pub looping: bool,
    /// La position dans le monde, `None` pour un son non spatialise.
    pub at: Option<Vec2>,
    pub id: VoiceId,
}

/// Identifies a playing voice, so a game can stop or adjust it later.
///
/// Une generation accompagne l'indice : sans elle, arreter une voix terminee
/// couperait celle qui a repris sa place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VoiceId {
    index: u32,
    generation: u32,
}

impl VoiceId {
    #[must_use]
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[must_use]
    pub(crate) fn index(self) -> u32 {
        self.index
    }

    #[must_use]
    pub(crate) fn generation(self) -> u32 {
        self.generation
    }
}

/// How a sound is placed in the world when it plays.
#[derive(Debug, Clone, Copy)]
pub struct Spatial {
    /// En deca, le son est a plein volume.
    pub near: f32,
    /// Au-dela, il est inaudible.
    pub far: f32,
    /// La distance a laquelle le son est completement d'un cote.
    pub pan_width: f32,
}

impl Spatial {
    #[must_use]
    pub fn new(near: f32, far: f32) -> Self {
        Self {
            near,
            far: far.max(near + f32::EPSILON),
            pan_width: far,
        }
    }

    /// The volume and pan of a sound at `at`, heard from `listener`.
    #[must_use]
    pub fn at(&self, listener: Vec2, at: Vec2) -> (f32, f32) {
        let offset = at - listener;
        let distance = offset.length();

        let volume = if distance <= self.near {
            1.0
        } else if distance >= self.far {
            0.0
        } else {
            // Lineaire plutot qu'inverse : previsible a regler, et un jeu 2D
            // n'a pas de raison de simuler l'acoustique.
            1.0 - (distance - self.near) / (self.far - self.near)
        };

        let pan = if self.pan_width > 0.0 {
            (offset.x / self.pan_width).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        (volume, pan)
    }
}

impl Default for Spatial {
    fn default() -> Self {
        Self::new(64.0, 512.0)
    }
}

/// The gains a pan maps to, left and right.
///
/// Loi en racine carree : a pan nul chaque cote recoit 0,707, dont les
/// puissances somment a 1. Une loi lineaire creuserait un trou au centre.
#[must_use]
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let pan = pan.clamp(-1.0, 1.0);
    let normalised = (pan + 1.0) * 0.5;
    ((1.0 - normalised).sqrt(), normalised.sqrt())
}

impl Voice {
    /// Advances by one frame, returning the stereo pair it contributes.
    ///
    /// `None` quand la voix est terminee.
    pub fn next_frame(&mut self, sound: &Sound, gain: f32) -> Option<(f32, f32)> {
        let (mut left, mut right) = match sound.frame(self.cursor) {
            Some(pair) => pair,
            None if self.looping && sound.frames() > 0 => {
                self.cursor = 0;
                sound.frame(0)?
            }
            None => return None,
        };

        self.cursor += 1;

        let (gl, gr) = pan_gains(self.pan);
        let g = gain * self.volume;
        left *= g * gl;
        right *= g * gr;

        Some((left, right))
    }

    #[must_use]
    pub fn finished(&self, sound: &Sound) -> bool {
        !self.looping && self.cursor >= sound.frames()
    }
}
