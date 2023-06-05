//! Write/Read a bevy hierarchy into a slice

use bevy::{
    ecs::world::EntityMut,
    prelude::{BuildWorldChildren, Handle, Mesh, SpatialBundle, StandardMaterial},
};

use crate::{fast, Archived};

pub struct Run<'a> {
    scene: &'a [Archived<fast::Entity>],
}
impl<'a> Run<'a> {
    pub fn new(scene: &'a [Archived<fast::Entity>]) -> Self {
        Run { scene }
    }
    fn extract_children(&mut self, at: u32) -> Self {
        let (start, scene) = self.scene.split_at(at as usize);
        let end = Run { scene };

        self.scene = start;
        end
    }
    fn next(&mut self) -> Option<&Archived<fast::Entity>> {
        let (entity, scene) = self.scene.split_first()?;
        self.scene = scene;

        Some(entity)
    }
    pub fn run(
        mut self,
        mut spawner: EntityMut,
        meshes: &[Handle<Mesh>],
        mats: &[Handle<StandardMaterial>],
    ) {
        spawner.with_children(|spawner| {
            while !self.scene.is_empty() {
                let Some(entity) = self.next() else { return; };
                let bevy_entity = spawner.spawn((
                    unsafe { entity.mesh.pick(meshes) }.clone(),
                    unsafe { entity.material.pick(mats) }.clone(),
                    SpatialBundle::default(),
                ));
                let children = entity.children;
                let children = self.extract_children(children);
                children.run(bevy_entity, meshes, mats);
            }
        });
    }
}
