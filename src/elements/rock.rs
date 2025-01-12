use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, ExternalImpulse, GravityScale, RigidBody, Velocity},
    geometry::{ActiveCollisionTypes, Collider},
    prelude::Damping,
};
use rand::{thread_rng, Rng};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{PositionSensorCollisionStart, Restart, SmallRockAboutToRelease},
};

use super::volcano::Lava;

pub const ROCK_SCALE_FACTOR: f32 = 1.0;
pub const ROCK_DIAMETER: f32 = 64.0;

#[derive(Component)]
pub struct Rock;

#[derive(Component)]
struct SmallRock;

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
            .add_systems(OnEnter(AppState::FinishLevel), despawn_rock)
            .add_systems(
                Update,
                (
                    spawn_rock,
                    spawn_small_rocks,
                    despawn_rock_on_restart,
                    despawn_smallrock,
                )
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .add_event::<SmallRockAboutToRelease>();
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
    rock_run_assets: Res<RockRunAssets>,
    mut rock_sensor_collision: EventReader<PositionSensorCollisionStart>,
) {
    for collision_event in rock_sensor_collision.read() {
        if !collision_event.sensor_name.contains("rock") {
            return;
        }

        let texture = rock_run_assets.rock_ball.clone();

        commands
            .spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(ROCK_SCALE_FACTOR),
                    translation: collision_event.spawn_pos.extend(20.0),
                    ..default()
                },
                RigidBody::Dynamic,
                GravityScale(20.0),
                Velocity::zero(),
                Collider::compound(get_collider_shapes(false)),
                ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                Ccd::enabled(),
                Rock,
            ))
            .insert(ExternalImpulse {
                impulse: Vec2::new(-4096.0 * 120.0, 0.0),
                ..default()
            });
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_small_rocks(
    mut commands: Commands,
    time: Res<Time>,
    rock_run_assets: Res<RockRunAssets>,
    mut spawn_timer: Local<Timer>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut small_rock_event: EventWriter<SmallRockAboutToRelease>,
    mut event_send: Local<bool>,
) {
    if current_level.id != 2 {
        return;
    }

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    spawn_timer.tick(time.delta());
    if spawn_timer.remaining() <= Duration::from_secs(1) && !*event_send {
        small_rock_event.send(SmallRockAboutToRelease);
        *event_send = true;
    }

    if spawn_timer.finished() {
        *event_send = false;
        let mut rng = thread_rng();
        let spawn_time: f32 = rng.gen_range(1.0..=3.5);
        *spawn_timer = Timer::from_seconds(spawn_time, TimerMode::Once);
        let texture = rock_run_assets.small_rock.clone();
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            Transform {
                scale: Vec3::splat(1.0),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(1475.0, 750.0))
                    .extend(4.0),
                ..default()
            },
            RigidBody::Dynamic,
            GravityScale(20.0),
            Velocity::zero(),
            Collider::ball(16.0),
            ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
            Ccd::enabled(),
            Damping {
                angular_damping: 7.0,
                ..default()
            },
            Rock,
            SmallRock,
        ));

        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            Transform {
                scale: Vec3::splat(1.0),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(5600.0, 750.0))
                    .extend(4.0),
                ..default()
            },
            RigidBody::Dynamic,
            GravityScale(20.0),
            Velocity::zero(),
            Collider::ball(16.0),
            ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
            Ccd::enabled(),
            Damping {
                angular_damping: 7.5,
                ..default()
            },
            Rock,
            SmallRock,
        ));
    }
}

fn despawn_smallrock(
    mut commands: Commands,
    rocks: Query<(Entity, &Transform), With<SmallRock>>,
    laval: Query<&Transform, With<Lava>>,
) {
    if let Ok(lava_pos) = laval.get_single() {
        for (rock, rock_pos) in rocks.iter() {
            if rock_pos.translation.y < lava_pos.translation.y {
                commands.entity(rock).despawn_recursive();
            }
        }
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
