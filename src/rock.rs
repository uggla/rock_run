use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{ExternalImpulse, GravityScale, RigidBody, Velocity},
    geometry::{ActiveCollisionTypes, Collider},
};

use crate::{
    collision::CollisionSet,
    coregame::state::AppState,
    events::{PositionSensorCollision, Restart},
};

pub const ROCK_SCALE_FACTOR: f32 = 1.0;
pub const ROCK_DIAMETER: f32 = 64.0;

#[derive(Component)]
pub struct Rock;

#[derive(Component, Deref, DerefMut)]
pub struct ChaseTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum RockMovement {
    Run(RockDirection),
    Crunch,
}

impl Default for RockMovement {
    fn default() -> Self {
        Self::Run(RockDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum RockDirection {
    Left,
    #[default]
    Right,
}

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_rock)
            .add_systems(
                Update,
                (spawn_rock, despawn_rock_on_restart)
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = vec![(
        // body
        Vec2::new(0.0, 0.0),
        0.0,
        Collider::ball(ROCK_DIAMETER / 2.0),
    )];

    if y_mirror {
        shapes
            .into_iter()
            .map(|(pos, angle, shape)| (pos * Vec2::new(-1.0, 1.0), angle, shape))
            .collect()
    } else {
        shapes
    }
}

fn spawn_rock(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rock_sensor_collision: EventReader<PositionSensorCollision>,
) {
    for collision_event in rock_sensor_collision.read() {
        if !collision_event.sensor_name.contains("rock") {
            return;
        }

        let texture = asset_server.load("rock_ball.png");

        commands
            .spawn((
                SpriteSheetBundle {
                    texture,
                    sprite: Sprite { ..default() },
                    transform: Transform {
                        scale: Vec3::splat(ROCK_SCALE_FACTOR),
                        translation: collision_event.spawn_pos.extend(20.0),
                        ..default()
                    },
                    ..default()
                },
                RigidBody::Dynamic,
                GravityScale(20.0),
                Velocity::zero(),
                Collider::compound(get_collider_shapes(false)),
                ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                Rock,
            ))
            .insert(ExternalImpulse {
                impulse: Vec2::new(-4096.0 * 100.0, 0.0),
                ..default()
            });
    }
}

fn despawn_rock(mut commands: Commands, rocks: Query<Entity, With<Rock>>) {
    for rock in rocks.iter() {
        commands.entity(rock).despawn_recursive();
    }
}

fn despawn_rock_on_restart(
    mut commands: Commands,
    rocks: Query<Entity, With<Rock>>,
    restart_event: EventReader<Restart>,
) {
    if restart_event.is_empty() {
        return;
    }

    for rock in rocks.iter() {
        commands.entity(rock).despawn_recursive();
    }
}
