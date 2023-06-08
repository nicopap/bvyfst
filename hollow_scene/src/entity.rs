use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Serialize};

pub mod storage;

pub use storage::{
    inline::{InlineStorage, Inlines},
    ref_table::{KeyStorage, Keys, TableStorage, Tables},
};

pub trait ArchiveProxy: Archive {
    // TODO: consider using bevy::Bundle for `N components -> 1 proxy` relations
    // this might be useful for components of size smaller than 1 byte, we could
    // put them together to avoid memory bloat.
    type Target: bevy::Component;

    fn to_target(archive: &Self::Archived) -> Self::Target;
    fn from_target(target: &Self::Target) -> Self;
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Entity<Keys, Inlines> {
    // How many entities following this one are its children.
    pub children: u32,
    pub inline_items: InlineStorage<Inlines>,
    pub ref_table_keys: KeyStorage<Keys>,
}
impl<Ks: Keys, Is: Inlines> Entity<Ks, Is> {
    pub fn with_children(children: u32) -> Self {
        Entity {
            children,
            inline_items: InlineStorage::new(),
            ref_table_keys: KeyStorage::no_component(),
        }
    }
}
