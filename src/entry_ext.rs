//! Extension methods for the [`ar::Entry<R>`] type.

use bevy::asset::AsyncReadExt;
use futures_io::AsyncRead;
use thiserror::Error;

use crate::err_string;

#[derive(Debug, Error)]
pub enum Error {
    #[error("In the `.bvyfst` archive: while reading {file}: {err}")]
    Io { file: String, err: std::io::Error },
}
pub async fn load_bytes<R: AsyncRead + Unpin>(
    entry: &mut ar::Entry<'_, R>,
) -> Result<Box<[u8]>, Error> {
    let len = entry.header().size() as usize;

    let mut bytes = vec![0; len].into_boxed_slice();
    entry
        .read_exact(&mut bytes)
        .await
        .map_err(|err| Error::Io { file: err_string(entry.header()), err })?;
    Ok(bytes)
}
