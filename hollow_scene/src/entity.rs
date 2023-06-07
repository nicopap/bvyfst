pub mod storage;

use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Serialize};

pub use storage::ref_table::{KeyStorage, Keys, TableStorage, Tables};

mod sealed {
    pub trait Storage {}
}
pub trait Storage: sealed::Storage {}

pub trait ArchiveProxy: Archive + for<'a> From<&'a Self::Target> {
    type StorageType: Storage;

    type Target: bevy::Component + for<'a> From<&'a Self::Archived>;
}

#[derive(Clone, Copy, Default, Archive, Deserialize, Serialize)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
impl ArchivedTransform {
    pub fn to_bevy(&self) -> bevy::Transform {
        bevy::Transform {
            translation: self.translation.into(),
            rotation: bevy::Quat::from_array(self.rotation),
            scale: self.scale.into(),
        }
    }
}
impl From<&'_ bevy::Transform> for Transform {
    fn from(bevy: &'_ bevy::Transform) -> Self {
        Transform {
            translation: bevy.translation.into(),
            rotation: bevy.rotation.into(),
            scale: bevy.scale.into(),
        }
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Entity<Ks> {
    // How many entities following this one are its children.
    pub children: u32,
    pub ref_table_keys: KeyStorage<Ks>,
}
impl<Ks: Keys> Entity<Ks> {
    pub fn empty() -> Self {
        Entity {
            children: 0,
            ref_table_keys: KeyStorage::no_component(),
        }
    }
}
