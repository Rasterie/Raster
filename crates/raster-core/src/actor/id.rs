use std::fmt;
use std::num::NonZeroU32;

/// Identifies one actor in the world.
///
/// Actors are addressed by id and never by Rust reference. Three reasons, and
/// only one is about the borrow checker: actors reference each other freely, a
/// script cannot hold a Rust reference across a call boundary, and a scene file
/// stores ids rather than pointers.
///
/// Eight bytes, and `Option<ActorId>` is eight too — the generation is
/// non-zero, which leaves a niche for `None`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActorId {
    index: u32,
    /// Distinguishes an actor from whatever occupies its slot later.
    ///
    /// Non-zero so `Option<ActorId>` costs nothing: the compiler uses zero to
    /// mean `None`.
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

/// Hands out slot indices and tracks which generation each slot is on.
///
/// The generation is what makes a stale id safe: reusing a slot bumps it, so an
/// id kept from before resolves to nothing rather than to whichever actor
/// happens to occupy the slot now. Without it, an enemy holding the id of a
/// dead player would silently start targeting a newly spawned crate.
#[derive(Debug, Default)]
pub(crate) struct Generations {
    /// The current generation of each slot. Always odd while the slot is
    /// occupied and even while it is free, so liveness needs no second array.
    slots: Vec<u32>,
    free: Vec<u32>,
}

impl Generations {
    /// Claims a slot, reusing a freed one when possible.
    pub(crate) fn allocate(&mut self) -> (u32, NonZeroU32) {
        if let Some(index) = self.free.pop() {
            let generation = &mut self.slots[index as usize];
            // Un emplacement libre porte une generation paire, donc +1 la rend
            // impaire et jamais nulle : le debordement est traite dans `free`.
            *generation += 1;
            let nz = NonZeroU32::new(*generation).expect("a freed slot is even, so +1 is non-zero");
            return (index, nz);
        }

        let index = u32::try_from(self.slots.len()).expect("more than 4 billion actor slots");
        self.slots.push(1);
        (index, NonZeroU32::new(1).expect("literal one is non-zero"))
    }

    /// Releases a slot, invalidating every id that pointed at it.
    ///
    /// Returns whether the slot was live; a double free is a no-op rather than
    /// a panic, since despawning an already-dead actor is a normal race in
    /// gameplay code.
    pub(crate) fn free(&mut self, index: u32, generation: NonZeroU32) -> bool {
        let Some(current) = self.slots.get_mut(index as usize) else {
            return false;
        };
        if *current != generation.get() || *current % 2 == 0 {
            return false;
        }

        /*
          Au bout de 2^31 reutilisations, le compteur deborderait et un ancien
          identifiant redeviendrait valide — un acteur mort se remettrait a
          repondre. Plutot que de reboucler, on retire l'emplacement de la
          circulation : il reste marque occupe et n'est jamais recycle. Perdre
          un emplacement sur quatre milliards est preferable a une confusion
          d'identite silencieuse.
        */
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

    /// The generation of one slot, or `None` if the slot is free or absent.
    #[must_use]
    pub(crate) fn generation_of(&self, index: u32) -> Option<u32> {
        let generation = *self.slots.get(index as usize)?;
        (generation % 2 == 1).then_some(generation)
    }

    /// Every slot's generation, for iterators that need to walk them alongside
    /// a mutable borrow of the actors.
    #[must_use]
    pub(crate) fn raw_slots(&self) -> &[u32] {
        &self.slots
    }

    /// How many slots are occupied.
    ///
    /// Counts rather than subtracting the free list, because a slot retired
    /// after generation overflow is in neither set.
    #[must_use]
    pub(crate) fn live_count(&self) -> usize {
        self.slots.iter().filter(|g| *g % 2 == 1).count()
    }

    /// Frees every live slot, without resetting the counters.
    ///
    /// Remettre les generations a zero ferait redevenir valides les
    /// identifiants d'avant l'effacement, qui designeraient alors d'autres
    /// acteurs : un changement de niveau ressusciterait les references de
    /// l'ancien. Les compteurs ne redescendent jamais.
    pub(crate) fn clear(&mut self) {
        self.free.clear();

        for (index, generation) in self.slots.iter_mut().enumerate() {
            if *generation % 2 == 1 {
                match generation.checked_add(1) {
                    Some(next) => {
                        *generation = next;
                        self.free.push(
                            u32::try_from(index).expect("index fits, it came from a u32 slot"),
                        );
                    }
                    // Emplacement epuise : retire de la circulation, comme dans `free`.
                    None => {}
                }
            } else {
                self.free
                    .push(u32::try_from(index).expect("index fits, it came from a u32 slot"));
            }
        }
    }
}
