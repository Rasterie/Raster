use super::AssetId;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// A loaded asset, reachable in constant time.
///
/// Un indice reste valide pour toujours : rien n'est retire d'un store, ce qui
/// permet a un rechargement a chaud de remplacer un asset sous les handles qui
/// le designent.
pub struct Handle<T> {
    index: u32,
    marker: std::marker::PhantomData<fn() -> T>,
}

// Ecrits a la main : un `derive` exigerait `T: Copy` alors qu'un handle ne
// contient qu'un entier, jamais un `T`.
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({})", self.index)
    }
}

impl<T> Handle<T> {
    fn new(index: u32) -> Self {
        Self {
            index,
            marker: std::marker::PhantomData,
        }
    }

    /// L'indice brut, pour un backend qui indexe ses ressources par entier.
    #[must_use]
    pub fn index(self) -> u32 {
        self.index
    }
}

/// Assets of one kind, loaded once and reachable by handle.
///
/// Le chargement est fourni par l'appelant : `raster-core` ne sait pas lire un
/// PNG et ne doit pas dependre de ce qui le sait.
pub struct AssetStore<T> {
    items: Vec<T>,
    /// Ce qui rend le chargement idempotent : deux acteurs qui nomment la meme
    /// texture partagent la meme.
    by_id: HashMap<AssetId, u32>,
    ids: Vec<AssetId>,
}

impl<T> AssetStore<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            by_id: HashMap::new(),
            ids: Vec::new(),
        }
    }

    /// Returns the handle for `id`, calling `load` only the first time.
    ///
    /// # Errors
    ///
    /// Whatever `load` returns. Un echec ne laisse aucune trace dans le cache :
    /// un asset absent puis ajoute se charge au prochain essai.
    pub fn load_with<E>(
        &mut self,
        id: &AssetId,
        load: impl FnOnce(&AssetId) -> Result<T, E>,
    ) -> Result<Handle<T>, E> {
        if let Some(&index) = self.by_id.get(id) {
            return Ok(Handle::new(index));
        }

        let item = load(id)?;
        Ok(self.insert(id.clone(), item))
    }

    /// Stores an already-loaded asset under `id`, replacing any earlier one.
    ///
    /// C'est par la que passe le rechargement a chaud : les handles existants
    /// continuent de pointer vers le meme indice, qui porte le nouvel asset.
    pub fn insert(&mut self, id: AssetId, item: T) -> Handle<T> {
        match self.by_id.entry(id.clone()) {
            Entry::Occupied(slot) => {
                let index = *slot.get();
                self.items[index as usize] = item;
                Handle::new(index)
            }
            Entry::Vacant(slot) => {
                let index = u32::try_from(self.items.len())
                    .expect("an asset store cannot hold more than u32::MAX items");
                self.items.push(item);
                self.ids.push(id);
                slot.insert(index);
                Handle::new(index)
            }
        }
    }

    #[must_use]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.items.get(handle.index as usize)
    }

    #[must_use]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.items.get_mut(handle.index as usize)
    }

    #[must_use]
    pub fn handle(&self, id: &AssetId) -> Option<Handle<T>> {
        self.by_id.get(id).copied().map(Handle::new)
    }

    #[must_use]
    pub fn by_id(&self, id: &AssetId) -> Option<&T> {
        self.handle(id).and_then(|h| self.get(h))
    }

    #[must_use]
    pub fn contains(&self, id: &AssetId) -> bool {
        self.by_id.contains_key(id)
    }

    /// The identifier an asset was loaded from.
    #[must_use]
    pub fn id_of(&self, handle: Handle<T>) -> Option<&AssetId> {
        self.ids.get(handle.index as usize)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Les assets dans l'ordre de leurs indices, pour un backend qui indexe
    /// une tranche par `Handle::index`.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AssetId, &T)> {
        self.ids.iter().zip(&self.items)
    }
}

impl<T> Default for AssetStore<T> {
    fn default() -> Self {
        Self::new()
    }
}
