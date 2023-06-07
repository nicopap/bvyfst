mod entity;
mod hierarchy;
mod loader;
mod saver;
mod scene;
mod version;

type Archived<T> = <T as rkyv::Archive>::Archived;
