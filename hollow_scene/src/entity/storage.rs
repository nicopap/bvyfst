//! `Entity` can store components in one of three ways:
//!
//! 1. [`inline::Inlines`]: the value is stored inline, in the `Entity`, as an `Option<Self>`,
//!    use this if most archived entities in the scene contains this component.
//!    It is recommended that `Self` supports niching (ie:
//!    `size_of::<Option<Self>>() == size_of::<Self>()`, often the case with enums)
//! 2. [`ref_table::Tables`]: the value is stored in the `Scene`, as an array.
//!    The index is stored in the `Entity` as a `Option<NonZeroU32>`.
//!    Use this if the archived format occupies a lot of memory (something like
//!    several thousand bits or more), or if the same value is shared by many
//!    different entites.
//! 3. `extra::Extra`: In the `extra` table of the entity-specific extras table.
//!    The `extra` table is a magic thing that is not implemented yet, so don't use it.

use bevy::{ecs::system::EntityCommands, ecs::world::EntityMut, prelude::Bundle};

use super::ArchiveProxy;

pub mod inline;
pub mod ref_table;

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

// In the `extra` table of the entity-specific extras table.
// The `extra` table is a magic thing that is not implemented yet, so don't use it.
// It's an "heterogenous list of erased values"
