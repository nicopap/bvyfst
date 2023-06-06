//! Write/Read a bevy hierarchy into a slice

use std::mem::{self, MaybeUninit};

use ::bevy::ecs::query::QueryItem;
use ::bevy::ecs::world::EntityMut;
use ::bevy::prelude::{BuildWorldChildren, QueryState, Scene, SpatialBundle};
use ::bevy::utils::HashMap;
use bevy::prelude as bevy;

use crate::{fast, Archived};

pub type Mesh = bevy::Handle<bevy::Mesh>;
pub type Mat = bevy::Handle<bevy::StandardMaterial>;

pub struct Spawn<'a> {
    scene: &'a [Archived<fast::Entity>],
}
impl<'a> Spawn<'a> {
    pub const fn new(scene: &'a [Archived<fast::Entity>]) -> Self {
        Spawn { scene }
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
        Spawn { scene: childr }
    }
    fn next(&mut self) -> Option<&Archived<fast::Entity>> {
        let (entity, scene) = self.scene.split_first()?;
        self.scene = scene;
        Some(entity)
    }
    pub fn as_children_of(mut self, mut spawner: EntityMut, meshes: &[Mesh], mats: &[Mat]) {
        spawner.with_children(|spawner| loop {
            let Some(entity) = self.next() else { return; };

            let bundle = SpatialBundle::from_transform(entity.transform.to_bevy());
            let mut bevy_entity = spawner.spawn(bundle);

            if let Some(mesh) = entity.mesh.as_ref() {
                bevy_entity.insert(unsafe { mesh.pick(meshes) }.clone());
            }
            if let Some(mat) = entity.material.as_ref() {
                bevy_entity.insert(unsafe { mat.pick(mats) }.clone());
            }
            let descendant_count = entity.children;

            let spawn = self.extract_children(descendant_count);
            spawn.as_children_of(bevy_entity, meshes, mats);
        });
    }
}
type BuildQuery = (
    Option<&'static Mesh>,
    Option<&'static Mat>,
    Option<&'static bevy::Children>,
    &'static bevy::Transform,
);
pub fn build(scene: &mut Scene) -> Box<[fast::Entity]> {
    let mut init = InitEntity::new();
    init.root(&mut scene.world)
}

struct InitEntity {
    mat_ids: HashMap<Mat, fast::MaterialId>,
    mesh_ids: HashMap<Mesh, fast::MeshId>,
    image_ids: HashMap<bevy::Handle<bevy::Image>, fast::ImageId>,
}
type UninitEntity = MaybeUninit<fast::Entity>;
impl InitEntity {
    fn new() -> Self {
        InitEntity {
            mat_ids: Default::default(),
            mesh_ids: Default::default(),
            image_ids: Default::default(),
        }
    }
    fn root(&mut self, world: &mut bevy::World) -> Box<[fast::Entity]> {
        let root_query = world.query_filtered::<BuildQuery, bevy::Without<bevy::Parent>>();
        let child_query = world.query::<BuildQuery>();

        let entity_count = child_query.iter_manual(world).len();
        let mut entities = vec![MaybeUninit::uninit(); entity_count + 1].into_boxed_slice();

        let (entity, uninit) = entities.split_first_mut().unwrap();

        entity.write(fast::Entity {
            mesh: None,
            material: None,
            children: entity_count as u32,
            transform: fast::Transform::default(),
        });

        let root_query = root_query.iter_manual(world);
        let mut added_count = root_query.len() as u32;
        added_count += root_query
            .map(|item| self.child(item, &child_query, uninit, world))
            .sum::<u32>();
        assert_eq!(entities.len() as u32, added_count + 1);

        // SAFETY: previous line indicates that all entries are initialized
        unsafe { mem::transmute::<Box<[UninitEntity]>, Box<[fast::Entity]>>(entities) }
    }
    fn serialize_mat(&mut self, mat: &Mat) -> fast::MaterialId {
        let mat_count = self.mat_ids.len();
        let mat_id = || fast::MaterialId::new(mat_count as u32);
        *self.mat_ids.entry(mat.clone()).or_insert_with(mat_id)
    }
    fn serialize_mesh(&mut self, mesh: &Mesh) -> fast::MeshId {
        let mesh_count = self.mesh_ids.len();
        let mesh_id = || fast::MeshId::new(mesh_count as u32);
        *self.mesh_ids.entry(mesh.clone()).or_insert_with(mesh_id)
    }
    fn serialize_image(&mut self, image: &bevy::Handle<bevy::Image>) -> fast::ImageId {
        let image_count = self.image_ids.len();
        let image_id = || fast::ImageId::new(image_count as u32);
        *self.image_ids.entry(image.clone()).or_insert_with(image_id)
    }
    // TODO(clean) there is too many arguments to this function
    fn child(
        &mut self,
        (mesh, mat, children, transform): QueryItem<BuildQuery>,
        query: &QueryState<BuildQuery>,
        uninit: &mut [UninitEntity],
        world: &bevy::World,
    ) -> u32 {
        // TODO(err) unwrap
        let (entity, uninit) = uninit.split_first_mut().unwrap();

        let child_count = children.map_or(0, |c| c.len());

        // TODO(clean) this is so wonk, and repeated in `root` and `child`
        entity.write(fast::Entity {
            mesh: mesh.map(|m| self.serialize_mesh(m)),
            material: mat.map(|m| self.serialize_mat(m)),
            children: child_count as u32,
            transform: transform.into(),
        });
        // SAFETY: we literally just initialized this (entity.write)
        let head = &mut unsafe { entity.assume_init_mut() }.children;

        let iter = self.iter_children(children, query, world);

        *head += iter
            .map(|item| self.child(item, query, uninit, world))
            .sum::<u32>();

        *head
    }

    fn iter_children<'chld, 'q, 'w>(
        &self,
        children: Option<&'chld bevy::Children>,
        query: &'q QueryState<BuildQuery>,
        world: &'w bevy::World,
    ) -> IterChildren<'chld, 'q, 'w> {
        IterChildren {
            entities: children.map_or(&[], |c| &*c),
            query,
            world,
        }
    }
}
struct IterChildren<'a, 'b, 'w> {
    entities: &'a [bevy::Entity],
    query: &'b QueryState<BuildQuery>,
    world: &'w bevy::World,
}
impl<'a, 'b, 'w> Iterator for IterChildren<'a, 'b, 'w> {
    type Item = QueryItem<'w, BuildQuery>;

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, tail) = self.entities.split_first()?;
        self.entities = tail;

        Some(self.query.get_manual(&self.world, *entity).unwrap())
    }
}
