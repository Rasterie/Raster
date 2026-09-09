use std::fmt;
use std::num::NonZeroU32;

/// Identifies one actor in the world.
///
/// Jamais de reference Rust : les acteurs se referencent librement, un script
/// ne peut pas en tenir une, et une scene stocke des identifiants.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActorId {
    index: u32,
    /// Distinguishes an actor from whatever occupies its slot later.
    generation: NonZeroU32,
    /// Which pool holds this actor — see decision 013.
    type_tag: u16,
}

impl ActorId {
    #[inline]
    #[must_use]
    pub(crate) const fn new(index: u32, generation: NonZeroU32, type_tag: u16) -> Self {
        Self {
            index,
            generation,
            type_tag,
        }
    }

    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    #[inline]
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation.get()
    }

    #[inline]
    #[must_use]
    pub const fn type_tag(self) -> u16 {
        self.type_tag
    }
}

impl fmt::Debug for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Actor({}#{}@{})",
            self.index, self.generation, self.type_tag
        )
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Actor({})", self.index)
    }
}

/// Distribue les emplacements et suit leur generation : reutiliser un
/// emplacement l'incremente, ce qui invalide les anciens identifiants.
#[derive(Debug, Default)]
pub(crate) struct Generations {
    /// Impair tant que l'emplacement est occupe, pair une fois libre : la
    /// parite evite un second tableau pour savoir qui est vivant.
    slots: Vec<u32>,
    free: Vec<u32>,
}

impl Generations {
    /// Claims a slot, reusing a freed one when possible.
    pub(crate) fn allocate(&mut self) -> (u32, NonZeroU32) {
        if let Some(index) = self.free.pop() {
            let generation = &mut self.slots[index as usize];
            // Un emplacement libre est pair : +1 donne un impair non nul.
            *generation += 1;
            let nz = NonZeroU32::new(*generation).expect("a freed slot is even, so +1 is non-zero");
            return (index, nz);
        }

        let index = u32::try_from(self.slots.len()).expect("more than 4 billion actor slots");
        self.slots.push(1);
        (index, NonZeroU32::new(1).expect("literal one is non-zero"))
    }

    /// Releases a slot, invalidating every id that pointed at it. Un double
    /// appel est sans effet plutot qu'une panique.
    pub(crate) fn free(&mut self, index: u32, generation: NonZeroU32) -> bool {
        let Some(current) = self.slots.get_mut(index as usize) else {
            return false;
        };
        if *current != generation.get() || *current % 2 == 0 {
            return false;
        }

        // Au debordement, l'emplacement est retire de la circulation : un
        // ancien identifiant redeviendrait valide s'il rebouclait.
        let Some(next) = current.checked_add(1) else {
            return true;
        };

        *current = next;
        self.free.push(index);
        true
    }

    /// Whether an id still refers to a live slot.
    #[must_use]
    pub(crate) fn is_live(&self, index: u32, generation: NonZeroU32) -> bool {
        self.slots.get(index as usize) == Some(&generation.get()) && generation.get() % 2 == 1
    }

    /// The generation of one slot, or `None` if free or absent.
    #[must_use]
    pub(crate) fn generation_of(&self, index: u32) -> Option<u32> {
        let generation = *self.slots.get(index as usize)?;
        (generation % 2 == 1).then_some(generation)
    }

    /// Les generations brutes, pour les parcourir en empruntant les acteurs.
    #[must_use]
    pub(crate) fn raw_slots(&self) -> &[u32] {
        &self.slots
    }

    /// How many slots are occupied. Compte plutot que soustrait : un
    /// emplacement retire au debordement n'est dans aucune des deux listes.
    #[must_use]
    pub(crate) fn live_count(&self) -> usize {
        self.slots.iter().filter(|g| *g % 2 == 1).count()
    }

    /// Frees every live slot, without resetting the counters.
    ///
    /// Les remettre a zero ferait redevenir valides les identifiants d'avant :
    /// un changement de niveau ressusciterait les references de l'ancien.
    pub(crate) fn clear(&mut self) {
        self.free.clear();

        for (index, generation) in self.slots.iter_mut().enumerate() {
            let index = u32::try_from(index).expect("the index came from a u32-sized slot list");

            if *generation % 2 == 0 {
                // Deja libre : il suffit de le remettre dans la liste.
                self.free.push(index);
            } else if let Some(next) = generation.checked_add(1) {
                *generation = next;
                self.free.push(index);
            }
            // Sinon l'emplacement est epuise : retire de la circulation, comme
            // dans `free`.
        }
    }
}
