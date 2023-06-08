//! Proxy types to ser/deser components with a separate layout

use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Infallible, Serialize};

use crate::ArchiveProxy;

/// A way to automatically implement [`ArchiveProxy`] for your own type, if you
/// can implement `Archive` and `Clone` on them.
#[derive(Clone, Copy, Default, Archive, Deserialize, Serialize)]
pub struct Id<T>(pub T);

impl<T> ArchiveProxy for Id<T>
where
    T: Archive + bevy::Component + Clone,
    T::Archived: Deserialize<T, Infallible>,
{
    type Target = T;
    fn to_target(archive: &Self::Archived) -> Self::Target {
        archive.0.deserialize(&mut Infallible).unwrap()
    }
    fn from_target(target: &Self::Target) -> Self {
        Id(target.clone())
    }
}
