// Architecture:
//
// - `entity`: Define a `rkyv`-based scene format parametrized over the kind of
//   components to ser/deser.
// - `entity::storage`: Define storage types to store components in [`FastScene`]
// - `hierarchy`: how to load from/to a bevy hierarchy to/from a [`FastScene`].
// - `scene`: define the [`FastScene`] struct, used to proxy bevy entities
// - `plugin`: Define the bevy plugin
// - `plugin::{saver,loader}`: Define bevy `AssetLoader` and `AssetSaver` for
//   bevy's `Scene` type based on a [`FastScene`].
// - `scene`: Define [`FastScene`]. However, most of the interesting code for
//   loading/saving the scene is in `hierarchy`. While the interesting code to
//   convert a list of types into a serializable data structure is in `entity`.
mod entity;
mod hierarchy;
#[cfg(feature = "bevy_plugin")]
mod plugin;
pub mod proxy;
mod scene;
mod version;

pub use crate::entity::ArchiveProxy;
#[cfg(feature = "bevy_plugin")]
pub use plugin::{Plugin, RkyvTypeNonsense};
pub use rkyv::{Archive, Deserialize, Serialize};

use scene::FastScene;

/// Expose bevy types used in `Plugin!` macro to check they are correct.
#[doc(hidden)]
pub mod __priv {
    pub use bevy::reflect::Reflect;
}

#[derive(Archive, Deserialize, Serialize, Default)]
#[doc(hidden)]
pub struct Inline<C>(Option<C>);

#[derive(Archive, Deserialize, Serialize)]
#[doc(hidden)]
pub struct Table<C> {
    table: Vec<C>,
}

#[derive(Archive, Deserialize, Serialize)]
#[doc(hidden)]
pub struct DedupTable<C>(pub(crate) Table<C>);
