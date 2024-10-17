use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::beasts::{bat, monkey, pterodactyl, squirel, trex, triceratops};

pub struct BeastsPlugins;

impl PluginGroup for BeastsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bat::BatPlugin)
            .add(pterodactyl::PterodactylPlugin)
            .add(triceratops::TriceratopsPlugin)
            .add(trex::TrexPlugin)
            .add(squirel::SquirelPlugin)
            .add(monkey::MonkeyPlugin)
    }
}
