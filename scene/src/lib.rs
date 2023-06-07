#![warn(clippy::pedantic)]
#![allow(clippy::use_self)]

mod basis_universal_loader;
mod entry_ext;
mod fast;
mod loader;
mod saver;

use version::{Version, VERSION};

fn err_string(header: &ayar::Header) -> String {
    String::from_utf8_lossy(header.identifier()).into_owned()
}
