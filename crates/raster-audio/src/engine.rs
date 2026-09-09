use crate::{Bus, Mixer, Sound, Spatial, Voice, VoiceId};
use raster_core::asset::{AssetId, AssetStore, Handle};
use raster_math::Vec2;

/// How a sound is asked to play.
#[derive(Debug, Clone, Copy)]
pub struct Play {
    pub bus: Bus,
    pub volume: f32,
    pub pan: f32,
    pub looping: bool,
    /// La position dans le monde, `None` pour un son non spatialise.
    pub at: Option<Vec2>,
}

impl Play {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bus: Bus::Sfx,
            volume: 1.0,
            pan: 0.0,
            looping: false,
            at: None,
        }
    }

    #[must_use]
    pub fn on(mut self, bus: Bus) -> Self {
        self.bus = bus;
        self
    }

    #[must_use]
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    #[must_use]
    pub fn pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    #[must_use]
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    #[must_use]
    pub fn at(mut self, position: Vec2) -> Self {
        self.at = Some(position);
        self
    }
}

impl Default for Play {
    fn default() -> Self {
        Self::new()
    }
}

/// The mix: sounds, playing voices, and where the listener stands.
///
/// Sans peripherique : `render` remplit une tranche, ce qui rend tout le
/// melange testable et permet d'exporter aussi bien que de jouer.
pub struct Audio {
    sounds: AssetStore<Sound>,
    voices: Vec<Slot>,
    mixer: Mixer,
    spatial: Spatial,
    listener: Vec2,
    sample_rate: u32,
    /// Les voix libres, pour ne pas parcourir la liste a chaque declenchement.
    free: Vec<u32>,
}

struct Slot {
    voice: Option<Voice>,
    generation: u32,
}

impl Audio {
    /// Trente-deux voix : au-dela, un jeu 2D empile des sons que personne ne
    /// distingue, et le vol de voix coute moins que le melange.
    pub const DEFAULT_VOICES: usize = 32;

    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        Self::with_voices(sample_rate, Self::DEFAULT_VOICES)
    }

    #[must_use]
    pub fn with_voices(sample_rate: u32, voices: usize) -> Self {
        let mut slots = Vec::with_capacity(voices);
        let mut free = Vec::with_capacity(voices);
        for i in 0..voices {
            slots.push(Slot {
                voice: None,
                generation: 1,
            });
            free.push((voices - 1 - i) as u32);
        }

        Self {
            sounds: AssetStore::new(),
            voices: slots,
            mixer: Mixer::new(),
            spatial: Spatial::default(),
            listener: Vec2::ZERO,
            sample_rate,
            free,
        }
    }

    /// Stores a decoded sound under its identifier.
    pub fn insert(&mut self, id: AssetId, sound: Sound) -> Handle<Sound> {
        self.sounds.insert(id, sound)
    }

    /// Loads a sound, calling `load` only the first time.
    ///
    /// # Errors
    ///
    /// Whatever `load` returns.
    pub fn load_with<E>(
        &mut self,
        id: &AssetId,
        load: impl FnOnce(&AssetId) -> Result<Sound, E>,
    ) -> Result<Handle<Sound>, E> {
        self.sounds.load_with(id, load)
    }

    #[must_use]
    pub fn sound(&self, handle: Handle<Sound>) -> Option<&Sound> {
        self.sounds.get(handle)
    }

    #[must_use]
    pub fn handle(&self, id: &AssetId) -> Option<Handle<Sound>> {
        self.sounds.handle(id)
    }

    /// Starts a sound, stealing the quietest voice if none is free.
    ///
    /// `None` seulement si le son est inconnu : un declenchement ne doit pas
    /// echouer parce que le jeu est bruyant.
    pub fn play(&mut self, handle: Handle<Sound>, how: Play) -> Option<VoiceId> {
        let sound = self.sounds.get(handle)?;
        if sound.is_empty() {
            return None;
        }

        let index = self.free.pop().or_else(|| self.steal())?;
        let slot = &mut self.voices[index as usize];
        let id = VoiceId::new(index, slot.generation);

        let (volume, pan) = match how.at {
            Some(at) => {
                let (v, p) = self.spatial.at(self.listener, at);
                (how.volume * v, p)
            }
            None => (how.volume, how.pan),
        };

        slot.voice = Some(Voice {
            sound: handle.index() as usize,
            bus: how.bus,
            cursor: 0,
            volume,
            pan,
            looping: how.looping,
            at: how.at,
            id,
        });

        Some(id)
    }

    /// The quietest voice, which is the least missed when one must go.
    fn steal(&mut self) -> Option<u32> {
        let (index, _) = self
            .voices
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                let voice = slot.voice.as_ref()?;
                // Une boucle est un fond sonore : la voler couperait une
                // musique pour un bruit de pas.
                if voice.looping {
                    return None;
                }
                Some((i, voice.volume * self.mixer.gain(voice.bus)))
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?;

        // La generation avance : sans cela, l'identifiant de la voix volee
        // designerait celle qui prend sa place, et l'arreter la couperait.
        let slot = &mut self.voices[index];
        slot.voice = None;
        slot.generation = slot.generation.wrapping_add(1);
        Some(index as u32)
    }

    /// Stops a voice, if it is still the one that was started.
    pub fn stop(&mut self, id: VoiceId) -> bool {
        let Some(slot) = self.voices.get_mut(id.index() as usize) else {
            return false;
        };
        if slot.generation != id.generation() || slot.voice.is_none() {
            return false;
        }

        slot.voice = None;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index());
        true
    }

    pub fn stop_all(&mut self) {
        for index in 0..self.voices.len() as u32 {
            let slot = &mut self.voices[index as usize];
            if slot.voice.take().is_some() {
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(index);
            }
        }
    }

    pub fn stop_bus(&mut self, bus: Bus) {
        for index in 0..self.voices.len() as u32 {
            let slot = &mut self.voices[index as usize];
            if slot.voice.as_ref().is_some_and(|v| v.bus == bus) {
                slot.voice = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(index);
            }
        }
    }

    #[must_use]
    pub fn is_playing(&self, id: VoiceId) -> bool {
        self.voices
            .get(id.index() as usize)
            .is_some_and(|s| s.generation == id.generation() && s.voice.is_some())
    }

    #[must_use]
    pub fn playing(&self) -> usize {
        self.voices.iter().filter(|s| s.voice.is_some()).count()
    }

    /// Fills `out` with interleaved stereo frames, advancing every voice.
    ///
    /// Aucune allocation : c'est ce qui rend l'appel sur un thread audio sur,
    /// ou un depassement s'entend comme un craquement.
    pub fn render(&mut self, out: &mut [f32]) {
        out.fill(0.0);

        for index in 0..self.voices.len() {
            let Some(voice) = self.voices[index].voice.as_mut() else {
                continue;
            };
            let Some(sound) = self.sounds.as_slice().get(voice.sound) else {
                continue;
            };

            let gain = self.mixer.gain(voice.bus);
            let mut finished = false;

            for pair in out.chunks_exact_mut(2) {
                match voice.next_frame(sound, gain) {
                    Some((l, r)) => {
                        pair[0] += l;
                        pair[1] += r;
                    }
                    None => {
                        finished = true;
                        break;
                    }
                }
            }

            if finished {
                let slot = &mut self.voices[index];
                slot.voice = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(index as u32);
            }
        }
    }

    /// Where the listener stands, which spatialised sounds are heard from.
    pub fn set_listener(&mut self, at: Vec2) {
        self.listener = at;
    }

    #[must_use]
    pub fn listener(&self) -> Vec2 {
        self.listener
    }

    /// Re-places the spatialised voices, after the listener moved.
    pub fn update_spatial(&mut self) {
        for slot in &mut self.voices {
            let Some(voice) = slot.voice.as_mut() else {
                continue;
            };
            let Some(at) = voice.at else { continue };

            let (volume, pan) = self.spatial.at(self.listener, at);
            voice.volume = volume;
            voice.pan = pan;
        }
    }

    /// Moves a playing voice, for a sound attached to something that moves.
    pub fn move_voice(&mut self, id: VoiceId, to: Vec2) -> bool {
        let listener = self.listener;
        let spatial = self.spatial;
        let Some(slot) = self.voices.get_mut(id.index() as usize) else {
            return false;
        };
        if slot.generation != id.generation() {
            return false;
        }
        let Some(voice) = slot.voice.as_mut() else {
            return false;
        };

        let (volume, pan) = spatial.at(listener, to);
        voice.at = Some(to);
        voice.volume = volume;
        voice.pan = pan;
        true
    }

    #[must_use]
    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    pub fn mixer_mut(&mut self) -> &mut Mixer {
        &mut self.mixer
    }

    pub fn set_spatial(&mut self, spatial: Spatial) {
        self.spatial = spatial;
    }

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub fn voices(&self) -> usize {
        self.voices.len()
    }
}
