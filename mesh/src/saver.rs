use std::io::Cursor;

use anyhow::Result as AnyResult;
use bevy::{
    asset::io::Writer,
    asset::{saver::AssetSaver, AsyncWriteExt},
    prelude::Mesh,
    utils::BoxedFuture,
};
use tmf::TMFPrecisionInfo;

use crate::{mesh_converter::Bevy2Tmf, MeshLoader};

pub struct MeshSaver;
impl AssetSaver for MeshSaver {
    type Asset = Mesh;

    // TODO: proxy TMFPrecisionInfo and provide it as setting.
    type Settings = ();

    type OutputLoader = MeshLoader;

    fn save<'a>(
        &'a self,
        writer: &'a mut Writer,
        bevy_mesh: &'a Mesh,
        _: &'a (),
    ) -> BoxedFuture<'a, AnyResult<()>> {
        Box::pin(async move {
            let tmf_mesh = bevy_mesh.clone().into_tmf();
            let tmf_infos = TMFPrecisionInfo::default();

            let mut bytes = Vec::new();
            tmf_mesh.write_tmf_one(&mut Cursor::new(&mut bytes), &tmf_infos, "bevy_mesh")?;
            writer.write(&bytes).await?;

            Ok(())
        })
    }
}
