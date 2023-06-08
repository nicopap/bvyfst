//! The value is stored inline, in the `Entity`, as an `Option<Self>`,
//! use this if most archived entities in the scene contains this component.
//! It is recommended that `Self` supports niching (ie:
//! `size_of::<Option<Self>>() == size_of::<Self>()`, often the case with enums)

use bevy::ecs::query::{ROQueryItem, WorldQuery};
use rkyv::{Archive, Deserialize, Serialize};

use super::EntitySpawner;
use crate::entity::ArchiveProxy;

pub type ComponentsOf<'w, I> = ROQueryItem<'w, <I as Inline>::Query>;

pub trait Inline: Archive + Default {
    type Items;
    type Query: WorldQuery;
    fn from_query_items(query: ComponentsOf<Self>) -> Self;
    fn insert_entity_components<S: EntitySpawner>(archive: &Self::Archived, cmds: &mut S);
}

#[derive(Clone, Copy, Default, Archive, Deserialize, Serialize)]
pub struct InlineStorage<I>(I);

impl<I: Inline> InlineStorage<I> {
    pub fn query(inline_query: ComponentsOf<I>) -> InlineStorage<I> {
        InlineStorage(I::from_query_items(inline_query))
    }
}

#[derive(Archive, Deserialize, Serialize, Default)]
struct InlineItem<C>(C);

impl Inline for () {
    type Items = ();
    type Query = ();
    fn from_query_items((): ()) {}
    fn insert_entity_components<S: EntitySpawner>((): &(), cmds: &mut S) {}
}
impl<H: ArchiveProxy + Default, T: Inline> Inline for (InlineItem<H>, T) {
    type Items = (Option<H>, T::Items);
    type Query = (Option<&'static H::Target>, T::Query);
    fn from_query_items((head, tail): ComponentsOf<Self>) -> Self {
        let head = InlineItem(head.map_or_else(H::default, H::from));
        (head, T::from_query_items(tail))
    }
    fn insert_entity_components<S: EntitySpawner>((head, tail): &Self::Archived, cmds: &mut S) {
        cmds.insert(H::Target::from(&head.0));
        T::insert_entity_components(tail, cmds);
    }
}
