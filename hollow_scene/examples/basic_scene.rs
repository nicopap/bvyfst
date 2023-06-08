use bvyfst_hollow_scene::{proxy, Archive, ArchiveProxy, Deserialize, Plugin, Serialize};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin::processed_dev()))
        .register_type::<ComponentA>()
        .register_type::<ComponentB>()
        .add_plugin(Plugin!(
            Inline[proxy::Id<ComponentA>, MyTransform]
            Table[proxy::Id<ComponentB>]
            Extras[]
        ))
        .add_systems(Startup, (load_scene_system, infotext_system))
        .add_systems(Update, log_system)
        .run();
}

#[derive(Clone, Copy, Default, Archive, Deserialize, Serialize)]
pub struct MyTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
impl ArchiveProxy for MyTransform {
    type Target = Transform;

    fn to_target(value: &Self::Archived) -> Self::Target {
        Transform {
            translation: value.translation.into(),
            rotation: Quat::from_array(value.rotation),
            scale: value.scale.into(),
        }
    }
    fn from_target(bevy: &Self::Target) -> Self {
        MyTransform {
            translation: bevy.translation.into(),
            rotation: bevy.rotation.into(),
            scale: bevy.scale.into(),
        }
    }
}

#[derive(Component, Clone, Reflect, Default, Archive, Deserialize, Serialize)]
#[reflect(Component)]
struct ComponentA {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, Reflect, Default, Archive, Deserialize, Serialize)]
#[reflect(Component)]
struct ComponentB {
    pub value: String,
}

const SCENE_FILE_PATH: &str = "load_scene_example.myscn.ron";

fn load_scene_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load(SCENE_FILE_PATH),
        ..default()
    });
}

fn log_system(query: Query<(Entity, &ComponentA), Changed<ComponentA>>) {
    for (entity, component_a) in &query {
        info!("  Entity({})", entity.index());
        info!(
            "    ComponentA: {{ x: {} y: {} }}\n",
            component_a.x, component_a.y
        );
    }
}

fn infotext_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(
        TextBundle::from_section(
            "Nothing to see in this window! Check the console output!",
            TextStyle { font_size: 50.0, color: Color::WHITE, ..default() },
        )
        .with_style(Style { align_self: AlignSelf::FlexEnd, ..default() }),
    );
}
