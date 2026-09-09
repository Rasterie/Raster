/// Decoded audio samples, ready to play.
///
/// Interleaves quand il y a deux canaux : `[g, d, g, d, …]`, comme le WAV et
/// comme la sortie, ce qui evite une conversion a chaque frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    samples: Vec<f32>,
    channels: u16,
    sample_rate: u32,
}

impl Sound {
    /// # Panics
    ///
    /// Si le nombre de canaux n'est ni 1 ni 2, ou si les echantillons ne se
    /// repartissent pas entre eux : un son mal forme sortirait en bruit.
    #[must_use]
    pub fn new(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        assert!(
            channels == 1 || channels == 2,
            "a sound must be mono or stereo, got {channels} channels"
        );
        assert!(sample_rate > 0, "a sound cannot have a sample rate of 0");
        assert!(
            samples.len().is_multiple_of(channels as usize),
            "{} samples do not split across {channels} channels",
            samples.len()
        );

        Self {
            samples,
            channels,
            sample_rate,
        }
    }

    /// Silence, useful as a placeholder for a sound that failed to load.
    #[must_use]
    pub fn silence(frames: usize, sample_rate: u32) -> Self {
        Self::new(vec![0.0; frames], 1, sample_rate)
    }

    /// The stereo pair at `frame`, mono duplicated across both sides.
    #[must_use]
    pub fn frame(&self, frame: usize) -> Option<(f32, f32)> {
        match self.channels {
            1 => self.samples.get(frame).map(|&s| (s, s)),
            _ => {
                let i = frame * 2;
                Some((*self.samples.get(i)?, *self.samples.get(i + 1)?))
            }
        }
    }

    /// The number of stereo frames, not of samples.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    #[must_use]
    pub fn channels(&self) -> u16 {
        self.channels
    }

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(
            f64::from(self.frames() as u32) / f64::from(self.sample_rate),
        )
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[must_use]
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}
