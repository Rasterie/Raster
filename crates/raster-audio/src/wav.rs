use crate::Sound;

/// Why a WAV could not be read.
#[derive(Debug)]
pub enum WavError {
    /// Not a RIFF/WAVE file at all.
    NotWav,
    Truncated,
    /// A format the decoder does not handle, named so the message is useful.
    Unsupported(String),
    /// No `data` chunk was found.
    NoData,
}

impl std::fmt::Display for WavError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotWav => write!(f, "not a WAV file"),
            Self::Truncated => write!(f, "the WAV file ends mid-way"),
            Self::Unsupported(what) => write!(f, "unsupported WAV: {what}"),
            Self::NoData => write!(f, "the WAV file has no data chunk"),
        }
    }
}

impl std::error::Error for WavError {}

/// Decodes a WAV file.
///
/// Ecrit ici plutot que pris d'une bibliotheque : `resonance-core` ecrit des
/// WAV mais n'en lit pas, et le format tient en cent lignes.
///
/// # Errors
///
/// If the bytes are not a WAV, or hold a format this decoder does not handle.
pub fn decode(bytes: &[u8]) -> Result<Sound, WavError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavError::NotWav);
    }

    let mut format = None;
    let mut data = None;
    let mut cursor = 12;

    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size = u32::from_le_bytes(read4(bytes, cursor + 4)?) as usize;
        let start = cursor + 8;
        let end = start.checked_add(size).ok_or(WavError::Truncated)?;

        if end > bytes.len() {
            return Err(WavError::Truncated);
        }

        match id {
            b"fmt " => format = Some(Format::parse(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }

        // Les morceaux sont alignes sur deux octets : un morceau de taille
        // impaire est suivi d'un octet de remplissage.
        cursor = end + (size & 1);
    }

    let format = format.ok_or(WavError::Unsupported("no fmt chunk".to_owned()))?;
    let data = data.ok_or(WavError::NoData)?;

    let samples = format.decode(data)?;
    Ok(Sound::new(samples, format.channels, format.sample_rate))
}

struct Format {
    channels: u16,
    sample_rate: u32,
    bits: u16,
    float: bool,
}

impl Format {
    fn parse(chunk: &[u8]) -> Result<Self, WavError> {
        if chunk.len() < 16 {
            return Err(WavError::Truncated);
        }

        let tag = u16::from_le_bytes([chunk[0], chunk[1]]);
        let channels = u16::from_le_bytes([chunk[2], chunk[3]]);
        let sample_rate = u32::from_le_bytes(read4(chunk, 4)?);
        let bits = u16::from_le_bytes([chunk[14], chunk[15]]);

        // 0xFFFE est l'extension : le vrai format est dans le sous-format, dont
        // les deux premiers octets suivent la meme convention que `tag`.
        let tag = if tag == 0xFFFE && chunk.len() >= 26 {
            u16::from_le_bytes([chunk[24], chunk[25]])
        } else {
            tag
        };

        let float = match tag {
            1 => false,
            3 => true,
            other => return Err(WavError::Unsupported(format!("format tag {other}"))),
        };

        if channels != 1 && channels != 2 {
            return Err(WavError::Unsupported(format!("{channels} channels")));
        }
        if sample_rate == 0 {
            return Err(WavError::Unsupported("a sample rate of 0".to_owned()));
        }

        Ok(Self {
            channels,
            sample_rate,
            bits,
            float,
        })
    }

    /// Ramene tout en `f32` entre -1 et 1, la forme que le melange attend.
    fn decode(&self, data: &[u8]) -> Result<Vec<f32>, WavError> {
        let samples = match (self.float, self.bits) {
            (false, 8) => data
                .iter()
                // Le 8 bits est le seul format WAV non signe : 128 est le zero.
                .map(|&b| (f32::from(b) - 128.0) / 128.0)
                .collect(),
            (false, 16) => data
                .chunks_exact(2)
                .map(|c| f32::from(i16::from_le_bytes([c[0], c[1]])) / 32768.0)
                .collect(),
            (false, 24) => data
                .chunks_exact(3)
                .map(|c| {
                    let v = i32::from_le_bytes([0, c[0], c[1], c[2]]) >> 8;
                    v as f32 / 8_388_608.0
                })
                .collect(),
            (false, 32) => data
                .chunks_exact(4)
                .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f32 / 2_147_483_648.0)
                .collect(),
            (true, 32) => data
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
            (true, 64) => data
                .chunks_exact(8)
                .map(|c| {
                    f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]) as f32
                })
                .collect(),
            (float, bits) => {
                let kind = if float { "float" } else { "integer" };
                return Err(WavError::Unsupported(format!("{bits}-bit {kind}")));
            }
        };

        Ok(trim(samples, self.channels))
    }
}

/// Ecarte une frame incomplete : un fichier tronque au milieu d'une paire
/// stereo decalerait tous les canaux.
fn trim(mut samples: Vec<f32>, channels: u16) -> Vec<f32> {
    let extra = samples.len() % channels as usize;
    samples.truncate(samples.len() - extra);
    samples
}

fn read4(bytes: &[u8], at: usize) -> Result<[u8; 4], WavError> {
    bytes
        .get(at..at + 4)
        .and_then(|s| s.try_into().ok())
        .ok_or(WavError::Truncated)
}
