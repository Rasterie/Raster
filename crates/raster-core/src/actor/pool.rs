use super::{ActorId, Generations};
use std::num::NonZeroU32;

/// Storage for every actor of one concrete type.
///
/// Keeping each type in its own dense `Vec` is what makes iterating one type
/// fast — 9.4× faster than a boxed arena at 25 000 actors, which is the
/// measurement decision 013 turned on.
#[derive(Debug)]
pub(crate) struct Pool<T> {
    slots: Vec<Option<T>>,
    generations: Generations,
    tag: u16,
}

impl<T> Pool<T> {
    #[must_use]
    pub(crate) fn new(tag: u16) -> Self {
        Self {
            slots: Vec::new(),
            generations: Generations::default(),
            tag,
        }
    }

    pub(crate) fn spawn(&mut self, actor: T) -> ActorId {
        let (index, generation) = self.generations.allocate();
        let i = index as usize;

        if i < self.slots.len() {
            self.slots[i] = Some(actor);
        } else {
            debug_assert_eq!(i, self.slots.len(), "the allocator skipped a slot");
            self.slots.push(Some(actor));
        }

        ActorId::new(index, generation, self.tag)
    }

    /// Removes an actor, returning it so a caller can run teardown on it.
    ///
    /// Returns `None` for an id that is already dead: despawning twice is a
    /// normal race in gameplay code, not a programming error.
    pub(crate) fn despawn(&mut self, id: ActorId) -> Option<T> {
        let generation = NonZeroU32::new(id.generation())?;
        if !self.generations.free(id.index(), generation) {
            return None;
        }
        self.slots.get_mut(id.index() as usize)?.take()
    }

    #[must_use]
    pub(crate) fn get(&self, id: ActorId) -> Option<&T> {
        self.slot_index(id).and_then(|i| self.slots[i].as_ref())
    }

    #[must_use]
    pub(crate) fn get_mut(&mut self, id: ActorId) -> Option<&mut T> {
        self.slot_index(id).and_then(|i| self.slots[i].as_mut())
    }

    /// Resolves an id to a slot index, checking the generation.
    ///
    /// This is what makes a stale id safe: it resolves to `None` rather than to
    /// whichever actor now occupies the slot.
    #[inline]
    fn slot_index(&self, id: ActorId) -> Option<usize> {
        if id.type_tag() != self.tag {
            return None;
        }
        let generation = NonZeroU32::new(id.generation())?;
        if !self.generations.is_live(id.index(), generation) {
            return None;
        }
        let i = id.index() as usize;
        (i < self.slots.len()).then_some(i)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (ActorId, &T)> {
        let tag = self.tag;
        self.slots.iter().enumerate().filter_map(move |(i, slot)| {
            let actor = slot.as_ref()?;
            let index = u32::try_from(i).ok()?;
            let generation = NonZeroU32::new(self.generations.generation_of(index)?)?;
            Some((ActorId::new(index, generation, tag), actor))
        })
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (ActorId, &mut T)> {
        let tag = self.tag;
        /*
          Les generations sont empruntees en lecture et les emplacements en
          ecriture. Les deux vivent dans des champs distincts de la struct, ce
          que le compilateur accepte si on les separe avant la fermeture — sans
          quoi il verrait un emprunt de `self` entier.
        */
        let generations = self.generations.raw_slots();

        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(move |(i, slot)| {
                let actor = slot.as_mut()?;
                let index = u32::try_from(i).ok()?;
                let generation = NonZeroU32::new(*generations.get(i)?)?;
                Some((ActorId::new(index, generation, tag), actor))
            })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    pub(crate) fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(Option::as_mut)
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.generations.live_count()
    }

    pub(crate) fn clear(&mut self) {
        self.slots.clear();
        self.generations.clear();
    }
}
