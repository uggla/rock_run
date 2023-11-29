mod player;

use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::window::WindowResolution;
use player::{
    move_player, setup_player, Player, PlayerMovement, PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH,
    PLAYER_SCALE_FACTOR, PLAYER_SPEED,
};

// 2/3 of 1080p
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 920.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy".to_string(),
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        ..default()
                    }),
                    ..default()
                })
                // prevents blurry sprites
                .set(ImagePlugin::default_nearest()),
        )
        .add_state::<PlayerMovement>()
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                check_for_collisions,
                gravity.after(check_for_collisions),
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Collider;

const GROUND_SIZE: Vec2 = Vec2::new(200.0, 10.0);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    setup_player(&mut commands, asset_server, texture_atlases);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(GROUND_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -300.0, 0.0),
            ..default()
        },
        Ground,
        Collider,
    ));
}

#[derive(Event, Default)]
#[allow(dead_code)]
struct CollisionEvent {
    x: f32,
    y: f32,
}

fn check_for_collisions(
    player_query: Query<&Transform, With<Player>>,

    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let player_transform = player_query.single();
    let player_size = Vec2::new(
        PLAYER_HITBOX_WIDTH * PLAYER_SCALE_FACTOR,
        PLAYER_HITBOX_HEIGHT * PLAYER_SCALE_FACTOR,
    );
    for (_collider_entity, transform) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            Vec2::new(200.0, 10.0),
        );
        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send(CollisionEvent {
                x: transform.translation.x,
                y: transform.translation.y,
            });

            match collision {
                Collision::Left => println!("Left collision"),
                Collision::Right => println!("Right collision"),
                Collision::Top => println!("Top collision"),
                Collision::Bottom => println!("Bottom collision"),
                Collision::Inside => println!("Inside collision"),
            }
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
