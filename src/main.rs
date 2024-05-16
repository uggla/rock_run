mod camera;
mod collision;
mod events;
mod ground_platforms;
mod helpers;
mod level;
mod localization;
mod menu;
mod player;
mod screen_map;
mod state;
mod text_syllable;

use bevy::window::WindowResolution;
use bevy::{prelude::*, utils::HashMap};

use bevy_ecs_tilemap::prelude::*;
use bevy_fluent::FluentPlugin;
use bevy_rapier2d::prelude::*;
use text_syllable::TextSyllablePlugin;

use crate::{
    camera::CameraPlugin,
    collision::CollisionPlugin,
    events::{NoMoreStoryMessages, StoryMessages},
    ground_platforms::GroundAndPlatformsPlugin,
    level::LevelPlugin,
    localization::LocalizationPlugin,
    menu::MenuPlugin,
    player::PlayerPlugin,
    state::{AppState, StatesPlugin},
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
            StatesPlugin,
            CameraPlugin,
            LevelPlugin,
            MenuPlugin,
            TilemapPlugin,
            helpers::tiled::TiledMapPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(60.0),
            GroundAndPlatformsPlugin,
            PlayerPlugin,
            CollisionPlugin,
            FluentPlugin,
            LocalizationPlugin,
            TextSyllablePlugin::default(),
            #[cfg(debug_assertions)]
            RapierDebugRenderPlugin::default(),
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
    }
}
