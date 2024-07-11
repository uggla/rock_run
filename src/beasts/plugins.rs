use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::beasts::bat;

pub struct BeastsPlugins;

impl PluginGroup for BeastsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(bat::BatPlugin)
    }
}
