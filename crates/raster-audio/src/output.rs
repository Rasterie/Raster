use crate::{Audio, Consumer, Producer, ring};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// The sound card, fed from a ring the game fills.
///
/// Le melange se fait cote jeu : le callback ne fait que copier, sans allouer,
/// sans verrou et sans toucher au disque.
pub struct Output {
    stream: cpal::Stream,
    producer: Producer,
    sample_rate: u32,
    channels: u16,
    /// Les fois ou le callback a manque d'echantillons : un craquement chacune,
    /// sauf apres le dernier son, ou le silence est voulu.
    starvation: Arc<AtomicU32>,
}

/// Why the sound card could not be opened.
#[derive(Debug)]
pub enum OutputError {
    NoDevice,
    /// The device offers no configuration this engine can use.
    NoConfig(String),
    Build(cpal::Error),
    Play(cpal::Error),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDevice => write!(f, "no audio output device"),
            Self::NoConfig(what) => write!(f, "no usable audio configuration: {what}"),
            Self::Build(e) => write!(f, "could not open the audio stream: {e}"),
            Self::Play(e) => write!(f, "could not start the audio stream: {e}"),
        }
    }
}

impl std::error::Error for OutputError {}

impl Output {
    /// Un quart de seconde d'avance : assez pour absorber une frame lente,
    /// assez peu pour qu'un son declenche parte sans retard perceptible.
    pub const DEFAULT_LATENCY: f32 = 0.25;

    /// Opens the default output device.
    ///
    /// # Errors
    ///
    /// If there is no device, or none this engine can drive.
    pub fn new() -> Result<Self, OutputError> {
        Self::with_latency(Self::DEFAULT_LATENCY)
    }

    /// Opens the default device, buffering `latency` seconds ahead.
    ///
    /// # Errors
    ///
    /// As [`Output::new`].
    pub fn with_latency(latency: f32) -> Result<Self, OutputError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(OutputError::NoDevice)?;

        let default = device
            .default_output_config()
            .map_err(|e| OutputError::NoConfig(e.to_string()))?;

        let sample_rate = default.sample_rate();
        let format = default.sample_format();

        // Deux canaux imposes : le melange produit du stereo entrelace, et
        // l'envoyer tel quel a une sortie 5.1 le repartirait de travers.
        let channels = 2;
        let config = cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let capacity = ((sample_rate as f32 * latency) as usize).max(1024) * channels as usize;
        let (producer, consumer) = ring(capacity);

        let starvation = Arc::new(AtomicU32::new(0));
        let stream = Self::build(&device, &config, format, consumer, &starvation)?;
        stream.play().map_err(OutputError::Play)?;

        Ok(Self {
            stream,
            producer,
            sample_rate,
            channels,
            starvation,
        })
    }

    fn build(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        format: cpal::SampleFormat,
        mut consumer: Consumer,
        starvation: &Arc<AtomicU32>,
    ) -> Result<cpal::Stream, OutputError> {
        if format != cpal::SampleFormat::F32 {
            return Err(OutputError::NoConfig(format!(
                "sample format {format:?} — only f32 is handled"
            )));
        }

        // Compte plutot que d'avertir : le callback ne sait pas si le tampon
        // est vide parce que le jeu a pris du retard ou parce qu'il n'y a plus
        // rien a jouer. Seul l'appelant le sait.
        let underruns = starvation.clone();

        device
            .build_output_stream(
                *config,
                move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if consumer.read(out) < out.len() {
                        underruns.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                },
                |e: cpal::Error| eprintln!("audio: {e}"),
                None,
            )
            .map_err(OutputError::Build)
    }

    /// An `Audio` matching this device's sample rate.
    ///
    /// Un melange a 44,1 kHz envoye a une sortie a 48 kHz jouerait trop vite :
    /// mieux vaut que le peripherique decide.
    #[must_use]
    pub fn audio(&self) -> Audio {
        Audio::new(self.sample_rate)
    }

    /// Mixes and pushes as much as the ring will take.
    ///
    /// A appeler a chaque frame : le tampon se vide en continu, et ce qui ne
    /// tient pas attend la frame suivante.
    pub fn pump(&mut self, audio: &mut Audio, scratch: &mut Vec<f32>) {
        let free = self.producer.free();
        if free == 0 {
            return;
        }

        // Toujours un nombre pair : une frame stereo ne se coupe pas en deux.
        let wanted = free & !1;
        if wanted == 0 {
            return;
        }

        if scratch.len() < wanted {
            scratch.resize(wanted, 0.0);
        }

        audio.render(&mut scratch[..wanted]);
        self.producer.write(&scratch[..wanted]);
    }

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// How many times the callback ran out of samples.
    ///
    /// Chacune est un craquement, sauf celles qui suivent le dernier son : le
    /// silence apres la fin est voulu.
    #[must_use]
    pub fn starvation(&self) -> u32 {
        self.starvation.load(Ordering::Relaxed)
    }

    /// How many samples are waiting to be played.
    #[must_use]
    pub fn queued(&self) -> usize {
        self.producer.filled()
    }

    /// # Errors
    ///
    /// If the device refuses.
    pub fn pause(&self) -> Result<(), OutputError> {
        self.stream.pause().map_err(OutputError::Play)
    }

    /// # Errors
    ///
    /// If the device refuses.
    pub fn resume(&self) -> Result<(), OutputError> {
        self.stream.play().map_err(OutputError::Play)
    }
}
