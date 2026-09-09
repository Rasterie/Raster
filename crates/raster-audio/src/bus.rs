/// Where a sound is routed, and what the player can turn down.
///
/// Un ensemble fixe : un jeu regle des volumes, il n'invente pas de categories
/// a l'execution. Une option de menu se branche directement dessus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Bus {
    #[default]
    Sfx,
    Music,
    Ui,
    Ambience,
}

impl Bus {
    pub const ALL: [Bus; 4] = [Bus::Sfx, Bus::Music, Bus::Ui, Bus::Ambience];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Sfx => "sfx",
            Self::Music => "music",
            Self::Ui => "ui",
            Self::Ambience => "ambience",
        }
    }
}

impl std::fmt::Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The volume of each bus, and of the master.
#[derive(Debug, Clone, Copy)]
pub struct Mixer {
    master: f32,
    volumes: [f32; 4],
}

impl Mixer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            master: 1.0,
            volumes: [1.0; 4],
        }
    }

    /// Le gain applique a un son : celui du bus multiplie par le master.
    #[must_use]
    pub fn gain(&self, bus: Bus) -> f32 {
        self.master * self.volumes[bus as usize]
    }

    /// Sets a bus volume, clamped to a sane range.
    ///
    /// Bornee : un volume negatif inverserait la phase, et au-dela de 2 la
    /// sortie sature en craquant.
    pub fn set(&mut self, bus: Bus, volume: f32) {
        self.volumes[bus as usize] = volume.clamp(0.0, 2.0);
    }

    #[must_use]
    pub fn volume(&self, bus: Bus) -> f32 {
        self.volumes[bus as usize]
    }

    pub fn set_master(&mut self, volume: f32) {
        self.master = volume.clamp(0.0, 2.0);
    }

    #[must_use]
    pub fn master(&self) -> f32 {
        self.master
    }

    /// Silences everything without losing the volumes underneath.
    pub fn mute(&mut self) {
        self.master = 0.0;
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}
