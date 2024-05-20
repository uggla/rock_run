mod colliders;
mod collision;
mod coregame;
mod events;
mod external_plugins;
mod helpers;
mod life;
mod localization;
mod player;
mod screen_map;
mod text_syllable;

use bevy::window::WindowResolution;
use bevy::{prelude::*, utils::HashMap};

use text_syllable::TextSyllablePlugin;

use crate::{
    colliders::GroundAndPlatformsPlugin,
    collision::CollisionPlugin,
    coregame::{plugins::CoreGamePlugins, state::AppState},
    events::{NoMoreStoryMessages, StoryMessages},
    external_plugins::ExternalPlugins,
    life::LifePlugin,
    localization::LocalizationPlugin,
    player::PlayerPlugin,
};

// 16/9 1280x720
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

fn main() {
    App::new()
        .init_state::<AppState>()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RockRun: Rose's Odyssey".to_string(),
                        resizable: false,
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        ..default()
                    }),
                    ..default()
                })
                // prevents blurry sprites
                .set(ImagePlugin::default_nearest()),
            CoreGamePlugins,
            ExternalPlugins,
            helpers::tiled::TiledMapPlugin,
            GroundAndPlatformsPlugin,
            PlayerPlugin,
            LifePlugin,
            CollisionPlugin,
            LocalizationPlugin,
            TextSyllablePlugin::default(),
        ))
        .add_systems(
            Update,
            (
                update_text,
                // bevy::window::close_on_esc,
                #[cfg(debug_assertions)]
                helpers::camera::movement,
            ),
        )
        .add_event::<StoryMessages>()
        .add_event::<NoMoreStoryMessages>()
        .run();
}

// TODO: remove as this is for debugging purpose
fn update_text(
    mut msg_event: EventWriter<StoryMessages>,
    mut life_event: EventWriter<events::LifeEvent>,
    input: Query<
        &leafwing_input_manager::action_state::ActionState<player::PlayerMovement>,
        With<player::Player>,
    >,
) {
    let input_state = match input.get_single() {
        Ok(state) => state,
        Err(_) => return,
    };

    if input_state.just_pressed(&player::PlayerMovement::Crouch) {
        debug!("open window to display messages");
        msg_event.send(StoryMessages::Display(vec![
            (
                "hello-world".to_string(),
                Some(HashMap::from([("name".to_string(), "Rose".to_string())])),
            ),
            ("story01".to_string(), None),
        ]));

        life_event.send(events::LifeEvent::Lost);
    }
}
