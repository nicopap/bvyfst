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
///    Table[<ty>,*]
///    Extras[]
/// );
/// ```
///
/// The order is important, and `Extras` is indeed always followed by an empty
/// list (it isn't implemented yet).
///
/// `Plugin!` accepts three storage types, and each storage types holds specific
/// [`crate::ArchiveProxy`], things that read and write to components.
#[macro_export]
macro_rules! Plugin {
    (
        Inline[$( $inline:ty ),* $(,)?] $(,)?
        Table[$( $table:ty ),* $(,)?] $(,)?
        Extras[]
    ) => {{
        fn is_proxy<T: $crate::ArchiveProxy>() {}
        fn is_reflect<T: $crate::ArchiveProxy>() where T::Target: $crate::__priv::Reflect {}
        fn is_default<T: ::core::default::Default>() {}

        $(is_proxy::<$table>();)*
        $(is_proxy::<$inline>();)*
        $(is_reflect::<$table>();)*
        $(is_reflect::<$inline>();)*
        $(is_default::<$inline>();)*

        $crate::Plugin::<
            Plugin![@table $($table,)*],
            Plugin![@inline $($inline,)*],
        >::IGNORE_THIS_ERROR_you_are_seeing_this_because_one_of_the_types_you_used_as_argument_to_Plugin_wasnt_valid___check_the_earlier_errors_to_know_which_ones()
    }};
    (@inline ) => { () };
    (@inline $head:ty, $($tail:ty,)*) => {
        ($crate::Inline<$head>, Plugin!(@inline $($tail,)*) )
    };
    (@table ) => { () };
    (@table $head:ty, $($tail:ty,)*) => {
        ($crate::Table<$head>, Plugin!(@table $($tail,)*) )
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
