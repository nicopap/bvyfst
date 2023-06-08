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
pub trait Inlines: Archive + Default {
    type Query: WorldQuery;
    fn from_query_items(query: ComponentsOf<Self>) -> Self;
    fn insert_entity_components<S: EntitySpawner>(archive: &Self::Archived, cmds: &mut S);
}

#[derive(Clone, Copy, Default, Archive, Deserialize, Serialize)]
pub struct InlineStorage<I>(I);

impl<I: Inlines> InlineStorage<I> {
    pub fn query(inline_query: ComponentsOf<I>) -> InlineStorage<I> {
        InlineStorage(I::from_query_items(inline_query))
    }
}

impl Inlines for () {
    type Query = ();
    fn from_query_items((): ()) {}
    fn insert_entity_components<S: EntitySpawner>((): &(), _: &mut S) {}
}
impl<H: ArchiveProxy + Default, T: Inlines> Inlines for (Inline<H>, T) {
    type Query = (Option<&'static H::Target>, T::Query);

    fn from_query_items((head, tail): ComponentsOf<Self>) -> Self {
        let head = Inline(head.map_or_else(H::default, H::from_target));
        (head, T::from_query_items(tail))
    }
    fn insert_entity_components<S: EntitySpawner>((head, tail): &Self::Archived, cmds: &mut S) {
        cmds.insert(H::to_target(&head.0));
        T::insert_entity_components(tail, cmds);
    }
}
