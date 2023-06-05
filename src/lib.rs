#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::use_self)]

mod fast;
mod hierarchy;
mod loader;
mod mesh_converter;
mod saver;

type Archived<T> = <T as rkyv::Archive>::Archived;

const VERSION: u16 = 1;
