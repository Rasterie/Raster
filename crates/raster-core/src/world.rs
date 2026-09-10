use crate::actor::{Actor, ActorId, Component, HasComponent, Pool};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// Every actor in the game, and the only path to mutate one. Une collection
/// plate, un pool par type d'acteur — voir la decision 013.
#[derive(Default)]
pub struct World {
    pools: HashMap<TypeId, ErasedPool>,
    /// Tag -> type. Un Vec indexe plutot qu'une carte : les tags sont
    /// sequentiels, et cela evite un hachage sur le chemin de `despawn`.
    tags: Vec<TypeId>,
}

/// A `Pool<T>` with its type forgotten, plus the operations the world needs to
/// perform without knowing `T`.
struct ErasedPool {
    pool: Box<dyn Any>,
    despawn: fn(&mut dyn Any, ActorId) -> bool,
    len: fn(&dyn Any) -> usize,
    clear: fn(&mut dyn Any),
    contains: fn(&dyn Any, ActorId) -> bool,
    /// L'acteur vu par la reflexion : ce qui permet a un inspecteur de le
    /// decrire et de l'ecrire sans connaitre son type.
    reflect: fn(&dyn Any, ActorId) -> Option<&dyn crate::reflect::ReflectObject>,
    reflect_mut: fn(&mut dyn Any, ActorId) -> Option<&mut dyn crate::reflect::ReflectObject>,
    /// Une iteration par type de composant : c'est ce qui laisse le renderer
    /// parcourir tous les `Sprite` sans connaitre `Player`.
    components: HashMap<TypeId, ComponentIter>,
}

/// Applique `f` a chaque composant du type voulu detenu par un pool.
type ComponentIter = fn(&dyn Any, &mut dyn FnMut(ActorId, &dyn Any));

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an actor and returns its id.
    ///
    /// Does not run `on_spawn` — see [`World::spawn_with_hook`] for that.
    pub fn spawn<T: Actor>(&mut self, actor: T) -> ActorId {
        self.pool_mut::<T>().spawn(actor)
    }

    /// Adds an actor and runs its `on_spawn` hook.
    pub fn spawn_with_hook<T: crate::actor::Behaviour>(&mut self, mut actor: T) -> ActorId {
        actor.on_spawn();
        self.spawn(actor)
    }

    /// Removes an actor, returning whether it was there. Despawning twice is a
    /// no-op: two systems destroying the same enemy in one frame is normal.
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

    /// An actor seen through reflection, whatever its type.
    ///
    /// Ce dont un inspecteur a besoin : decrire un acteur selectionne sans
    /// savoir ce qu'il est.
    #[must_use]
    pub fn reflect(&self, id: ActorId) -> Option<&dyn crate::reflect::ReflectObject> {
        let tag = self.tags.get(id.type_tag() as usize)?;
        let pool = self.pools.get(tag)?;
        (pool.reflect)(pool.pool.as_ref(), id)
    }

    /// The same, to write a field back.
    #[must_use]
    pub fn reflect_mut(&mut self, id: ActorId) -> Option<&mut dyn crate::reflect::ReflectObject> {
        let tag = self.tags.get(id.type_tag() as usize)?;
        let pool = self.pools.get_mut(tag)?;
        (pool.reflect_mut)(pool.pool.as_mut(), id)
    }

    /// Whether an id still refers to a live actor, without knowing its type.
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

    /// Every actor of one type, with its id. The fast path the storage model
    /// was chosen for — actors of one type are contiguous.
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

    /// Declare que les acteurs `T` portent des composants `C`. Explicite :
    /// l'oublier fait ignorer ce type silencieusement.
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

        // Force la creation du pool, sinon l'enregistrement serait perdu.
        let _ = self.pool_mut::<T>();

        if let Some(erased) = self.pools.get_mut(&TypeId::of::<T>()) {
            erased.components.insert(TypeId::of::<C>(), iterate);
        }
    }

    /// Applique `f` a chaque composant `C`, quel que soit l'acteur. Un rappel
    /// plutot qu'un iterateur : collecter allouerait chaque frame.
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

        // `entry` : une seule recherche la ou contains_key + insert en fait deux.
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
                reflect: |pool, id| {
                    let actor = pool.downcast_ref::<Pool<T>>()?.get(id)?;
                    Some(actor as &dyn crate::reflect::ReflectObject)
                },
                reflect_mut: |pool, id| {
                    let actor = pool.downcast_mut::<Pool<T>>()?.get_mut(id)?;
                    Some(actor as &mut dyn crate::reflect::ReflectObject)
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
