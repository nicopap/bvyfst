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
mod plugin;
mod scene;
mod version;

use scene::FastScene;

type Archived<T> = <T as rkyv::Archive>::Archived;
