use std::marker::PhantomData;

use bevy::{
    asset::{io::Writer, saver::AssetSaver, AsyncWriteExt},
    scene::Scene,
    utils::BoxedFuture,
};

use crate::{entity::Keys, entity::Tables, loader::Loader, scene::FastScene};

pub struct Saver<Ks, Ts>(PhantomData<fn(Ks, Ts)>);
impl<Ks: Keys + 'static, Ts: Tables<Ks> + 'static> AssetSaver for Saver<Ks, Ts> {
    type Asset = Scene;

    type Settings = ();

    type OutputLoader = Loader<Ks, Ts>;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: &'a Scene,
        settings: &'a (),
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let fast_scene = FastScene::<Ks, Ts>::from_bevy(&mut asset.clone());
            let bytes = rkyv::to_bytes(&fast_scene)?;
            writer.write(&bytes).await?;
            Ok(())
        })
    }
}
