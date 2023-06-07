use bevy::{
    asset::{io::Writer, saver::AssetSaver},
    scene::Scene,
    utils::BoxedFuture,
};

use crate::loader::Loader;

pub struct Saver;
impl AssetSaver for Saver {
    type Asset = Scene;

    type Settings = ();

    type OutputLoader = Loader;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        asset: &'a Scene,
        settings: &'a (),
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        todo!()
    }
}

fn write_scene(scene: Scene) {
    todo!()
}
