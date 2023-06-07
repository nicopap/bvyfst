use std::marker::PhantomData;

use anyhow::Result as AnyResult;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::{Scene, World},
    utils::BoxedFuture,
};

use crate::{
    entity::{Keys, Tables},
    hierarchy::Spawn,
    scene::FastScene,
};

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

pub struct Loader<Ks, Ts>(PhantomData<fn(Ks, Ts)>);
impl<Ks: Keys + 'static, Ts: Tables<Ks> + 'static> AssetLoader for Loader<Ks, Ts> {
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
            let fast_scene = unsafe { rkyv::archived_root::<FastScene<Ks, Ts>>(&bytes) };
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
