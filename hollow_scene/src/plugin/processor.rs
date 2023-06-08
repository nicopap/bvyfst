use bevy::{
    asset::processor::{AssetProcessor, LoadAndSave},
    prelude::{info, FromWorld, World},
};

use super::{loader::Loader, saver::Saver, RkyvTypeNonsense};
use crate::{entity::Inlines, entity::Tables};

type Processor<T, I> = LoadAndSave<Loader<T, I>, Saver<T, I>>;

pub(super) fn insert<Ts: Tables + 'static, Is: Inlines + 'static>(world: &mut World)
where
    Ts::Keys: RkyvTypeNonsense,
    Ts: RkyvTypeNonsense,
    Is: RkyvTypeNonsense,
{
    let saver = Saver::<Ts, Is>::from_world(world);
    let Some(processor) = world.get_resource::<AssetProcessor>() else {
            info!(
                "Your bevy plugin config isn't setup to use asset processing. \
                Scenes won't be saved in the hollow_bvyfst format."
            );
            return;
        };
    info!(
        "Registering processor for plugin: {}",
        std::any::type_name::<Processor<Ts, Is>>()
    );
    processor.register_processor::<Processor<Ts, Is>>(saver.into());

    processor.set_default_processor::<Processor<Ts, Is>>("myscn.ron");
}
