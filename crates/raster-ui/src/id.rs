use std::hash::{Hash, Hasher};

/// A widget's identity, stable across frames.
///
/// Derivee d'un nom plutot que d'une position : un bouton qui se deplace garde
/// son focus et son etat enfonce, ce qu'un identifiant positionnel perdrait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(u64);

impl Id {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(hash(name.as_bytes(), 0))
    }

    /// Un identifiant sous un autre : deux boutons "Supprimer" dans deux
    /// panneaux differents ne doivent pas se confondre.
    #[must_use]
    pub fn child(self, name: &str) -> Self {
        Self(hash(name.as_bytes(), self.0))
    }

    /// Pour une liste, ou les elements n'ont pas de nom propre.
    #[must_use]
    pub fn index(self, index: usize) -> Self {
        Self(hash(&index.to_le_bytes(), self.0))
    }

    #[must_use]
    pub fn raw(self) -> u64 {
        self.0
    }
}

/// FNV-1a : court, stable d'une execution a l'autre, et sans dependance.
///
/// La stabilite compte : un identifiant qui change entre deux lancements
/// perdrait l'etat sauvegarde d'un panneau.
fn hash(bytes: &[u8], seed: u64) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut h = if seed == 0 { OFFSET } else { seed };
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    }
    h
}

impl From<&str> for Id {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:016x}", self.0)
    }
}

/// Hashes anything hashable into an id, for keys a name cannot express.
#[must_use]
pub fn id_of<T: Hash>(value: &T) -> Id {
    struct Fnv(u64);

    impl Hasher for Fnv {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0 = hash(bytes, self.0);
        }
    }

    let mut hasher = Fnv(0);
    value.hash(&mut hasher);
    Id(hasher.finish())
}
