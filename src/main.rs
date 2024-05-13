mod camera;
mod collision;
mod ground_platforms;
mod helpers;
mod level;
mod menu;
mod player;
mod screen_map;
mod state;
mod text_syllable;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use text_syllable::TextSyllablePlugin;

use crate::{
    camera::CameraPlugin,
    collision::CollisionPlugin,
    ground_platforms::GroundAndPlatformsPlugin,
    level::LevelPlugin,
    menu::MenuPlugin,
    player::PlayerPlugin,
    state::{AppState, StatesPlugin},
    text_syllable::{TextSyllableState, TextSyllableValues},
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
        .run();
}

fn update_text(
    // mut commands: Commands,
    mut params: ResMut<TextSyllableValues>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<TextSyllableState>>,
    mut next_state: ResMut<NextState<TextSyllableState>>,
    // time: Res<Time>,
    // mut map_query: Query<
    //     (
    //         Entity,
    //         &Handle<TiledMap>,
    //         &mut TilesetLayerToStorageEntity,
    //         &TilemapRenderSettings,
    //     ),
    //     With<Level>,
    // >,
    // tile_storage_query: Query<(Entity, &TileStorage)>,
    // mut tile_query: Query<&mut TileVisible>,
) {
    if state.get() == &TextSyllableState::Hidden && keyboard_input.pressed(KeyCode::Space) {
        next_state.set(TextSyllableState::Visible);
        // for (e, map_handle, mut tileset_layer_storage, render_settings) in map_query.iter_mut() {
        //     for layer_entity in tileset_layer_storage.get_entities() {
        //         if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
        //             for tile in layer_tile_storage.iter().flatten() {
        //                 commands.entity(*tile).despawn_recursive()
        //             }
        //         }
        //         commands.entity(*layer_entity).despawn_recursive();
        //     }
        //     commands.entity(e).despawn_recursive();
        // }
    }

    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::KeyN) {
        params.text = "bi-du-le".into();
    }

    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::Backspace) {
        next_state.set(TextSyllableState::Hidden);
    }
}
