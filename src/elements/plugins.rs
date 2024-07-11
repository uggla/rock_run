use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::elements::{enigma, moving_platform, rock, story};

pub struct ElementsPlugins;

impl PluginGroup for ElementsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(enigma::EnigmaPlugin)
            .add(moving_platform::MovingPlatformPlugin)
            .add(rock::RockPlugin)
            .add(story::StoryPlugin::default())
    }
}
