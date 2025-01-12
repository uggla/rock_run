use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
    pipeline::QueryFilterFlags,
};

use crate::{
    assets::RockRunAssets,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{MovingPlatformCollision, MovingPlatformDescending},
    player::{PlayerSet, PlayerState},
};

const MOVING_PLATFORM_SCALE_FACTOR: f32 = 1.0;
const MOVING_PLATFORM_WIDTH: f32 = 96.0;
const MOVING_PLATFORM_HEIGHT: f32 = 16.0;

#[derive(Component, Clone)]
pub struct MovingPlatform {
    pub start_pos: Vec2,
    pub movement: MovingPlatformMovement,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MovingPlatformMovement {
    LeftRight(LeftRightData),
    UpDown(UpDownData),
    Circle(CircleData),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CircleData {
    direction: MovingPlatformDirection,
    center: Vec2,
    speed: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LeftRightData {
    direction: MovingPlatformDirection,
    max_right: f32,
    max_left: f32,
    speed: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UpDownData {
    direction: MovingPlatformDirection,
    max_down: f32,
    max_up: f32,
    pub speed: f32,
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
enum MovingPlatformDirection {
    Left,
    Right,
    Up,
    Down,
    Clockwise,
    Anticlockwise,
}

pub struct MovingPlatformPlugin;

impl Plugin for MovingPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_moving_platform)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_moving_platform)
            .add_systems(OnEnter(AppState::GameCreate), setup_moving_platforms)
            .add_systems(OnEnter(AppState::NextLevel), setup_moving_platforms)
            .add_systems(
                Update,
                (move_moving_platform)
                    .before(PlayerSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .add_event::<MovingPlatformDescending>();
    }
}

fn setup_moving_platforms(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_moving_moving_platforms");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.moving_platform.clone();
    let mut level_moving_platforms: HashMap<u8, Vec<MovingPlatform>> = HashMap::new();
    level_moving_platforms.insert(
        1,
        vec![
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(2145.0, 550.0)),
                movement: MovingPlatformMovement::UpDown(UpDownData {
                    direction: MovingPlatformDirection::Up,
                    max_down: level.map.tiled_to_bevy_coord(Vec2::new(2145.0, 575.0)).y,
                    max_up: level.map.tiled_to_bevy_coord(Vec2::new(2145.0, 335.0)).y,
                    speed: 1.0,
                }),
            },
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5750.0, 368.0)),
                movement: MovingPlatformMovement::Circle(CircleData {
                    center: level.map.tiled_to_bevy_coord(Vec2::new(5875.0, 368.0)),
                    direction: MovingPlatformDirection::Anticlockwise,
                    speed: 1.0,
                }),
            },
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(6165.0, 368.0)),
                movement: MovingPlatformMovement::Circle(CircleData {
                    center: level.map.tiled_to_bevy_coord(Vec2::new(6290.0, 368.0)),
                    direction: MovingPlatformDirection::Clockwise,
                    speed: 2.0,
                }),
            },
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(6515.0, 368.0)),
                movement: MovingPlatformMovement::LeftRight(LeftRightData {
                    direction: MovingPlatformDirection::Right,
                    max_left: level.map.tiled_to_bevy_coord(Vec2::new(6515.0, 0.0)).x,
                    max_right: level.map.tiled_to_bevy_coord(Vec2::new(6915.0, 0.0)).x,
                    speed: 2.5,
                }),
            },
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(7100.0, 400.0)),
                movement: MovingPlatformMovement::UpDown(UpDownData {
                    direction: MovingPlatformDirection::Up,
                    max_down: level.map.tiled_to_bevy_coord(Vec2::new(7100.0, 549.0)).y,
                    max_up: level.map.tiled_to_bevy_coord(Vec2::new(7100.0, 400.0)).y,
                    speed: 0.0,
                }),
            },
        ],
    );
    level_moving_platforms.insert(
        3,
        vec![
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5920.0, 576.0)),
                movement: MovingPlatformMovement::Circle(CircleData {
                    center: level.map.tiled_to_bevy_coord(Vec2::new(5920.0, 451.0)),
                    direction: MovingPlatformDirection::Clockwise,
                    speed: 1.0,
                }),
            },
            MovingPlatform {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5920.0, 326.0)),
                movement: MovingPlatformMovement::Circle(CircleData {
                    center: level.map.tiled_to_bevy_coord(Vec2::new(5920.0, 451.0)),
                    direction: MovingPlatformDirection::Clockwise,
                    speed: 1.0,
                }),
            },
            // MovingPlatform {
            //     start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5920.0, 176.0)),
            //     movement: MovingPlatformMovement::LeftRight(LeftRightData {
            //         direction: MovingPlatformDirection::Right,
            //         max_left: level.map.tiled_to_bevy_coord(Vec2::new(5800.0, 0.0)).x,
            //         max_right: level.map.tiled_to_bevy_coord(Vec2::new(6000.0, 0.0)).x,
            //     }),
            // },
            // MovingPlatform {
            //     start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5600.0, 176.0)),
            //     movement: MovingPlatformMovement::UpDown(UpDownData {
            //         direction: MovingPlatformDirection::Up,
            //         max_up: level.map.tiled_to_bevy_coord(Vec2::new(0.0, 426.0)).y,
            //         max_down: level.map.tiled_to_bevy_coord(Vec2::new(0.0, 170.0)).y,
            //     }),
            // },
        ],
    );

    let moving_platforms = match level_moving_platforms.get(&current_level.id) {
        Some(items) => items,
        None => return,
    };

    for moving_platform in moving_platforms {
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            Transform {
                scale: Vec3::splat(MOVING_PLATFORM_SCALE_FACTOR),
                translation: moving_platform.start_pos.extend(8.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            Collider::cuboid(MOVING_PLATFORM_WIDTH / 2.0, MOVING_PLATFORM_HEIGHT / 2.0),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::ONLY_KINEMATIC,
                ..default()
            },
            moving_platform.clone(),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn move_moving_platform(
    time: Res<Time>,
    state: Res<State<PlayerState>>,
    mut moving_platform_query: Query<(Entity, &mut Transform, &mut MovingPlatform)>,
    mut moving_platform_collision: EventReader<MovingPlatformCollision>,
    mut moving_platform_descending: EventWriter<MovingPlatformDescending>,
) {
    let player_on_platform_events = moving_platform_collision.read().collect::<Vec<_>>();
    for (moving_platform_entity, mut moving_platform_pos, mut moving_platform) in
        moving_platform_query.iter_mut()
    {
        let (translation_x, translation_y) = match moving_platform.movement {
            MovingPlatformMovement::LeftRight(ref mut right_left_data) => {
                match right_left_data.direction {
                    MovingPlatformDirection::Left => {
                        let moving_platform_speed = -right_left_data.speed * 100.0;
                        if moving_platform_pos.translation.x > right_left_data.max_left {
                            (moving_platform_speed * time.delta_secs(), 0.0)
                        } else {
                            right_left_data.direction = MovingPlatformDirection::Right;
                            (0.0, 0.0)
                        }
                    }
                    MovingPlatformDirection::Right => {
                        let moving_platform_speed = right_left_data.speed * 100.0;
                        if moving_platform_pos.translation.x < right_left_data.max_right {
                            (moving_platform_speed * time.delta_secs(), 0.0)
                        } else {
                            right_left_data.direction = MovingPlatformDirection::Left;
                            (0.0, 0.0)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            MovingPlatformMovement::UpDown(ref mut up_down_data) => match up_down_data.direction {
                MovingPlatformDirection::Up => {
                    let moving_platform_speed = -up_down_data.speed * 100.0;
                    if moving_platform_pos.translation.y > up_down_data.max_down {
                        (0.0, moving_platform_speed * time.delta_secs())
                    } else {
                        up_down_data.direction = MovingPlatformDirection::Down;
                        (0.0, 0.0)
                    }
                }
                MovingPlatformDirection::Down => {
                    let moving_platform_speed = up_down_data.speed * 100.0;
                    if moving_platform_pos.translation.y < up_down_data.max_up {
                        (0.0, moving_platform_speed * time.delta_secs())
                    } else {
                        up_down_data.direction = MovingPlatformDirection::Up;
                        (0.0, 0.0)
                    }
                }
                _ => unreachable!(),
            },
            MovingPlatformMovement::Circle(circle_data) => match circle_data.direction {
                MovingPlatformDirection::Clockwise => {
                    let moving_platform_speed = circle_data.speed;
                    rotate_platform(
                        &time,
                        moving_platform_speed,
                        &moving_platform_pos,
                        circle_data.center,
                    )
                }
                MovingPlatformDirection::Anticlockwise => {
                    let moving_platform_speed = -circle_data.speed;
                    rotate_platform(
                        &time,
                        moving_platform_speed,
                        &moving_platform_pos,
                        circle_data.center,
                    )
                }
                _ => unreachable!(),
            },
        };

        moving_platform_pos.translation += Vec2::new(translation_x, translation_y).extend(0.0);

        if let Some(current_platform) = player_on_platform_events
            .iter()
            .find(|e| e.entity == moving_platform_entity)
        {
            trace!("Player on moving platform {:?}", current_platform.entity);
            if translation_y < 0.0 && state.get() == &PlayerState::Idling {
                moving_platform_descending.send(MovingPlatformDescending {
                    movement: Vec2::new(translation_x, translation_y),
                });
            }
        }
    }
}

fn rotate_platform(
    time: &Res<Time>,
    moving_platform_speed: f32,
    moving_platform_pos: &Mut<Transform>,
    center: Vec2,
) -> (f32, f32) {
    let angle = time.delta_secs() * moving_platform_speed % (2.0 * PI);
    let target_pos =
        Vec2::from_angle(angle).rotate(moving_platform_pos.translation.xy() - center) + center;
    let translation = target_pos - moving_platform_pos.translation.xy();

    (translation.x, translation.y)
}

fn despawn_moving_platform(
    mut commands: Commands,
    moving_platforms: Query<Entity, With<MovingPlatform>>,
) {
    for moving_platform in moving_platforms.iter() {
        commands.entity(moving_platform).despawn_recursive();
    }
}
