//! The audio runtime: cues, voices, buses, and the mix.
//!
//! Ce qu'un jeu attend du son, par-dessus la synthese que `resonance-core`
//! fournit : declencher, doser, spatialiser, arreter.

mod bus;
mod engine;
mod output;
mod ring;
mod sound;
mod voice;
mod wav;

pub use bus::{Bus, Mixer};
pub use engine::{Audio, Play};
pub use output::{Output, OutputError};
pub use ring::{Consumer, Producer, ring};
pub use sound::Sound;
pub use voice::{Spatial, Voice, VoiceId, pan_gains};
pub use wav::{WavError, decode};
