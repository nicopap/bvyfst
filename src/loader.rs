use std::io::Cursor;

use anyhow::Result;
use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, LoadContext},
    prelude::{Handle, Image, Mesh, Scene, SpatialBundle, StandardMaterial, World},
    utils::BoxedFuture,
};
use rkyv::archived_root;
use tmf::TMFMesh;

use crate::{fast, hierarchy, mesh_converter::Tmf2Bevy, Archived};

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

struct FastSceneProcessor<'a, 'b, 'c> {
    ctx: Ctx<'b, 'c>,
    scene_file: AssetPath<'static>,
    scene: &'a Archived<fast::Scene>,
}

struct FastSceneLoader;
impl AssetLoader for FastSceneLoader {
    type Asset = Scene;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        ctx: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, Result<Scene>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut processor = FastSceneProcessor {
                scene_file: ctx.asset_path().to_owned(),
                scene: unsafe { archived_root::<fast::Scene>(&bytes) },
                ctx,
            };
            processor.load_scene().await
        })
    }
    fn extensions(&self) -> &[&str] {
        &["fstbvy"]
    }
}
impl<'a, 'b, 'c> FastSceneProcessor<'a, 'b, 'c> {
    async fn load_scene(&mut self) -> Result<Scene> {
        let images = self.load_images();
        let materials = self.load_materials(&images);
        let meshes = self.load_meshes().await?;
        let hierarchy = hierarchy::Run::new(self.scene.entities.as_ref());

        let mut scene_world = World::new();

        let root_entity = scene_world.spawn(SpatialBundle::default());
        hierarchy.run(root_entity, &meshes, &materials);

        Ok(Scene::new(scene_world))
    }
    fn load_images(&mut self) -> Box<[Handle<Image>]> {
        let load_image = |(i, img): (usize, &Archived<fast::Image>)| {
            let Some(path_prefix) = self.scene_file.path.file_stem() else {
                panic!("Somehow managed to load a file without a name")
            };
            let path_prefix = path_prefix.to_string_lossy();
            self.ctx.load(&format!("{path_prefix}_{i}.basis"))
        };
        let images = self.scene.images.iter();
        images.enumerate().map(load_image).collect()
    }
    fn load_materials(&mut self, images: &[Handle<Image>]) -> Box<[Handle<StandardMaterial>]> {
        // SAFETY: `images` is taken from same scene
        let load_mat = |(i, mat): (usize, &Archived<fast::Material>)| unsafe {
            self.ctx
                .add_labeled_asset(format!("Mat{i}"), mat.to_bevy(images))
        };
        let mats = self.scene.materials.iter();
        mats.enumerate().map(load_mat).collect()
    }
    async fn load_meshes(&mut self) -> Result<Box<[Handle<Mesh>]>> {
        let path = self.scene_file.path();
        let mut bytes = Cursor::new(self.ctx.read_asset_bytes(path).await?);
        let tmf_mesh = TMFMesh::read_tmf_async(&mut bytes).await?;

        let load_mesh = |(mesh, name): (TMFMesh, String)| {
            let mesh = mesh.into_bevy();
            self.ctx.add_labeled_asset(name, mesh)
        };
        Ok(tmf_mesh.into_iter().map(load_mesh).collect())
    }
}
