//! Write/Read a bevy hierarchy into a slice

use std::iter;
use std::marker::PhantomData;

use ::bevy::ecs::query::ROQueryItem;
use ::bevy::ecs::world::EntityMut;
use ::bevy::prelude::{BuildWorldChildren, QueryState};
use bevy::prelude as bevy;

use crate::entity::{KeyStorage, Keys, TableStorage, Tables};
use crate::{entity::Entity, Archived};

pub struct Spawn<'ett, 'b: 'ett + 't, 't, Ts: Tables + 'b> {
    scene: &'ett [Archived<Entity<Ts::Keys>>],
    tables: &'t Archived<TableStorage<Ts>>,
    _b: PhantomData<&'b ()>,
}
impl<'ett, 'b: 'ett, 't, Ts: Tables> Spawn<'ett, 'b, 't, Ts> {
    pub const fn new(
        scene: &'ett [Archived<Entity<Ts::Keys>>],
        tables: &'t Archived<TableStorage<Ts>>,
    ) -> Self {
        Spawn { scene, _b: PhantomData, tables }
    }
    /// Split this `Spawn` in two.
    ///
    /// Assuming this is a list of entities and `descendants` is how many
    /// children the current entity has (accumulated recursively), we have:
    ///
    /// - `&mut self` becomes all remaining entities to spawn.
    /// - the return value contains all children entities to spawn.
    fn extract_children(&mut self, descendants: u32) -> Self {
        let (childr, scene) = self.scene.split_at(descendants as usize);
        self.scene = scene;
        Spawn { scene: childr, ..*self }
    }
    fn next(&mut self) -> Option<&'ett Archived<Entity<Ts::Keys>>> {
        let (entity, scene) = self.scene.split_first()?;
        self.scene = scene;
        Some(entity)
    }
    pub fn children_of(mut self, mut spawner: EntityMut) {
        spawner.with_children(|spawner| loop {
            let Some(entity) = self.next() else { return; };

            let mut bevy_entity = spawner.spawn_empty();

            self.tables
                .spawn_keys(&entity.ref_table_keys, &mut bevy_entity);

            let descendant_count = entity.children;

            let spawn = self.extract_children(descendant_count);
            spawn.children_of(bevy_entity);
        });
    }
}
type BuildQuery<Ks> = (Option<&'static bevy::Children>, <Ks as Keys>::Query);

pub fn build<Ts: Tables>(
    world: &mut bevy::World,
    tables: &mut TableStorage<Ts>,
) -> Box<[Entity<Ts::Keys>]> {
    let root_query = world.query_filtered::<BuildQuery<Ts::Keys>, bevy::Without<bevy::Parent>>();
    let child_query = world.query::<BuildQuery<Ts::Keys>>();

    let entity_count = child_query.iter_manual(world).len();
    let mut entities: Box<[_]> = iter::repeat_with(Entity::empty)
        .take(entity_count + 1)
        .collect();

    let entity_count = entities.len();
    let (entity, uninit) = entities.split_first_mut().unwrap();

    *entity = Entity {
        children: entity_count as u32,
        ref_table_keys: KeyStorage::no_component(),
    };
    let root_query = root_query.iter_manual(world);

    entity.children += root_query
        .map(|item| child(item, &child_query, uninit, tables, world))
        .sum::<u32>();

    assert_eq!(entity_count as u32, entity.children + 1);

    entities
}
// TODO(clean) there is too many arguments to this function
fn child<Ts: Tables>(
    (children, components): ROQueryItem<BuildQuery<Ts::Keys>>,
    query: &QueryState<BuildQuery<Ts::Keys>>,
    uninit: &mut [Entity<Ts::Keys>],
    tables: &mut TableStorage<Ts>,
    world: &bevy::World,
) -> u32 {
    // TODO(err) unwrap (technically the unwrap should never occur)
    let (entity, uninit) = uninit.split_first_mut().unwrap();

    let child_count = children.map_or(0, |c| c.len());

    *entity = Entity {
        children: child_count as u32,
        ref_table_keys: tables.insert_values(components),
    };
    entity.children += IterChildren::<Ts::Keys>::new(children, query, world)
        .map(|item| child(item, query, uninit, tables, world))
        .sum::<u32>();

    entity.children
}
struct IterChildren<'chld, 'q, 'w, Ks: Keys> {
    entities: &'chld [bevy::Entity],
    query: &'q QueryState<BuildQuery<Ks>>,
    world: &'w bevy::World,
}
impl<'chld, 'q, 'w, Ks: Keys> IterChildren<'chld, 'q, 'w, Ks> {
    fn new(
        children: Option<&'chld bevy::Children>,
        query: &'q QueryState<BuildQuery<Ks>>,
        world: &'w bevy::World,
    ) -> Self {
        IterChildren {
            entities: children.map_or(&[], |c| c),
            query,
            world,
        }
    }
}
impl<'chld, 'q, 'w, Ks: Keys> Iterator for IterChildren<'chld, 'q, 'w, Ks> {
    type Item = ROQueryItem<'w, BuildQuery<Ks>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, tail) = self.entities.split_first()?;
        self.entities = tail;

        Some(self.query.get_manual(&self.world, *entity).unwrap())
    }
}
