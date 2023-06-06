#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::use_self)]

mod basis_universal_loader;
mod entry_ext;
mod fast;
mod hierarchy;
mod loader;
mod mesh_converter;
mod saver;
mod version;

type Archived<T> = <T as rkyv::Archive>::Archived;

use version::{Version, VERSION};

fn err_string(header: &ar::Header) -> String {
    String::from_utf8_lossy(header.identifier()).into_owned()
}
