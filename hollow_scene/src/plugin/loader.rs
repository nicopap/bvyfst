use std::marker::PhantomData;

use anyhow::Result as AnyResult;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::{Scene, World},
    utils::BoxedFuture,
};

use crate::{
    entity::{Inline, Tables},
    hierarchy::Spawn,
    FastScene,
};

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

pub struct Loader<Ts, Is>(PhantomData<fn(Ts, Is)>);
impl<Ts: Tables + 'static, Is: Inline + 'static> AssetLoader for Loader<Ts, Is> {
    type Asset = Scene;
    // TODO: use meta file to verify that the layout is valid
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        _: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, AnyResult<Scene>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let fast_scene = unsafe { rkyv::archived_root::<FastScene<Ts, Is>>(&bytes) };
            let mut world = World::new();

            let root_entity = world.spawn_empty();
            let spawn = Spawn::new(&fast_scene.entities, &fast_scene.tables);
            spawn.children_of(root_entity);

            Ok(Scene::new(world))
        })
    }
    fn extensions(&self) -> &[&str] {
        &["hollowbvyfst"]
    }
}
