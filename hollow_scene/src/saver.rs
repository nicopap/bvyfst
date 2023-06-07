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

use crate::{entity::Keys, entity::Tables, loader::Loader, scene::FastScene};

pub struct Saver<Ks, Ts>(PhantomData<fn(Ks, Ts)>, AppTypeRegistry);
impl<Ks: Keys + 'static, Ts: Tables<Ks> + 'static> AssetSaver for Saver<Ks, Ts>
where
    // TODO: remove this total rkyv nonsense
    Ks: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
    Ts: rkyv::Serialize<
        CompositeSerializer<
            AlignedSerializer<rkyv::AlignedVec>,
            FallbackScratch<HeapScratch<1024>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
{
    type Asset = Scene;

    type Settings = ();

    type OutputLoader = Loader<Ks, Ts>;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: &'a Scene,
        _: &'a (),
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let serializer = {
                let mut scene_world = asset.clone_with(&self.1)?;
                let fast_scene = FastScene::<Ks, Ts>::from_bevy(&mut scene_world);
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
