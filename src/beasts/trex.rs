use bevy::{audio::PlaybackMode, prelude::*, utils::HashMap};
use bevy_rapier2d::{
    control::{KinematicCharacterController, KinematicCharacterControllerOutput},
    dynamics::RigidBody,
    geometry::Collider,
    pipeline::QueryFilterFlags,
};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    helpers::texture::cycle_texture,
    player::Player,
    WINDOW_WIDTH,
};

const TREX_SPEED: f32 = 550.0;
const TREX_SCALE_FACTOR: f32 = 1.0;
const TREX_WIDTH: f32 = 150.0;
const TREX_HEIGHT: f32 = 105.0;

#[derive(Component)]
pub struct Trex {
    current_movement: TrexMovement,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum TrexMovement {
    Run(TrexDirection),
    Idle,
}

impl Default for TrexMovement {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum TrexDirection {
    Left,
    #[default]
    Right,
}

enum ColliderType {
    Normal,
    Bite,
}

pub struct TrexPlugin;

impl Plugin for TrexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_trex)
            .add_systems(OnEnter(AppState::NextLevel), setup_trex)
            .add_systems(OnEnter(AppState::StartMenu), despawn_trex)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_trex)
            .add_systems(
                Update,
                move_trex
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(collider_type: ColliderType, y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = match collider_type {
        ColliderType::Normal => vec![
            (
                //head
                Vec2::new(32.0, 6.0),
                0.0,
                Collider::cuboid(40.0 / 2.0, 30.0 / 2.0),
            ),
            (
                // body
                Vec2::new(4.0, -20.0),
                0.0,
                Collider::cuboid(68.0 / 2.0, 60.0 / 2.0),
            ),
            (
                // tail
                Vec2::new(-48.0, -30.0),
                0.0,
                Collider::cuboid(34.0 / 2.0, 26.0 / 2.0),
            ),
        ],
        ColliderType::Bite => vec![
            (
                //head
                Vec2::new(50.0, -20.0),
                0.0,
                Collider::cuboid(40.0 / 2.0, 30.0 / 2.0),
            ),
            (
                // body
                Vec2::new(4.0, -20.0),
                0.0,
                Collider::cuboid(68.0 / 2.0, 60.0 / 2.0),
            ),
            (
                // tail
                Vec2::new(-48.0, -30.0),
                0.0,
                Collider::cuboid(34.0 / 2.0, 26.0 / 2.0),
            ),
        ],
    };

    if y_mirror {
        shapes
            .into_iter()
            .map(|(pos, angle, shape)| (pos * Vec2::new(-1.0, 1.0), angle, shape))
            .collect()
    } else {
        shapes
    }
}

fn setup_trex(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_trex");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.trex.clone();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TREX_WIDTH as u32, TREX_HEIGHT as u32),
        6,
        4,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);
    let mut level_trex_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    level_trex_pos.insert(
        2,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(9200.0, 2030.0))],
    );

    let start_positions = match level_trex_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform {
                scale: Vec3::splat(TREX_SCALE_FACTOR),
                translation: start_pos.extend(20.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Collider::compound(get_collider_shapes(ColliderType::Normal, false)),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::ONLY_FIXED,
                max_slope_climb_angle: 30.0f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0f32.to_radians(),
                normal_nudge_factor: 1.0,
                ..default()
            },
            Trex {
                current_movement: TrexMovement::default(),
            },
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn move_trex(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    time: Res<Time>,
    mut trex_query: Query<
        (
            Entity,
            &mut Trex,
            &mut Collider,
            &Transform,
            &mut KinematicCharacterController,
        ),
        With<Trex>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    player_query: Query<&mut Transform, (With<Player>, Without<Trex>)>,
    mut previous_direction: Local<TrexDirection>,
    mut previous_movement: Local<TrexMovement>,
    mut speed_coef: Local<f32>,
    trex_controller_query: Query<&KinematicCharacterControllerOutput, With<Trex>>,
) {
    let player = player_query.single();
    for (trex_entity, mut trex, mut trex_collider, trex_pos, mut trex_controller) in
        trex_query.iter_mut()
    {
        let mut anim =
            |current_movement: TrexMovement, commands: &mut Commands| match current_movement {
                TrexMovement::Run(trex_direction) => {
                    let (mut anim_timer, mut sprite) =
                        animation_query.get_mut(trex_entity).unwrap();
                    anim_timer.tick(time.delta());
                    match trex_direction {
                        TrexDirection::Left => {
                            sprite.flip_x = true;
                            if let Some(texture) = &mut sprite.texture_atlas {
                                if texture.index > 15 {
                                    *trex_collider = Collider::compound(get_collider_shapes(
                                        ColliderType::Bite,
                                        sprite.flip_x,
                                    ));
                                } else {
                                    *trex_collider = Collider::compound(get_collider_shapes(
                                        ColliderType::Normal,
                                        sprite.flip_x,
                                    ));
                                }
                            }
                        }
                        TrexDirection::Right => {
                            sprite.flip_x = false;
                            if let Some(texture) = &mut sprite.texture_atlas {
                                if texture.index > 15 {
                                    *trex_collider = Collider::compound(get_collider_shapes(
                                        ColliderType::Bite,
                                        sprite.flip_x,
                                    ));
                                } else {
                                    *trex_collider = Collider::compound(get_collider_shapes(
                                        ColliderType::Normal,
                                        sprite.flip_x,
                                    ));
                                }
                            }
                        }
                    }
                    if anim_timer.just_finished() {
                        if let Some(texture) = &mut sprite.texture_atlas {
                            cycle_texture(texture, 6..=18);
                            if texture.index == 16 {
                                commands.spawn((
                                    AudioPlayer::new(rock_run_assets.trex_bite_sound.clone()),
                                    PlaybackSettings {
                                        mode: PlaybackMode::Despawn,
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                }
                TrexMovement::Idle => {
                    let (mut anim_timer, mut _sprite) =
                        animation_query.get_mut(trex_entity).unwrap();
                    anim_timer.tick(time.delta());
                    if anim_timer.just_finished() {
                        if let Some(texture) = &mut _sprite.texture_atlas {
                            cycle_texture(texture, 0..=5);
                        }
                    }
                }
            };

        if trex_pos.translation.xy().distance(player.translation.xy()) > WINDOW_WIDTH / 2.0 - 150.0
        {
            trex.current_movement = TrexMovement::Idle;
        } else {
            let direction = if player.translation.x > trex_pos.translation.x {
                TrexDirection::Right
            } else {
                TrexDirection::Left
            };

            if *previous_direction != direction {
                *previous_direction = direction;
                *speed_coef = 0.0;
            }

            trex.current_movement = TrexMovement::Run(direction);
        }

        if trex.current_movement != TrexMovement::Idle {
            // Smooth movement when trex is changing direction.
            let delta_pos_x = (player.translation.x - trex_pos.translation.x).signum()
                * TREX_SPEED
                * *speed_coef
                * time.delta_secs();

            *speed_coef += 0.01;
            if *speed_coef > 1.0 {
                *speed_coef = 1.0;
            }

            trex_controller.translation =
                Some(Vec2::new(delta_pos_x, -TREX_SPEED * time.delta_secs()));

            if let Ok(trex_controller_output) = trex_controller_query.get(trex_entity) {
                // If trex is blocked, then set it to Idle.
                if trex_controller_output.effective_translation.x.abs() < 1.0 {
                    trex.current_movement = TrexMovement::Idle;
                }
            }
        }

        if *previous_movement != trex.current_movement {
            *previous_movement = trex.current_movement;
            commands.spawn((
                AudioPlayer::new(rock_run_assets.trex_rush_sound.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            ));
        }

        anim(trex.current_movement, &mut commands);
    }
}

fn despawn_trex(mut commands: Commands, trex: Query<Entity, With<Trex>>) {
    for trex in trex.iter() {
        commands.entity(trex).despawn_recursive();
    }
}
