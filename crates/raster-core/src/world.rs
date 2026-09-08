use crate::actor::{Actor, ActorId, Component, HasComponent, Pool};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// Every actor in the game, and the only path to mutate one.
///
/// A flat collection: there is no root and no scene tree. An actor exists in
/// the world, and a hierarchy is something you opt into.
///
/// Storage is one pool per actor type — see decision 013 — reached through a
/// map from `TypeId` to a type-erased pool. That indirection is the cost of the
/// design, so it is kept to a single hash lookup and a downcast.
#[derive(Default)]
pub struct World {
    pools: HashMap<TypeId, ErasedPool>,
    /*
      Les tags sont attribues sequentiellement a partir de zero, donc un Vec
      indexe par le tag remplace une seconde HashMap sur le chemin de `despawn`
      et de `contains` — les deux operations qui partent d'un identifiant sans
      connaitre le type.
    */
    tags: Vec<TypeId>,
}

/// A `Pool<T>` with its type forgotten, plus the operations the world needs to
/// perform without knowing `T`.
struct ErasedPool {
    pool: Box<dyn Any>,
    /// Removes an actor without the caller knowing its type — what `despawn`
    /// needs when all it has is an id.
    despawn: fn(&mut dyn Any, ActorId) -> bool,
    len: fn(&dyn Any) -> usize,
    clear: fn(&mut dyn Any),
    /// Whether an id is still live, without the caller knowing the type.
    contains: fn(&dyn Any, ActorId) -> bool,
    /*
      Une fonction d'iteration par type de composant present dans ce type
      d'acteur. C'est ce qui permet au renderer de parcourir tous les Sprite
      sans connaitre Player : la fonction est enregistree quand le type est
      declare, et le rappel evite d'allouer un Vec par frame.
    */
    components: HashMap<TypeId, ComponentIter>,
}

/// Applique `f` a chaque composant du type voulu detenu par un pool.
///
/// Le composant est passe en `&dyn Any` faute de pouvoir nommer son type ici ;
/// l'appelant le retrouve par downcast, qui est resolu statiquement.
type ComponentIter = fn(&dyn Any, &mut dyn FnMut(ActorId, &dyn Any));

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an actor and returns its id.
    ///
    /// The actor's `on_spawn` hook is not called here: `spawn` knows nothing
    /// about `Behaviour`, which an actor need not implement. Worlds driven by
    /// the frame loop use [`World::spawn_with_hook`].
    pub fn spawn<T: Actor>(&mut self, actor: T) -> ActorId {
        self.pool_mut::<T>().spawn(actor)
    }

    /// Adds an actor and runs its `on_spawn` hook.
    pub fn spawn_with_hook<T: crate::actor::Behaviour>(&mut self, mut actor: T) -> ActorId {
        actor.on_spawn();
        self.spawn(actor)
    }

    /// Removes an actor, returning whether it was there.
    ///
    /// Despawning an already-dead actor is a no-op rather than an error: two
    /// systems deciding to destroy the same enemy in one frame is normal.
    pub fn despawn(&mut self, id: ActorId) -> bool {
        let Some(type_id) = self.tags.get(id.type_tag() as usize).copied() else {
            return false;
        };
        let Some(erased) = self.pools.get_mut(&type_id) else {
            return false;
        };
        (erased.despawn)(erased.pool.as_mut(), id)
    }

    /// Removes an actor, running its `on_despawn` hook first.
    pub fn despawn_with_hook<T: crate::actor::Behaviour>(&mut self, id: ActorId) -> bool {
        let Some(pool) = self.pool_opt_mut::<T>() else {
            return false;
        };
        match pool.despawn(id) {
            Some(mut actor) => {
                actor.on_despawn();
                true
            }
            None => false,
        }
    }

    #[must_use]
    pub fn get<T: Actor>(&self, id: ActorId) -> Option<&T> {
        self.pool_opt::<T>()?.get(id)
    }

    #[must_use]
    pub fn get_mut<T: Actor>(&mut self, id: ActorId) -> Option<&mut T> {
        self.pool_opt_mut::<T>()?.get_mut(id)
    }

    /// Whether an id still refers to a live actor.
    ///
    /// Does not require knowing the actor's type, which is what makes it usable
    /// on an id received from elsewhere.
    #[must_use]
    pub fn contains(&self, id: ActorId) -> bool {
        let Some(type_id) = self.tags.get(id.type_tag() as usize) else {
            return false;
        };
        let Some(erased) = self.pools.get(type_id) else {
            return false;
        };
        (erased.contains)(erased.pool.as_ref(), id)
    }

    /// Every actor of one type, with its id.
    ///
    /// This is the fast path the storage model was chosen for: the actors of
    /// one type are contiguous.
    pub fn iter<T: Actor>(&self) -> impl Iterator<Item = (ActorId, &T)> {
        self.pool_opt::<T>().into_iter().flat_map(Pool::iter)
    }

    pub fn iter_mut<T: Actor>(&mut self) -> impl Iterator<Item = (ActorId, &mut T)> {
        self.pool_opt_mut::<T>()
            .into_iter()
            .flat_map(Pool::iter_mut)
    }

    /// Every actor of one type, without its id.
    pub fn values<T: Actor>(&self) -> impl Iterator<Item = &T> {
        self.pool_opt::<T>().into_iter().flat_map(Pool::values)
    }

    pub fn values_mut<T: Actor>(&mut self) -> impl Iterator<Item = &mut T> {
        self.pool_opt_mut::<T>()
            .into_iter()
            .flat_map(Pool::values_mut)
    }

    /// Runs `tick` on every actor of one type.
    pub fn tick<T: crate::actor::Behaviour>(&mut self, dt: f32) {
        for actor in self.values_mut::<T>() {
            actor.tick(dt);
        }
    }

    /// Runs `fixed_tick` on every actor of one type.
    pub fn fixed_tick<T: crate::actor::Behaviour>(&mut self, dt: f32) {
        for actor in self.values_mut::<T>() {
            actor.fixed_tick(dt);
        }
    }

    /// How many actors of one type are alive.
    #[must_use]
    pub fn count<T: Actor>(&self) -> usize {
        self.pool_opt::<T>().map_or(0, Pool::len)
    }

    /// How many actors are alive, all types together.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pools.values().map(|e| (e.len)(e.pool.as_ref())).sum()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes every actor, keeping the pools so ids stay routable.
    pub fn clear(&mut self) {
        for erased in self.pools.values_mut() {
            (erased.clear)(erased.pool.as_mut());
        }
    }

    /// Declares that actors of type `T` hold components of type `C`.
    ///
    /// Registration is explicit for the same reason type registration is — see
    /// decision 011. Without it, [`World::each_component`] would silently skip
    /// this actor type, which is worse than a compile-time chore.
    pub fn register_component<T, C>(&mut self)
    where
        T: Actor + HasComponent<C>,
        C: Component,
    {
        let iterate: ComponentIter = |pool, f| {
            let Some(pool) = pool.downcast_ref::<Pool<T>>() else {
                return;
            };
            for (id, actor) in pool.iter() {
                for component in actor.components() {
                    f(id, component);
                }
            }
        };

        // Force la creation du pool : un type declare mais jamais peuple doit
        // quand meme etre connu, sinon l'enregistrement serait perdu.
        let _ = self.pool_mut::<T>();

        if let Some(erased) = self.pools.get_mut(&TypeId::of::<T>()) {
            erased.components.insert(TypeId::of::<C>(), iterate);
        }
    }

    /// Applies `f` to every component of type `C` in the world, whatever actor
    /// holds it.
    ///
    /// This is the renderer's path to every `Sprite`. Takes a callback rather
    /// than returning an iterator: the components live in different pools of
    /// different types, and collecting them would allocate once per frame.
    pub fn each_component<C: Component>(&self, mut f: impl FnMut(ActorId, &C)) {
        let component_id = TypeId::of::<C>();

        for erased in self.pools.values() {
            let Some(iterate) = erased.components.get(&component_id) else {
                continue;
            };
            iterate(erased.pool.as_ref(), &mut |id, component| {
                if let Some(component) = component.downcast_ref::<C>() {
                    f(id, component);
                }
            });
        }
    }

    /// How many actor types have been seen. Mostly of interest to tests.
    #[must_use]
    pub fn type_count(&self) -> usize {
        self.pools.len()
    }

    // --- internals ---------------------------------------------------------

    /// The pool for `T`, creating it on first use.
    fn pool_mut<T: Actor>(&mut self) -> &mut Pool<T> {
        let type_id = TypeId::of::<T>();

        // `entry` plutot que contains_key + insert : une seule recherche, et
        // `spawn` est appele en boucle de jeu.
        if let Entry::Vacant(slot) = self.pools.entry(type_id) {
            let tag =
                u16::try_from(self.tags.len()).expect("more than 65 535 actor types in one world");
            self.tags.push(type_id);

            slot.insert(ErasedPool {
                pool: Box::new(Pool::<T>::new(tag)),
                despawn: |pool, id| {
                    pool.downcast_mut::<Pool<T>>()
                        .is_some_and(|p| p.despawn(id).is_some())
                },
                len: |pool| pool.downcast_ref::<Pool<T>>().map_or(0, Pool::len),
                contains: |pool, id| {
                    pool.downcast_ref::<Pool<T>>()
                        .is_some_and(|p| p.get(id).is_some())
                },
                clear: |pool| {
                    if let Some(p) = pool.downcast_mut::<Pool<T>>() {
                        p.clear();
                    }
                },
                components: HashMap::new(),
            });
        }

        self.pools
            .get_mut(&type_id)
            .and_then(|e| e.pool.downcast_mut::<Pool<T>>())
            .expect("the pool was just inserted with this exact type")
    }

    /// The pool for `T` if any actor of that type was ever spawned.
    fn pool_opt<T: Actor>(&self) -> Option<&Pool<T>> {
        self.pools
            .get(&TypeId::of::<T>())?
            .pool
            .downcast_ref::<Pool<T>>()
    }

    fn pool_opt_mut<T: Actor>(&mut self) -> Option<&mut Pool<T>> {
        self.pools
            .get_mut(&TypeId::of::<T>())?
            .pool
            .downcast_mut::<Pool<T>>()
    }
}
