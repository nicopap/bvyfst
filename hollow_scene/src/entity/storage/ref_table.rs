use std::{marker::PhantomData, num::NonZeroU16};

use bevy::ecs::query::{ROQueryItem, WorldQuery};
use rkyv::{Archive, Archived, Deserialize, Serialize};

use super::{ArchiveProxy, EntitySpawner};
use crate::{ArchivedTable, Table};

// -------------------------------------
//               TABLES
// -------------------------------------

/// A collection of values referenced by [`Keys`] representing bevy components.
///
/// The value is stored in the `Scene`, as an array.
/// The index is stored in the `Entity` as a `Option<NonZeroU16>`.
/// Use this if the archived format occupies a lot of memory (something like
/// several thousand bits or more), or if the same value is shared by many
/// different entites.
pub trait Tables: Archive {
    type Keys: Keys;

    /// Deserialize a collection of components to directly insert into the ECS.
    fn insert_archived_keys<S: EntitySpawner>(
        archive: &Self::Archived,
        keys: &Archived<Self::Keys>,
        cmds: S,
    );
    fn new() -> Self;
    fn insert_entity_components(&mut self, components: ComponentsOf<Self::Keys>) -> Self::Keys;
}

impl<C: ArchiveProxy> Table<C> {
    fn store(&mut self, c: &C::Target) -> usize {
        self.table.push(C::from_target(c));
        self.table.len()
    }
}
impl<C: ArchiveProxy> ArchivedTable<C> {
    fn insert_at(&self, key: &ArchivedKey<C>, cmds: &mut impl EntitySpawner) {
        if let Some(index) = key.index.as_ref() {
            let index = usize::try_from(index.get() - 1).unwrap();
            // SAFETY: by construction, all keys are compatible with tables of given type.
            let component = unsafe { self.table.get_unchecked(index) };
            cmds.insert(C::to_target(component));
        }
    }
}
impl Tables for () {
    type Keys = ();
    fn insert_archived_keys<S: EntitySpawner>(&(): &(), &(): &(), _: S) {}
    fn new() {}
    fn insert_entity_components(&mut self, (): ()) {}
}
impl<Hk: ArchiveProxy, Tk: Keys, Tt: Tables<Keys = Tk>> Tables for (Table<Hk>, Tt) {
    type Keys = (Key<Hk>, Tk);

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

impl<Ts: Tables> TableStorage<Ts> {
    pub(crate) fn new() -> Self {
        TableStorage { tables: Ts::new() }
    }
}
impl<Ts: Tables> TableStorage<Ts> {
    pub fn insert_values(&mut self, values: ComponentsOf<Ts::Keys>) -> KeyStorage<Ts::Keys> {
        KeyStorage(self.tables.insert_entity_components(values))
    }
}
impl<Ts: Tables> ArchivedTableStorage<Ts> {
    pub fn spawn_keys(&self, keys: &ArchivedKeyStorage<Ts::Keys>, cmds: impl EntitySpawner) {
        Ts::insert_archived_keys(&self.tables, &keys.0, cmds);
    }
}

// -------------------------------------
//                KEYS
// -------------------------------------

pub type ComponentsOf<'w, K> = ROQueryItem<'w, <K as Keys>::Query>;

/// A collection of keys in [`Tables`] to read components as external storage.
pub trait Keys: Archive {
    type Query: WorldQuery;
    fn empty() -> Self;
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Key<C: ArchiveProxy> {
    index: Option<NonZeroU16>,
    _value_ty: PhantomData<fn(C)>,
}
impl<C: ArchiveProxy> Key<C> {
    fn from_index(index: Option<usize>) -> Self {
        Key {
            index: index.map(|i| NonZeroU16::new(u16::try_from(i).unwrap()).unwrap()),
            _value_ty: PhantomData,
        }
    }
}

impl Keys for () {
    type Query = ();
    fn empty() -> Self {}
}
impl<C: ArchiveProxy, Tl: Keys> Keys for (Key<C>, Tl) {
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
