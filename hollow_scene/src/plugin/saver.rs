use std::marker::PhantomData;

use bevy::{
    asset::{io::Writer, saver::AssetSaver, AsyncWriteExt},
    prelude::AppTypeRegistry,
    scene::Scene,
    utils::BoxedFuture,
};
use rkyv::ser::{
    serializers::{
        AlignedSerializer, AllocScratch, AllocSerializer, CompositeSerializer, FallbackScratch,
        HeapScratch, SharedSerializeMap,
    },
    Serializer,
};

use super::loader::Loader;
use crate::{entity::Inline, entity::Tables, FastScene};

pub struct Saver<Ts, Is>(PhantomData<fn(Ts, Is)>, AppTypeRegistry);
impl<Ts: Tables + 'static, Is: Inline + 'static> AssetSaver for Saver<Ts, Is>
where
    // TODO: remove this total rkyv nonsense
    Ts: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
    Ts::Keys: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
    Is: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
{
    type Asset = Scene;

    type Settings = ();

    type OutputLoader = Loader<Ts, Is>;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: &'a Scene,
        _: &'a (),
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let serializer = {
                let mut scene_world = asset.clone_with(&self.1)?;
                let fast_scene = FastScene::<Ts, Is>::from_bevy(&mut scene_world);
                let mut serializer = AllocSerializer::<1024>::default();
                serializer.serialize_value(&fast_scene)?;
                serializer
            };
            let bytes = serializer.into_serializer().into_inner();
            writer.write(&bytes).await?;
            Ok(())
        })
    }
}
