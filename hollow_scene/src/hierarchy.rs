//! Write/Read a bevy hierarchy into a slice

use std::marker::PhantomData;

use ::bevy::ecs::query::{ROQueryItem, WorldQuery};
use ::bevy::ecs::world::EntityMut;
use ::bevy::prelude::{BuildWorldChildren, QueryState};
use bevy::prelude as bevy;
use rkyv::Archived;

use crate::entity::{Entity, InlineStorage, Inlines, Keys, TableStorage, Tables};

pub struct Spawn<'ett, 'b: 'ett + 't, 't, Ts: Tables + 'b, Is: Inlines + 'b> {
    scene: &'ett [Archived<Entity<Ts::Keys, Is>>],
    tables: &'t Archived<TableStorage<Ts>>,
    _b: PhantomData<&'b ()>,
}
impl<'ett, 'b: 'ett, 't, Ts: Tables, Is: Inlines> Spawn<'ett, 'b, 't, Ts, Is> {
    pub const fn new(
        scene: &'ett [Archived<Entity<Ts::Keys, Is>>],
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
    fn next(&mut self) -> Option<&'ett Archived<Entity<Ts::Keys, Is>>> {
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
            entity.inline_items.spawn(&mut bevy_entity);

            let descendant_count = entity.children;

            let spawn = self.extract_children(descendant_count);
            spawn.children_of(bevy_entity);
        });
    }
}
type BuildQuery<Ks, Is> = (
    Option<&'static bevy::Children>,
    <Ks as Keys>::Query,
    <Is as Inlines>::Query,
);

pub fn build<Ts: Tables, Is: Inlines>(
    world: &mut bevy::World,
    tables: &mut TableStorage<Ts>,
) -> Box<[Entity<Ts::Keys, Is>]> {
    let mut root_query =
        world.query_filtered::<BuildQuery<Ts::Keys, Is>, bevy::Without<bevy::Parent>>();
    root_query.update_archetypes(world);
    let mut child_query = world.query::<BuildQuery<Ts::Keys, Is>>();
    child_query.update_archetypes(world);

    let entity_count = child_query.iter_manual(world).len();
    let mut entities: Vec<_> = Vec::with_capacity(entity_count + 1);
    entities.push(Entity::with_children(entity_count as u32));

    let grand_children = root_query
        .iter_manual(world)
        .map(|item| child(item, &child_query, &mut entities, tables, world))
        .sum::<u32>();

    entities[0].children += grand_children;
    entities.into_boxed_slice()
}
// TODO(clean) there is too many arguments to this function
fn child<Ts: Tables, Is: Inlines>(
    (children, table_query, inline_query): ROQueryItem<BuildQuery<Ts::Keys, Is>>,
    query: &QueryState<BuildQuery<Ts::Keys, Is>>,
    uninit: &mut Vec<Entity<Ts::Keys, Is>>,
    tables: &mut TableStorage<Ts>,
    world: &bevy::World,
) -> u32 {
    let child_count = children.map_or(0, |c| c.len());

    let inserted_index = uninit.len();
    uninit.push(Entity {
        children: child_count as u32,
        inline_items: InlineStorage::query(inline_query),
        ref_table_keys: tables.insert_values(table_query),
    });
    let grand_children = IterChildren::new(children, query, world)
        .map(|item| child(item, query, uninit, tables, world))
        .sum::<u32>();

    uninit[inserted_index].children += grand_children;
    uninit[inserted_index].children
}
struct IterChildren<'chld, 'q, 'w, Q: WorldQuery> {
    entities: &'chld [bevy::Entity],
    query: &'q QueryState<Q>,
    world: &'w bevy::World,
}
impl<'chld, 'q, 'w, Q: WorldQuery> IterChildren<'chld, 'q, 'w, Q> {
    fn new(
        children: Option<&'chld bevy::Children>,
        query: &'q QueryState<Q>,
        world: &'w bevy::World,
    ) -> Self {
        IterChildren {
            entities: children.map_or(&[], |c| c),
            query,
            world,
        }
    }
}
impl<'chld, 'q, 'w, Q: WorldQuery> Iterator for IterChildren<'chld, 'q, 'w, Q> {
    type Item = ROQueryItem<'w, Q>;

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, tail) = self.entities.split_first()?;
        self.entities = tail;

        Some(self.query.get_manual(self.world, *entity).unwrap())
    }
}
