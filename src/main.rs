mod helpers;
mod player;
mod text_syllable;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use player::{
    move_player, setup_player, Player, PlayerMovement, PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH,
    PLAYER_SCALE_FACTOR, PLAYER_SPEED,
};

use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use text_syllable::TextSyllablePlugin;

use crate::text_syllable::{TextSyllableState, TextSyllableValues};

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
        .add_event::<CollisionEvent>()
        .add_systems(
            Startup,
            (setup_background, setup_ground, setup_player, setup_physics),
        )
        .add_systems(
            Update,
            (
                move_player,
                check_for_collisions,
                gravity.after(check_for_collisions),
                apply_forces,
                print_ball_altitude,
                bevy::window::close_on_esc,
                helpers::camera::movement,
                update_text,
            ),
        )
        .run();
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct MyCollider;

const GROUND_SIZE: Vec2 = Vec2::new(200.0, 60.0);

fn setup_ground(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(GROUND_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -300.0, 10.0),
            ..default()
        },
        Ground,
        MyCollider,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(GROUND_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(290.0, -260.0, 10.0),
            ..default()
        },
        Ground,
        MyCollider,
    ));
}

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("level01.tmx");

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

#[derive(Event, Default)]
#[allow(dead_code)]
struct CollisionEvent {
    x: f32,
    y: f32,
}

fn check_for_collisions(
    player_query: Query<&Transform, With<Player>>,

    collider_query: Query<(Entity, &Transform), With<MyCollider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let player_transform = player_query.single();
    let player_size = Vec2::new(
        PLAYER_HITBOX_WIDTH * PLAYER_SCALE_FACTOR,
        PLAYER_HITBOX_HEIGHT * PLAYER_SCALE_FACTOR,
    );
    for (_collider_entity, collider_transform) in &collider_query {
        let player = Aabb2d::new(player_transform.translation.truncate(), player_size);
        let ground = Aabb2d::new(collider_transform.translation.truncate(), GROUND_SIZE);
        let collision = player.intersects(&ground);
        if collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send(CollisionEvent {
                x: collider_transform.translation.x,
                y: collider_transform.translation.y,
            });

            // match collision {
            //     Collision::Left => println!("Left collision"),
            //     Collision::Right => println!("Right collision"),
            //     Collision::Top => println!("Top collision"),
            //     Collision::Bottom => println!("Bottom collision"),
            //     Collision::Inside => println!("Inside collision"),
            // }
        }
    }
}

fn gravity(
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut collision_events: EventReader<CollisionEvent>,
    state: ResMut<State<PlayerMovement>>,
) {
    let mut player_transform = player_query.single_mut();
    match state.get() {
        PlayerMovement::Jump => {}
        _ => {
            if !collision_events.is_empty() {
                for collision_event in collision_events.read() {
                    player_transform.translation.y = collision_event.y
                        + PLAYER_HITBOX_HEIGHT * PLAYER_SCALE_FACTOR / 2.0
                        + GROUND_SIZE.y / 2.0
                        - 1.0; // make the collision
                }
                collision_events.clear();
            } else {
                player_transform.translation.y -= PLAYER_SPEED * time.delta_seconds();
            }
        }
    }
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(500.0, 50.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

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
        info!("Ball altitude: {}", transform.translation.y);
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
    mut params: ResMut<TextSyllableValues>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<TextSyllableState>>,
    mut next_state: ResMut<NextState<TextSyllableState>>,
    time: Res<Time>,
) {
    time.delta_seconds();
    if state.get() == &TextSyllableState::Hidden && keyboard_input.pressed(KeyCode::Space) {
        next_state.set(TextSyllableState::Visible);
    }

    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::Enter) {
        params.text = "bi-du-le".into();
    }
    if state.get() == &TextSyllableState::Visible && keyboard_input.pressed(KeyCode::Backspace) {
        next_state.set(TextSyllableState::Hidden);
    }
}
