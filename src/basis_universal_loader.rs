//! Due to a limitation in the bevy asset loader, we need to locally copy
//! The basis-universal image loading code in this crate.

use basis_universal::Transcoder;
use thiserror::Error;

use ::bevy::render::render_resource::SamplerDescriptor;
use bevy::prelude as bevy;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid basis file format")]
    Invalid,
}

pub fn load(bytes: &[u8]) -> Result<ImageIter, Error> {
    todo!()
}
pub struct ImageIter<'a> {
    bytes: &'a [u8],
    transcoder: Transcoder,
    current: u32,
}
impl<'a> Iterator for ImageIter<'a> {
    type Item = bevy::Image;

    fn next(&mut self) -> Option<Self::Item> {
        let count = self.transcoder.image_count(self.bytes);
        if count == self.current {
            return None;
        }
        let image = todo!();
        self.current += 1;

        Some(image)
    }
}
