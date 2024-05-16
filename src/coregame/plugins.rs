use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::coregame::{camera, level, menu, state};

pub struct CoreGamePlugins;

impl PluginGroup for CoreGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(state::StatesPlugin)
            .add(camera::CameraPlugin)
            .add(menu::MenuPlugin)
            .add(level::LevelPlugin)
    }
}
