use std::marker::PhantomData;

use bevy::{
    asset::{io::Writer, saver::AssetSaver, AsyncWriteExt},
    prelude::{info, AppTypeRegistry, FromWorld, World},
    scene::Scene,
    utils::BoxedFuture,
};
use rkyv::ser::{serializers::AllocSerializer, Serializer};

use super::{loader::Loader, processor::Format, RkyvTypeNonsense};
use crate::{entity::Inlines, entity::Tables, FastScene};

pub struct Saver<Ts, Is>(Option<AppTypeRegistry>, PhantomData<fn(Ts, Is)>);
impl<Ts: Tables + 'static, Is: Inlines + 'static> AssetSaver for Saver<Ts, Is>
where
    Ts::Keys: RkyvTypeNonsense,
    Ts: RkyvTypeNonsense,
    Is: RkyvTypeNonsense,
{
    type Asset = Scene;

    type Settings = Format;

    type OutputLoader = Loader<Ts, Is>;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: &'a Scene,
        _: &'a Format,
    ) -> BoxedFuture<'a, Result<Format, anyhow::Error>> {
        Box::pin(async move {
            info!("Saving a scene as hollow_bvyfst");
            let bytes = if let Some(registry) = &self.0 {
                let mut scene_world = asset.clone_with(registry)?;
                let fast_scene = FastScene::<Ts, Is>::from_bevy(&mut scene_world);
                let mut serializer = AllocSerializer::<1024>::default();
                serializer.serialize_value(&fast_scene)?;
                serializer.into_serializer().into_inner()
            } else {
                return Err(anyhow::anyhow!(
                    "The appregistry doesn't exist, can't save scenes"
                ));
            };
            writer.write(&bytes).await?;
            Ok(Format::Fast)
        })
    }
}
impl<Ts: Tables + 'static, Is: Inlines + 'static> FromWorld for Saver<Ts, Is> {
    fn from_world(world: &mut World) -> Self {
        let registry = world.get_resource::<AppTypeRegistry>();
        if registry.is_none() {
            info!(
                "Your bevy plugin config isn't setup to use asset processing. \
                Scenes won't be saved in the hllwfstbvy format."
            );
        };
        Saver(registry.map(Clone::clone), PhantomData)
    }
}
