mod helpers;
mod player;
mod text_syllable;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use player::{move_player, setup_player, PlayerMovement};

use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use text_syllable::TextSyllablePlugin;

use crate::{
    helpers::tiled::{TiledMap, TilesetLayerToStorageEntity},
    text_syllable::{TextSyllableState, TextSyllableValues},
};

// 16/9 1280x720
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RockRun: Rose's Odyssey".to_string(),
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        ..default()
                    }),
                    ..default()
                })
                // prevents blurry sprites
                .set(ImagePlugin::default_nearest()),
            TilemapPlugin,
            helpers::tiled::TiledMapPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(60.0),
            RapierDebugRenderPlugin::default(),
            TextSyllablePlugin::default(),
        ))
        .init_state::<PlayerMovement>()
        .insert_resource(Levels::default())
        .add_systems(Startup, (setup_background, setup_player, setup_physics))
        .add_systems(
            Update,
            (
                move_player,
                read_result_system,
                apply_forces,
                print_ball_altitude,
                bevy::window::close_on_esc,
                helpers::camera::movement,
                update_text,
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct Levels {
    menu: Option<Handle<helpers::tiled::TiledMap>>,
}

#[derive(Component)]
struct Truc;

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut levels: ResMut<Levels>,
) {
    commands.spawn(Camera2dBundle::default());

    // let menu: Handle<helpers::tiled::TiledMap> = asset_server.load("menu.tmx");
    // levels.menu = Some(menu);

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("level01.tmx");
    commands.spawn((
        helpers::tiled::TiledMapBundle {
            tiled_map: map_handle,
            ..Default::default()
        },
        Truc,
    ));
}

fn read_result_system(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput)>,
    state: Res<State<PlayerMovement>>,
    mut next_state: ResMut<NextState<PlayerMovement>>,
) {
    for (entity, output) in controllers.iter() {
        // info!(
        //     "Entity {:?} moved by {:?} and touches the ground: {:?}",
        //     entity, output.effective_translation, output.grounded
        // );

        if output.grounded && state.get() != &PlayerMovement::Jump {
            next_state.set(PlayerMovement::Idle);
        }
    }
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    let points = vec![
        Vec2::new(-640.0, -8.0 - 224.0),
        Vec2::new(640.0, -8.0 - 224.0),
    ];

    commands
        .spawn(Collider::polyline(points, None))
        // .spawn(Collider::cuboid(500.0, 50.0))
        .insert(Ccd::enabled());
    // .insert(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

    /* Create the bouncing ball. */
    commands
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
    state: ResMut<State<PlayerMovement>>,
) {
    // Apply impulses.
    if keyboard_input.pressed(KeyCode::ArrowUp) && state.get() != &PlayerMovement::Jump {
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
    mut commands: Commands,
    mut params: ResMut<TextSyllableValues>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<TextSyllableState>>,
    mut next_state: ResMut<NextState<TextSyllableState>>,
    time: Res<Time>,
    map_handle: Res<Levels>,
    mut map_query: Query<
        (
            Entity,
            &Handle<TiledMap>,
            &mut TilesetLayerToStorageEntity,
            &TilemapRenderSettings,
        ),
        With<Truc>,
    >,

    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut tile_query: Query<&mut TileVisible>,
) {
    if state.get() == &TextSyllableState::Hidden && keyboard_input.pressed(KeyCode::Space) {
        next_state.set(TextSyllableState::Visible);
        // if let Some(map_handle) = map_handle.menu.as_ref() {
        info!("here");
        // commands.spawn((helpers::tiled::TiledMapBundle {
        //     tiled_map: map_handle.clone(),
        //     ..Default::default()
        // },));

        // let (map_handle, mut layer_storage, render_settings) = map_query.single_mut();
        for (e, map_handle, mut tileset_layer_storage, render_settings) in map_query.iter_mut() {
            for layer_entity in tileset_layer_storage.get_entities() {
                if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                    for tile in layer_tile_storage.iter().flatten() {
                        commands.entity(*tile).despawn_recursive()
                    }
                }
                commands.entity(*layer_entity).despawn_recursive();
            }
            commands.entity(e).despawn_recursive();
        }
        //
        // for ts in tile_storage_query.iter() {
        //     info!("ent: {:?}", ts);
        // }
        // for mut t in tile_query.iter_mut() {
        //     t.0 = false;
        // }
        // }
    }

    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::Enter) {
        params.text = "bi-du-le".into();
    }
    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::Backspace) {
        next_state.set(TextSyllableState::Hidden);
    }
}
