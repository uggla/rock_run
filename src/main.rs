mod camera;
mod collision;
mod helpers;
mod level;
mod menu;
mod player;
mod screen_map;
mod state;
mod text_syllable;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use player::PlayerState;

use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use text_syllable::TextSyllablePlugin;

use crate::{
    camera::CameraPlugin,
    collision::CollisionPlugin,
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
            PlayerPlugin,
            CollisionPlugin,
            TextSyllablePlugin::default(),
            #[cfg(debug_assertions)]
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup_physics)
        .add_systems(
            Update,
            (
                apply_forces,
                print_ball_altitude,
                update_text,
                // bevy::window::close_on_esc,
                #[cfg(debug_assertions)]
                helpers::camera::movement,
            ),
        )
        .run();
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Platform;

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    let points = vec![
        Vec2::new(-1280.0, -8.0 - 224.0),
        Vec2::new(48.0, -8.0 - 224.0),
        Vec2::new(48.0 + 14.0 * 16.0, -8.0 - (224.0 - 7.0 * 16.0)),
        Vec2::new(272.0 + 3.0 * 16.0, -8.0 - (224.0 - 7.0 * 16.0)),
        Vec2::new(320.0 + 7.0 * 16.0, -8.0 - (224.0)),
        Vec2::new(1280.0, -8.0 - 224.0),
    ];

    commands
        .spawn((
            SpatialBundle::default(),
            Collider::polyline(points, None),
            Ground,
        ))
        .insert(Ccd::enabled())
        // .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .with_children(|parent| {
            // Create 2 x test platforms
            parent
                .spawn((Collider::cuboid(60.0, 5.0), Platform))
                .insert(TransformBundle::from(Transform::from_xyz(
                    -1280.0 / 2.0 + 100.0,
                    -224.0 + 5.0 * 16.0, // 8.0 is hard to climb, 9.0 can not be climbed
                    0.0,
                )));

            parent
                .spawn((Collider::cuboid(60.0, 5.0), Platform))
                .insert(TransformBundle::from(Transform::from_xyz(
                    -1280.0 / 2.0 + 380.0, // 270 Gap is reachable, 290 seems not
                    -224.0 + 10.0 * 16.0,  // 8.0 is hard to climb, 9.0 can not be climbed
                    0.0,
                )));

            /* Create the bouncing ball. */
            parent
                .spawn(RigidBody::Dynamic)
                .insert(GravityScale(20.0))
                .insert(Collider::ball(20.0))
                .insert(Restitution::coefficient(0.0))
                // .insert(ColliderMassProperties::Density(20.0))
                .insert(ExternalImpulse {
                    // impulse: Vec2::new(100.0, 200.0),
                    // torque_impulse: 14.0,
                    ..default()
                })
                // .insert(Damping {
                //     linear_damping: 100.0,
                //     angular_damping: 0.0,
                // })
                .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
        });
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        debug!("Ball altitude: {}", transform.translation.y);
    }
}

/* Apply forces and impulses inside of a system. */
fn apply_forces(
    mut ext_impulses: Query<&mut ExternalImpulse>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: ResMut<State<PlayerState>>,
) {
    // Apply impulses.
    if keyboard_input.pressed(KeyCode::ArrowUp) && state.get() != &PlayerState::Jumping {
        for mut ext_impulse in ext_impulses.iter_mut() {
            ext_impulse.impulse = Vec2::new(0.0, 250.0);
            // ext_impulse.torque_impulse = 0.4;
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        for mut ext_impulse in ext_impulses.iter_mut() {
            ext_impulse.impulse = Vec2::new(20.0, 0.0);
            // ext_impulse.torque_impulse = 0.4;
        }
    }
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
