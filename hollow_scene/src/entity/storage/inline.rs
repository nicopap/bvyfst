use bevy::ecs::query::{ROQueryItem, WorldQuery};
use rkyv::{Archive, Deserialize, Serialize};

use super::EntitySpawner;
use crate::{entity::ArchiveProxy, Inline};

pub type ComponentsOf<'w, I> = ROQueryItem<'w, <I as Inlines>::Query>;

/// A collection of [`ArchiveProxy`] stored directly in the entity array.
///
/// The value is stored inline, in the `Entity`, as an `Option<Self>`,
/// use this if most archived entities in the scene contains this component,
/// and the component in question doesn't occupy a lot of memory.
pub trait Inlines: Archive {
    type Query: WorldQuery;
    fn from_query_items(query: ComponentsOf<Self>) -> Self;
    fn insert_entity_components<S: EntitySpawner>(archive: &Self::Archived, cmds: &mut S);
    fn new() -> Self;
    fn occupancy(&self) -> String;
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct InlineStorage<I>(I);
impl<I: Inlines> ArchivedInlineStorage<I> {
    pub fn spawn(&self, mut cmds: impl EntitySpawner) {
        I::insert_entity_components(&self.0, &mut cmds);
    }
}
impl<I: Inlines> InlineStorage<I> {
    pub fn new() -> Self {
        InlineStorage(I::new())
    }
    pub fn occupancy(&self) -> String {
        self.0.occupancy()
    }
}

impl<I: Inlines> InlineStorage<I> {
    pub fn query(inline_query: ComponentsOf<I>) -> InlineStorage<I> {
        InlineStorage(I::from_query_items(inline_query))
    }
}

impl Inlines for () {
    type Query = ();
    #[inline]
    fn from_query_items((): ()) {}
    #[inline]
    fn insert_entity_components<S: EntitySpawner>((): &(), _: &mut S) {}
    #[inline]
    fn new() {}
    fn occupancy(&self) -> String {
        String::new()
    }
}
impl<H: ArchiveProxy, T: Inlines> Inlines for (Inline<H>, T) {
    type Query = (Option<&'static H::Target>, T::Query);

    #[inline]
    fn from_query_items((head, tail): (Option<&H::Target>, ComponentsOf<T>)) -> Self {
        let head = Inline(head.map(H::from_target));
        (head, T::from_query_items(tail))
    }
    #[inline]
    fn insert_entity_components<S: EntitySpawner>((head, tail): &Self::Archived, cmds: &mut S) {
        if let Some(value) = head.0.as_ref() {
            cmds.insert(H::to_target(value));
        }
        T::insert_entity_components(tail, cmds);
    }
    #[inline]
    fn new() -> Self {
        (Inline(None), T::new())
    }
    fn occupancy(&self) -> String {
        let head = if self.0 .0.is_some() { '#' } else { '_' };
        format!("{head}{}", self.1.occupancy())
    }
}
