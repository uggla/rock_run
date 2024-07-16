mod assets;
mod beasts;
mod collisions;
mod coregame;
mod elements;
mod events;
mod external_plugins;
mod helpers;
mod life;
mod music;
mod player;
mod screen_map;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::{
    assets::RockRunAssets,
    beasts::plugins::BeastsPlugins,
    collisions::CollisionsPlugin,
    coregame::{plugins::CoreGamePlugins, state::AppState},
    elements::plugins::ElementsPlugins,
    events::{NoMoreStoryMessages, StoryMessages},
    external_plugins::ExternalPlugins,
    life::LifePlugin,
    music::MusicPlugin,
    player::PlayerPlugin,
};

use bevy_asset_loader::prelude::*;

// 16/9 1280x720
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::new();
    app.add_plugins((
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
        BeastsPlugins,
        ElementsPlugins,
        helpers::tiled::TiledMapPlugin,
        MusicPlugin,
        PlayerPlugin,
        LifePlugin,
        CollisionsPlugin,
    ))
    // with 0.14, init_state needs to be declared after plugins
    // https://github.com/bevyengine/bevy/issues/14154
    .init_state::<AppState>()
    .add_loading_state(
        LoadingState::new(AppState::Loading)
            .continue_to_state(AppState::StartMenu)
            .load_collection::<RockRunAssets>(),
    )
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
    .add_event::<NoMoreStoryMessages>();

    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    app.add_systems(
        Update,
        toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
    );

    app.run();
}

#[allow(dead_code)]
fn toggle_perf_ui(
    mut commands: Commands,
    q_root: Query<Entity, With<iyes_perf_ui::ui::root::PerfUiRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn(iyes_perf_ui::prelude::PerfUiCompleteBundle::default());
        }
    }
}

// TODO: remove as this is for debugging purpose
#[allow(unused)]
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
        debug!("debugging you press crouch(down array key)");
    }
}
