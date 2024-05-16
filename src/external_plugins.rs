use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_fluent::FluentPlugin;
use bevy_rapier2d::prelude::*;
pub struct ExternalPlugins;

impl PluginGroup for ExternalPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group
            .add(TilemapPlugin)
            .add(FluentPlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(60.0));

        #[cfg(debug_assertions)]
        {
            group = group.add(RapierDebugRenderPlugin::default());
        }

        group
    }
}
