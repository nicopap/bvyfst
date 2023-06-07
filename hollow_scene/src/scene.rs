use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    entity::{Entity, Keys, TableStorage, Tables},
    hierarchy::{self, Spawn},
};

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct FastScene<Ks: Keys, Ts: Tables<Ks>> {
    pub entities: Box<[Entity<Ks>]>,
    pub tables: TableStorage<Ts>,
}
impl<Ks: Keys, Ts: Tables<Ks>> ArchivedFastScene<Ks, Ts> {
    pub fn into_bevy(&self) -> bevy::Scene {
        let mut world = bevy::World::new();

        let root_entity = world.spawn_empty();
        let spawn = Spawn::new(&self.entities, &self.tables);
        spawn.children_of(root_entity);

        bevy::Scene::new(world)
    }
}
impl<Ks: Keys, Ts: Tables<Ks>> FastScene<Ks, Ts> {
    pub fn from_bevy(scene: &mut bevy::Scene) -> Self {
        let mut tables = TableStorage::new();
        let entities = hierarchy::build(&mut scene.world, &mut tables);
        FastScene { entities, tables }
    }
}
