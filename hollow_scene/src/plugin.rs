//! Define the bevy plugin

mod loader;
mod processor;
mod saver;

use std::marker::PhantomData;

use ::bevy::prelude::AssetApp;
use bevy::prelude as bevy;
use rkyv::ser::serializers::{
    AlignedSerializer, AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch,
    SharedSerializeMap,
};

use crate::entity::{Inlines, Tables};

/// Initialize the fast scene [`Plugin`]
///
/// # Syntax
///
/// ```text
/// Plugin!(
///    Inline[<ty>,*]
///    DedupTable[<ty>,*]
///    Table[<ty>,*]
///    Extras[]
/// );
/// ```
///
/// The order is important, and `Extras` is indeed always followed by an empty
/// list (it isn't implemented yet).
///
/// `Plugin!` accepts four storage types, and each storage types holds specific
/// [`crate::ArchiveProxy`], things that read and write to components.
///
/// Consider the file format as an array of entities. Each entity contains
/// many proxies. Each proxy corresponds to a single component.
///
/// For brievty, we will use `Component` and `proxy` interchangeably in the next
/// sections, but beware that — indeed — what is being stored is not the
/// component itself, but its proxy defined by `ArchiveProxy`.
///
/// With the provided storage formats you have:
///
/// - `Inline`: Every entity stores an `Option<Component>` for all components
///   in this section.
/// - `Table`: Every entity contains an `Option<NonZeroU16>` for all
///   components in this section. The `NonZeroU16` is an index to a table
///   where the actual component values are stored. The table is stored next
///   to the entity array.
///   \
///   Note that the index being a `NonZeroU16` means there can't be more than
///   65535 instances of the same component in table storage.
/// - `DedupTable`: Same as `Table`, except newly added components will be checked
///   against previously found components. If they match, the value won't be added,
///   the index of the existing one is used.
///   \
///   Note that this is a O(n²) operation at save-time, with `n` the number of
///   distinct components (typically this is O(n) for zero-sized types)
/// - `Extras`: **NOT IMPLEMENTED**
#[macro_export]
macro_rules! Plugin {
    (
        $(  Inline[$( $inline:ty ),* $(,)?] $(,)?  )?
        $(  DedupTable[$( $dedup_table:ty ),* $(,)?] $(,)?  )?
        $(  Table[$( $table:ty ),* $(,)?] $(,)?  )?
        $(  Extras[]  )?
    ) => {{
        fn is_proxy<T: $crate::ArchiveProxy>() {}
        fn is_reflect<T: $crate::ArchiveProxy>() where T::Target: $crate::__priv::Reflect {}
        fn is_partial_eq<T: $crate::ArchiveProxy>() where T: ::core::cmp::PartialEq<T::Target> {}

        $(  $(is_proxy::<$table>();)*  )?
        $(  $(is_proxy::<$dedup_table>();)*  )?
        $(  $(is_proxy::<$inline>();)*  )?
        $(  $(is_reflect::<$table>();)*  )?
        $(  $(is_reflect::<$dedup_table>();)*  )?
        $(  $(is_reflect::<$inline>();)*  )?
        $(  $(is_partial_eq::<$dedup_table>();)*  )?

        $crate::Plugin::<
            Plugin![@table [ $( $($table,)* )? ] Plugin![@dedup $($($dedup_table,)*)?]],
            Plugin![@inline $( $($inline,)* )?],
        >::IGNORE_THIS_ERROR_you_are_seeing_this_because_one_of_the_types_you_used_as_argument_to_Plugin_wasnt_valid___check_the_earlier_errors_to_know_which_ones()
    }};
    (@inline ) => { () };
    (@inline $head:ty, $($tail:ty,)*) => {
        ($crate::Inline<$head>, Plugin!(@inline $($tail,)*) )
    };
    (@table [] $tail:ty ) => { $tail };
    (@table [ $head:ty, $($tail:ty,)* ] $remaining:ty) => {
        ($crate::Table<$head>, Plugin!(@table [$($tail,)*] $remaining))
    };
    (@dedup ) => { () };
    (@dedup $head:ty, $($tail:ty,)*) => {
        ($crate::DedupTable<$head>, Plugin!(@dedup $($tail,)*) )
    };
}

// TODO: remove this total rkyv nonsense
pub trait RkyvTypeNonsense:
    rkyv::Serialize<
    CompositeSerializer<
        AlignedSerializer<rkyv::AlignedVec>,
        FallbackScratch<HeapScratch<1024>, AllocScratch>,
        SharedSerializeMap,
    >,
>
{
}
impl<T> RkyvTypeNonsense for T where
    T: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >
{
}

/// Create a scene serialization plugin for the provided types.
///
/// You muse use the [`Plugin!`] macro to create an instance of this plugin.
/// It provides enhanced error messages, and constructs transparently the inane
/// nonsense of a type parameter you need to specify to get it working.
pub struct Plugin<Ts, Is>(PhantomData<fn(Ts, Is)>);

impl<Ts: Tables + 'static, Is: Inlines + 'static> Plugin<Ts, Is>
where
    Ts::Keys: RkyvTypeNonsense,
    Ts: RkyvTypeNonsense,
    Is: RkyvTypeNonsense,
{
    #[doc(hidden)]
    #[allow(non_snake_case)]
    pub fn IGNORE_THIS_ERROR_you_are_seeing_this_because_one_of_the_types_you_used_as_argument_to_Plugin_wasnt_valid___check_the_earlier_errors_to_know_which_ones(
    ) -> Self {
        Plugin(PhantomData)
    }
}

impl<Ts: Tables + 'static, Is: Inlines + 'static> bevy::Plugin for Plugin<Ts, Is>
where
    Ts::Keys: RkyvTypeNonsense,
    Ts: RkyvTypeNonsense,
    Is: RkyvTypeNonsense,
{
    fn build(&self, app: &mut bevy::App) {
        app.init_asset::<bevy::Scene>()
            .init_asset_loader::<loader::Loader<Ts, Is>>();
        processor::insert::<Ts, Is>(&mut app.world);
    }
}
