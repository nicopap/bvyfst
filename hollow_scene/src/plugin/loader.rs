use std::marker::PhantomData;

use anyhow::Result as AnyResult;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::{info, AppTypeRegistry, FromWorld, Scene, World},
    scene::SceneLoader,
    utils::BoxedFuture,
};

use crate::{entity::Inlines, entity::Tables, FastScene};

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

// TODO: parametrize over loaders
pub struct Loader<Ts, Is>(SceneLoader, AppTypeRegistry, PhantomData<fn(Ts, Is)>);

impl<Ts, Is> FromWorld for Loader<Ts, Is> {
    fn from_world(world: &mut World) -> Self {
        let scene_loader = FromWorld::from_world(world);
        let registry = world.resource::<AppTypeRegistry>();
        Loader(scene_loader, registry.clone(), PhantomData)
    }
}

impl<Ts: Tables + 'static, Is: Inlines + 'static> AssetLoader for Loader<Ts, Is> {
    type Asset = Scene;
    // TODO: use meta file to verify that the layout is valid
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        ctx: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, AnyResult<Scene>> {
        Box::pin(async move {
            // unwrap: this will only be called with the extensions defined in fn extensions
            match ctx.path().extension().unwrap().to_str().unwrap() {
                "hollow_bvyfst" => {
                    let mut bytes = Vec::new();
                    reader.read_to_end(&mut bytes).await?;
                    let fast_scene = unsafe { rkyv::archived_root::<FastScene<Ts, Is>>(&bytes) };
                    Ok(fast_scene.to_bevy())
                }
                "myscn" | "ron" => {
                    info!("got a dynamic scene, reading it");
                    let dynamic_scene = self.0.load(reader, &(), ctx).await?;
                    info!("turning dynamic scene into real scene");
                    let scene = Scene::from_dynamic_scene(&dynamic_scene, &self.1)?;
                    info!("completed the truing of dynamcis cene to real scen");
                    Ok(scene)
                }
                ext => unreachable!(
                    "Loader should only be called with extensions: {:?}, got '{ext}'",
                    self.extensions()
                ),
            }
        })
    }
    fn extensions(&self) -> &[&str] {
        &["hollow_bvyfst", "myscn", "myscn.ron"]
    }
}
