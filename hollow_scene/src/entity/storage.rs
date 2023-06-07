//! `Entity` can store components in one of three ways:
//!
//! 1. [`storage::Inline`]: the value is stored inline, in the `Entity`, as an `Option<Self>`,
//!    use this if most archived entities in the scene contains this component.
//!    It is recommended that `Self` supports niching (ie:
//!    `size_of::<Option<Self>>() == size_of::<Self>()`, often the case with enums)
//! 2. [`storage::RefTable`]: the value is stored in the `Scene`, as an array.
//!    The index is stored in the `Entity` as a `Option<NonZeroU32>`.
//!    Use this if the archived format occupies a lot of memory (something like
//!    several thousand bits or more), or if the same value is shared by many
//!    different entites.
//! 3. [`storage::Extra`]: In the `extra` table of the entity-specific extras table.
//!    The `extra` table is a magic thing that is not implemented yet, so don't use it.

use super::{sealed, ArchiveProxy, Storage};

pub(super) mod inline;
pub(super) mod ref_table;

/// In the `extra` table of the entity-specific extras table.
/// The `extra` table is a magic thing that is not implemented yet, so don't use it.
pub enum Extra {}
impl Storage for Extra {}
impl sealed::Storage for Extra {}

/// The value is stored in the `Scene`, as an array.
/// The index is stored in the `Entity` as a `Option<NonZeroU32>`.
/// Use this if the archived format occupies a lot of memory (something like
/// several thousand bits or more), or if the same value is shared by many
/// different entites.
pub enum RefTable {}
impl Storage for RefTable {}
impl sealed::Storage for RefTable {}

/// The value is stored inline, in the `Entity`, as an `Option<Self>`,
/// use this if most archived entities in the scene contains this component.
/// It is recommended that `Self` supports niching (ie:
/// `size_of::<Option<Self>>() == size_of::<Self>()`, often the case with enums)
pub enum Inline {}
impl Storage for Inline {}
impl sealed::Storage for Inline {}
