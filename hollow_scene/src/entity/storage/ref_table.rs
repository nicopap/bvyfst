use std::{marker::PhantomData, num::NonZeroU32};

use bevy::{
    ecs::{
        query::{ROQueryItem, WorldQuery},
        system::EntityCommands,
        world::EntityMut,
    },
    prelude::Bundle,
};
use rkyv::{Archive, Deserialize, Serialize};

use super::ArchiveProxy;

type Archived<T> = <T as Archive>::Archived;

// -------------------------------------
//              SPAWNER
// -------------------------------------

/// Enables spawning entites from tables with `EntityCommands` and `EntityMut`
pub trait EntitySpawner {
    fn insert<B: Bundle>(&mut self, bundle: B);
}
impl EntitySpawner for &'_ mut EntityCommands<'_, '_, '_> {
    fn insert<B: Bundle>(&mut self, bundle: B) {
        EntityCommands::insert(self, bundle);
    }
}
impl EntitySpawner for &'_ mut EntityMut<'_> {
    fn insert<B: Bundle>(&mut self, bundle: B) {
        EntityMut::insert(self, bundle);
    }
}

// -------------------------------------
//               TABLES
// -------------------------------------

/// A collection of values referenced by [`Keys`] representing bevy components.
pub trait Tables<K: Keys>: Archive {
    /// Deserialize a collection of components to directly insert into the ECS.
    fn insert_archived_keys<S: EntitySpawner>(
        archive: &Self::Archived,
        keys: &K::Archived,
        cmds: S,
    );
    fn new() -> Self;
    fn insert_entity_components(&mut self, components: ComponentsOf<K>) -> K;
}

#[derive(Archive, Deserialize, Serialize)]
struct Table<C> {
    table: Vec<C>,
}
impl<C: ArchiveProxy> Table<C> {
    fn store(&mut self, c: &C::Target) -> usize {
        self.table.push(C::from(c));
        self.table.len()
    }
}
impl<C: ArchiveProxy> ArchivedTable<C> {
    fn insert_at(&self, key: &ArchivedKey<C>, cmds: &mut impl EntitySpawner) {
        if let Some(index) = key.index.as_ref() {
            let index = usize::try_from(index.get() - 1).unwrap();
            // SAFETY: by construction, all keys are compatible with tables of given type.
            let component = unsafe { self.table.get_unchecked(index) };
            cmds.insert(C::Target::from(component));
        }
    }
}
impl Tables<()> for () {
    fn insert_archived_keys<S: EntitySpawner>(_: &(), _: &(), _: S) {}
    fn new() {}
    fn insert_entity_components(&mut self, _: ()) {}
}
impl<Hk: ArchiveProxy, Tk: Keys, Tt: Tables<Tk>> Tables<(Key<Hk>, Tk)> for (Table<Hk>, Tt) {
    fn insert_archived_keys<S: EntitySpawner>(
        (head, tail): &(ArchivedTable<Hk>, Tt::Archived),
        (key_head, key_tail): &(ArchivedKey<Hk>, Archived<Tk>),
        mut cmds: S,
    ) {
        head.insert_at(key_head, &mut cmds);
        Tt::insert_archived_keys(tail, key_tail, cmds);
    }
    fn new() -> Self {
        (Table { table: Vec::new() }, Tt::new())
    }
    fn insert_entity_components(
        &mut self,
        (head, tail): ComponentsOf<(Key<Hk>, Tk)>,
    ) -> (Key<Hk>, Tk) {
        let head = head.map(|c| self.0.store(c));
        let tail = self.1.insert_entity_components(tail);
        (Key::from_index(head), tail)
    }
}

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct TableStorage<Ts> {
    tables: Ts,
}

impl<Ts> TableStorage<Ts> {
    pub(crate) fn new<K: Keys>() -> Self
    where
        Ts: Tables<K>,
    {
        TableStorage { tables: Ts::new() }
    }
}
impl<Ts> TableStorage<Ts> {
    pub fn insert_values<K: Keys>(&mut self, values: ComponentsOf<K>) -> KeyStorage<K>
    where
        Ts: Tables<K>,
    {
        KeyStorage(self.tables.insert_entity_components(values))
    }
}
impl<Ts: Archive> ArchivedTableStorage<Ts> {
    pub fn spawn_keys<K: Keys>(&self, keys: &ArchivedKeyStorage<K>, cmds: impl EntitySpawner)
    where
        Ts: Tables<K>,
    {
        Ts::insert_archived_keys(&self.tables, &keys.0, cmds);
    }
}

// -------------------------------------
//                KEYS
// -------------------------------------

pub type ComponentsOf<'w, K> = ROQueryItem<'w, <K as Keys>::Query>;

/// A collection of keys in [`Tables`] to read components as external storage.
pub trait Keys: Archive {
    type Components;
    type Query: WorldQuery;
    fn empty() -> Self;
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
struct Key<C: ArchiveProxy> {
    index: Option<NonZeroU32>,
    _value_ty: PhantomData<fn(C)>,
}
impl<C: ArchiveProxy> Key<C> {
    fn from_index(index: Option<usize>) -> Self {
        Key {
            index: index.map(|i| NonZeroU32::new(u32::try_from(i).unwrap()).unwrap()),
            _value_ty: PhantomData,
        }
    }
}

impl Keys for () {
    type Components = ();
    type Query = ();
    fn empty() -> Self {}
}
impl<C: ArchiveProxy, Tl: Keys> Keys for (Key<C>, Tl) {
    type Components = (Option<C>, Tl::Components);
    type Query = (Option<&'static C::Target>, Tl::Query);
    fn empty() -> Self {
        (Key { index: None, _value_ty: PhantomData }, Tl::empty())
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct KeyStorage<Ks>(Ks);

impl<Ks: Keys> Default for KeyStorage<Ks> {
    fn default() -> Self {
        KeyStorage::no_component()
    }
}

impl<Ks: Keys> KeyStorage<Ks> {
    pub fn no_component() -> Self {
        KeyStorage(Ks::empty())
    }
}