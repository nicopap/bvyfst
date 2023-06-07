use std::io::Cursor;

use anyhow::Result as AnyResult;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::Mesh,
    utils::BoxedFuture,
};
use tmf::TMFMesh;

use crate::mesh_converter::Tmf2Bevy;

type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

pub struct MeshLoader;
impl AssetLoader for MeshLoader {
    type Asset = Mesh;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a (),
        ctx: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, AnyResult<Mesh>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes);
            let (tmf_mesh, _) = TMFMesh::read_tmf_one_async(&mut Cursor::new(bytes)).await?;

            Ok(tmf_mesh.into_bevy())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["tmf"]
    }
}
