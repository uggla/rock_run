mod bat;
mod colliders;
mod collision;
mod coregame;
mod enigma;
mod events;
mod external_plugins;
mod helpers;
mod life;
mod localization;
mod moving_platform;
mod player;
mod rock;
mod screen_map;
mod text_syllable;
mod triceratops;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use text_syllable::TextSyllablePlugin;

use crate::{
    bat::BatPlugin,
    colliders::GroundAndPlatformsPlugin,
    collision::CollisionPlugin,
    coregame::{plugins::CoreGamePlugins, state::AppState},
    enigma::EnigmaPlugin,
    events::{NoMoreStoryMessages, StoryMessages},
    external_plugins::ExternalPlugins,
    life::LifePlugin,
    localization::LocalizationPlugin,
    moving_platform::MovingPlatformPlugin,
    player::PlayerPlugin,
    rock::RockPlugin,
    triceratops::TriceratopsPlugin,
};

// 16/9 1280x720
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::new();
    app.init_state::<AppState>()
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
            TriceratopsPlugin,
            BatPlugin,
            RockPlugin,
            LifePlugin,
            MovingPlatformPlugin,
            CollisionPlugin,
            LocalizationPlugin,
            TextSyllablePlugin::default(),
            EnigmaPlugin,
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
        .add_event::<NoMoreStoryMessages>();

    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    app.add_systems(
        Update,
        toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
    );

    app.run()
}

#[allow(dead_code)]
fn toggle_perf_ui(
    mut commands: Commands,
    q_root: Query<Entity, With<iyes_perf_ui::PerfUiRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn(iyes_perf_ui::PerfUiCompleteBundle::default());
        }
    }
}

// TODO: remove as this is for debugging purpose
#[allow(unused)]
fn update_text(
    mut msg_event: EventWriter<StoryMessages>,
    // mut life_event: EventWriter<events::LifeEvent>,
    input: Query<
        &leafwing_input_manager::action_state::ActionState<player::PlayerMovement>,
        With<player::Player>,
    >,
    mut ext_impulses: Query<&mut bevy_rapier2d::dynamics::ExternalImpulse, With<rock::Rock>>,
) {
    let input_state = match input.get_single() {
        Ok(state) => state,
        Err(_) => return,
    };

    if input_state.just_pressed(&player::PlayerMovement::Crouch) {
        debug!("open window to display messages");
        // for mut ext_impulse in ext_impulses.iter_mut() {
        //     ext_impulse.impulse = Vec2::new(-100.0, 0.0);
        // }
        //     msg_event.send(StoryMessages::Display(vec![
        //         (
        //             "hello-world".to_string(),
        //             Some(HashMap::from([("name".to_string(), "Rose".to_string())])),
        //         ),
        //         ("story01-01".to_string(), None),
        //     ]));

        // life_event.send(events::LifeEvent::Lost);
    }
}
