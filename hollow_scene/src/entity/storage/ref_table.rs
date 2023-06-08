use core::fmt;
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

    const COMPONENT_COUNT: usize;
    fn component_count(&self, index: usize) -> usize;
    fn component_name(&self, index: usize) -> &'static str;
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
    #[inline]
    fn insert_archived_keys<S: EntitySpawner>(&(): &(), &(): &(), _: S) {}
    #[inline]
    fn new() {}
    #[inline]
    fn insert_entity_components(&mut self, (): ()) {}
    fn component_count(&self, _: usize) -> usize {
        panic!("Out of bound, terminal node isn't a component table")
    }
    fn component_name(&self, _: usize) -> &'static str {
        panic!("Out of bound, terminal node isn't a component table")
    }

    const COMPONENT_COUNT: usize = 0;
}
impl<Hk: ArchiveProxy, Tk: Keys, Tt: Tables<Keys = Tk>> Tables for (Table<Hk>, Tt) {
    type Keys = (Key<Hk>, Tk);

    #[inline]
    fn insert_archived_keys<S: EntitySpawner>(
        (head, tail): &(ArchivedTable<Hk>, Tt::Archived),
        (key_head, key_tail): &(ArchivedKey<Hk>, Archived<Tk>),
        mut cmds: S,
    ) {
        head.insert_at(key_head, &mut cmds);
        Tt::insert_archived_keys(tail, key_tail, cmds);
    }
    #[inline]
    fn new() -> Self {
        (Table { table: Vec::new() }, Tt::new())
    }
    #[inline]
    fn insert_entity_components(
        &mut self,
        (head, tail): ComponentsOf<(Key<Hk>, Tk)>,
    ) -> (Key<Hk>, Tk) {
        let head = head.map(|c| self.0.store(c));
        let tail = self.1.insert_entity_components(tail);
        (Key::from_index(head), tail)
    }
    fn component_count(&self, index: usize) -> usize {
        (index == 0)
            .then_some(self.0.table.len())
            .unwrap_or_else(|| self.1.component_count(index - 1))
    }
    fn component_name(&self, index: usize) -> &'static str {
        (index == 0)
            .then_some(std::any::type_name::<Hk::Target>())
            .unwrap_or_else(|| self.1.component_name(index - 1))
    }
    const COMPONENT_COUNT: usize = 1 + Tt::COMPONENT_COUNT;
}

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct TableStorage<Ts> {
    tables: Ts,
}

impl<Ts: Tables> TableStorage<Ts> {
    #[inline]
    pub(crate) fn new() -> Self {
        TableStorage { tables: Ts::new() }
    }
    #[inline]
    pub fn insert_values(&mut self, values: ComponentsOf<Ts::Keys>) -> KeyStorage<Ts::Keys> {
        KeyStorage(self.tables.insert_entity_components(values))
    }
    pub const fn component_count(&self) -> usize {
        Ts::COMPONENT_COUNT
    }
    pub fn component_count_of(&self, index: usize) -> usize {
        self.tables.component_count(index)
    }
    pub fn component_name(&self, index: usize) -> &'static str {
        self.tables.component_name(index)
    }
}
impl<Ts: Tables> ArchivedTableStorage<Ts> {
    #[inline]
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
    fn occupancy(&self) -> String;
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Key<C: ArchiveProxy> {
    index: Option<NonZeroU16>,
    _value_ty: PhantomData<fn(C)>,
}
impl<C: ArchiveProxy> fmt::Debug for Key<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(index) = self.index {
            write!(f, "K#{}", index.get())
        } else {
            write!(f, "K__")
        }
    }
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
    fn empty() {}
    fn occupancy(&self) -> String {
        String::new()
    }
}
impl<C: ArchiveProxy, Tl: Keys> Keys for (Key<C>, Tl) {
    type Query = (Option<&'static C::Target>, Tl::Query);

    #[inline]
    fn empty() -> Self {
        (Key { index: None, _value_ty: PhantomData }, Tl::empty())
    }
    fn occupancy(&self) -> String {
        format!("{:?}{}", &self.0, self.1.occupancy())
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct KeyStorage<Ks>(Ks);

impl<Ks: Keys> Default for KeyStorage<Ks> {
    #[inline]
    fn default() -> Self {
        KeyStorage::no_component()
    }
}

impl<Ks: Keys> KeyStorage<Ks> {
    pub fn no_component() -> Self {
        KeyStorage(Ks::empty())
    }
    pub fn occupancy(&self) -> String {
        self.0.occupancy()
    }
}
