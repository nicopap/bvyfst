//! Write/Read a bevy hierarchy into a slice

use std::iter;
use std::marker::PhantomData;

use ::bevy::ecs::query::ROQueryItem;
use ::bevy::ecs::world::EntityMut;
use ::bevy::prelude::{BuildWorldChildren, QueryState};
use bevy::prelude as bevy;
use rkyv::Archive;

use crate::entity::{KeyStorage, Keys, TableStorage, Tables};
use crate::{entity::Entity, Archived};

pub struct Spawn<'ett, 'bfr: 'ett + 'tbl, 'tbl, Ts: Archive + 'bfr, Ks: Archive + 'bfr> {
    scene: &'ett [Archived<Entity<Ks>>],
    tables: &'tbl Archived<TableStorage<Ts>>,
    _b: PhantomData<&'bfr ()>,
}
impl<'ett, 'bfr: 'ett, 'tbl, Ts: Tables<Ks>, Ks: Keys + Archive + 'bfr>
    Spawn<'ett, 'bfr, 'tbl, Ts, Ks>
{
    pub const fn new(
        scene: &'ett [Archived<Entity<Ks>>],
        tables: &'tbl Archived<TableStorage<Ts>>,
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
    fn next(&mut self) -> Option<&'ett Archived<Entity<Ks>>> {
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

pub fn build<Ts: Tables<Ks>, Ks: Keys>(
    world: &mut bevy::World,
    tables: &mut TableStorage<Ts>,
) -> Box<[Entity<Ks>]> {
    let root_query = world.query_filtered::<BuildQuery<Ks>, bevy::Without<bevy::Parent>>();
    let child_query = world.query::<BuildQuery<Ks>>();

    let entity_count = child_query.iter_manual(world).len();
    let mut entities: Box<[_]> = iter::repeat_with(Entity::empty)
        .take(entity_count + 1)
        .collect();

    let (entity, uninit) = entities.split_first_mut().unwrap();

    *entity = Entity {
        children: entity_count as u32,
        ref_table_keys: KeyStorage::no_component(),
    };

    let root_query = root_query.iter_manual(world);
    let mut added_count = root_query.len() as u32;
    added_count += root_query
        .map(|item| child(item, &child_query, uninit, tables, world))
        .sum::<u32>();
    assert_eq!(entities.len() as u32, added_count + 1);

    entities
}
// TODO(clean) there is too many arguments to this function
fn child<Ks: Keys, Ts: Tables<Ks>>(
    (children, components): ROQueryItem<BuildQuery<Ks>>,
    query: &QueryState<BuildQuery<Ks>>,
    uninit: &mut [Entity<Ks>],
    tables: &mut TableStorage<Ts>,
    world: &bevy::World,
) -> u32 {
    // TODO(err) unwrap
    let (entity, uninit) = uninit.split_first_mut().unwrap();

    let child_count = children.map_or(0, |c| c.len());

    // TODO(clean) this is so wonk, and repeated in `root` and `child`
    *entity = Entity {
        children: child_count as u32,
        ref_table_keys: tables.insert_values(components),
    };
    // SAFETY: we literally just initialized this (entity.write)
    let head = &mut entity.children;

    *head += IterChildren::<Ks>::new(children, query, world)
        .map(|item| child(item, query, uninit, tables, world))
        .sum::<u32>();

    *head
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
