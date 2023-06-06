//! Write/Read a bevy hierarchy into a slice

use bevy::{
    ecs::world::EntityMut,
    prelude::{BuildWorldChildren, Handle, Mesh, SpatialBundle, StandardMaterial},
};

use crate::{fast, Archived};

pub type Meshes<'a> = &'a [Handle<Mesh>];
pub type Mats<'a> = &'a [Handle<StandardMaterial>];

pub struct Run<'a> {
    scene: &'a [Archived<fast::Entity>],
}
impl<'a> Run<'a> {
    pub const fn new(scene: &'a [Archived<fast::Entity>]) -> Self {
        Run { scene }
    }
    fn extract_children(&mut self, at: u32) -> Self {
        let scene;
        (scene, self.scene) = self.scene.split_at(at as usize);
        Run { scene }
    }
    fn next(&mut self) -> Option<&Archived<fast::Entity>> {
        let entity;
        (entity, self.scene) = self.scene.split_first()?;
        Some(entity)
    }
    pub fn run(mut self, mut spawner: EntityMut, meshes: Meshes, mats: Mats) {
        spawner.with_children(|spawner| loop {
            let Some(entity) = self.next() else { return; };
            let bevy_entity = spawner.spawn((
                unsafe { entity.mesh.pick(meshes) }.clone(),
                unsafe { entity.material.pick(mats) }.clone(),
                SpatialBundle::from_transform(entity.transform.to_bevy()),
            ));
            let children = entity.children;
            let children = self.extract_children(children);
            children.run(bevy_entity, meshes, mats);
        });
    }
}
