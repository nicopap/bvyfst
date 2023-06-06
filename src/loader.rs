use std::io::Cursor;

use anyhow::Result as AnyResult;
use ar::Archive;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::{Handle, Image, Mesh, Scene, SpatialBundle, StandardMaterial, World},
    utils::BoxedFuture,
};
use futures_io::AsyncRead;
use rkyv::archived_root;
use thiserror::Error;
use tmf::TMFMesh;

use crate::{
    basis_universal_loader, entry_ext, entry_ext::load_bytes, err_string, fast, hierarchy,
    mesh_converter::Tmf2Bevy, version, Archived, VERSION,
};

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

struct FastSceneProcessor<'a, 'b, 'c, R: AsyncRead + Unpin + Send> {
    ctx: Ctx<'b, 'c>,
    archive: Archive<R>,
    scene: &'a Archived<fast::Scene>,
}

#[derive(Debug, Error)]
enum LoadError {
    #[error("The scene isn't compatible with the current version: (file: {0}, us: {VERSION})")]
    IncompatibleVersion(u16),
    #[error("Can't parse version from scene archived file: {0}")]
    InvalidVersion(#[from] version::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Load(#[from] entry_ext::Error),
    #[error("The meshes file is missing, only the scene file exists!")]
    OnlyScene,
    #[error("The 'images' file is missing")]
    NoImages,
    #[error("In the `.bvyfst` archive got wrong file: (expected '{expected}', got '{got}')")]
    WrongFile { expected: String, got: String },
    #[error(transparent)]
    Tmf(#[from] tmf::TMFImportError),
    #[error(transparent)]
    Image(#[from] basis_universal_loader::Error),
}

struct ValidatedSceneBytes(Box<[u8]>);
impl ValidatedSceneBytes {
    fn as_scene(&self) -> &Archived<fast::Scene> {
        // SAFETY: this is not safe, but supposedly, at this point, the user looked
        // for it, as they constructed an archive file with a valid version number.
        unsafe { archived_root::<fast::Scene>(&self.0) }
    }
    async fn new(
        mut entry: ar::Entry<'_, impl AsyncRead + Unpin + Send>,
    ) -> Result<ValidatedSceneBytes, LoadError> {
        let name = entry.header().identifier();
        if !name.starts_with(b"scene_v") {
            let expected = format!("scene_v{VERSION:0pad$}", pad = version::DIGIT_COUNT);
            let got = err_string(entry.header());
            return Err(LoadError::WrongFile { expected, got });
        }
        if !VERSION.digits_represents(name) {
            let actual_version = crate::Version::get_version_slice(name)?;
            return Err(LoadError::IncompatibleVersion(actual_version));
        }
        Ok(ValidatedSceneBytes(load_bytes(&mut entry).await?))
    }
}

pub struct Loader;
impl AssetLoader for Loader {
    type Asset = Scene;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        ctx: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, AnyResult<Scene>> {
        Box::pin(async move {
            let mut archive = Archive::new(reader);

            let first_entry = archive.next_entry().await.unwrap()?;
            let scene_bytes = ValidatedSceneBytes::new(first_entry).await?;
            let scene = scene_bytes.as_scene();

            let mut processor = FastSceneProcessor { ctx, archive, scene };
            processor.load_scene().await
        })
    }
    fn extensions(&self) -> &[&str] {
        &["bvyfst"]
    }
}
impl<'a, 'b, 'c, R: AsyncRead + Unpin + Send> FastSceneProcessor<'a, 'b, 'c, R> {
    async fn load_scene(&mut self) -> AnyResult<Scene> {
        let meshes = self.load_meshes().await?;
        let images = self.load_images().await?;
        let materials = self.load_materials(&images);
        let hierarchy = hierarchy::Run::new(self.scene.entities.as_ref());

        let mut scene_world = World::new();

        let root_entity = scene_world.spawn(SpatialBundle::default());
        hierarchy.run(root_entity, &meshes, &materials);

        Ok(Scene::new(scene_world))
    }
    async fn load_images(&mut self) -> Result<Box<[Handle<Image>]>, LoadError> {
        let no_images = LoadError::NoImages;

        let entry = self.archive.next_entry();
        let mut entry = entry.await.ok_or(no_images)??;
        if entry.header().identifier() != b"images" {
            let got = err_string(entry.header());
            return Err(LoadError::WrongFile { expected: "images".to_string(), got });
        }
        let bytes = load_bytes(&mut entry).await?;
        let images = basis_universal_loader::load(&bytes)?;

        let load_image =
            |(i, image): (usize, Image)| self.ctx.add_labeled_asset(format!("image_{i}"), image);
        Ok(images.enumerate().map(load_image).collect())
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
    async fn load_meshes(&mut self) -> Result<Box<[Handle<Mesh>]>, LoadError> {
        let only_scene = LoadError::OnlyScene;

        let entry = self.archive.next_entry();
        let mut entry = entry.await.ok_or(only_scene)??;
        if entry.header().identifier() != b"meshes" {
            let got = err_string(entry.header());
            return Err(LoadError::WrongFile { expected: "meshes".to_string(), got });
        }

        let load_mesh = |(mesh, name): (TMFMesh, String)| {
            let mesh = mesh.into_bevy();
            self.ctx.add_labeled_asset(name, mesh)
        };
        let bytes = load_bytes(&mut entry).await?;

        let tmf_mesh = TMFMesh::read_tmf_async(&mut Cursor::new(bytes)).await?;
        Ok(tmf_mesh.into_iter().map(load_mesh).collect())
    }
}
