use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    entity::{Entity, Inlines, TableStorage, Tables},
    hierarchy::{self, Spawn},
};

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct FastScene<Ts: Tables, Is: Inlines> {
    pub entities: Box<[Entity<Ts::Keys, Is>]>,
    pub tables: TableStorage<Ts>,
}
impl<Ts: Tables, Is: Inlines> ArchivedFastScene<Ts, Is> {
    pub fn to_bevy(&self) -> bevy::Scene {
        let mut world = bevy::World::new();

        let root_entity = world.spawn_empty();
        let spawn = Spawn::new(&self.entities, &self.tables);
        spawn.children_of(root_entity);

        bevy::Scene::new(world)
    }
}
impl<Ts: Tables, Is: Inlines> FastScene<Ts, Is> {
    pub fn from_bevy(scene: &mut bevy::Scene) -> Self {
        let mut tables = TableStorage::new();
        let entities = hierarchy::build(&mut scene.world, &mut tables);
        FastScene { entities, tables }
    }
}

#[cfg(test)]
mod tests {
    use super::FastScene;
    use crate::{proxy::Id, Archive, Deserialize, Serialize};
    use std::fmt::Write;

    macro_rules! inline {
        () => { () };
        ($head:ty, $($tail:ty,)*) => {
            ($crate::Inline<$head>, inline!($($tail,)*) )
        };
    }
    macro_rules! table {
        () => { () };
        ($head:ty, $($tail:ty,)*) => {
            ($crate::Table<$head>, table!($($tail,)*) )
        };
    }
    macro_rules! make_world {
        (@branch $( [  $( $comp:expr ),*  ] )*) => {{
            let mut world = World::new();
            $( world.spawn( ( $( $comp ),* ) ); )*
            world
        }};
        ($( $any:tt )*) => { (
            make_world!(@branch $($any)*),
            make_world!(@branch $($any)*),
        ) }
    }
    use bevy::prelude::*;

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct A1;

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct B1;

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct C1(u32);

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct A2;

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct B2;

    #[rustfmt::skip]
    #[derive(
        Component,
        Debug, Default, Clone,
        PartialEq, PartialOrd, Eq, Ord,
        Archive, Serialize, Deserialize,
    )]
    struct C2(u32);

    type Tables = table![Id<A1>, Id<B1>, Id<C1>,];
    type Inlines = inline![Id<A2>, Id<B2>, Id<C2>,];

    #[test]
    fn roundtrip_just_entities() {
        let (mut old_world, world) = make_world![
            [A1]
            [A2, B2, C2(7)]
            [A1, A2]
            [C2(5), C1(6)]
            [A1, A2, B1, B2, C1(3), C2(3)]
            [A1, B1]
            [A2, B2]
            [A1, B1, C1(2)]
            [A1, C1(0)]
            // [C1(1)]
            [C1(3), C2(4)]
            []
        ];

        let fast_scene = FastScene::<Tables, Inlines>::from_bevy(&mut Scene::new(world));
        println!("created scene\nsize: {}", fast_scene.entities.len());
        for i in 0..fast_scene.tables.component_count() {
            let name = fast_scene.tables.component_name(i);
            let count = fast_scene.tables.component_count_of(i);
            println!("\t{name}: {count}")
        }
        println!(
            "Entity inline components: {}",
            std::any::type_name::<Inlines>()
        );
        for entity in &*fast_scene.entities {
            println!(
                "\tinline {:?} || keys: {:?}",
                entity.inline_items.occupancy(),
                entity.ref_table_keys.occupancy()
            );
        }

        let bytes = rkyv::to_bytes::<_, 0>(&fast_scene).unwrap();
        let archived = unsafe { rkyv::archived_root::<FastScene<Tables, Inlines>>(&bytes) };
        let mut new_world = archived.to_bevy().world;

        let root = new_world
            .query_filtered::<Entity, Without<Parent>>()
            .single(&new_world);

        new_world.entity_mut(root).despawn();
        println!("new_world entities: {}", new_world.entities().len());

        let mut new_entities = new_world
            .query::<AnyOf<(&A1, &B1, &C1, &A2, &B2, &C2)>>()
            .iter(&new_world)
            .collect::<Vec<_>>();

        let mut old_entities = old_world
            .query::<AnyOf<(&A1, &B1, &C1, &A2, &B2, &C2)>>()
            .iter(&old_world)
            .collect::<Vec<_>>();

        new_entities.sort();
        old_entities.sort();

        let new_printed = new_entities.iter().fold(String::new(), |mut acc, value| {
            writeln!(&mut acc, "{value:?}").unwrap();
            acc
        });
        let old_printed = old_entities.iter().fold(String::new(), |mut acc, value| {
            writeln!(&mut acc, "{value:?}").unwrap();
            acc
        });

        assert_ne!(
            old_entities, new_entities,
            "\n==== old ====\n{old_printed}\n==== new ====\n{new_printed}"
        );
    }
}
