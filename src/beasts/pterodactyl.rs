use bevy::{audio::PlaybackMode, prelude::*};
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::{Ccd, GravityScale, RigidBody, Velocity},
    geometry::{ActiveCollisionTypes, Collider},
    pipeline::QueryFilterFlags,
};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::state::AppState,
    elements::rock::Rock,
    events::{PositionSensorCollisionStart, Restart},
    helpers::texture::cycle_texture,
    player::Player,
    WINDOW_HEIGHT, WINDOW_WIDTH,
};

const PTERODACTYL_SPEED: f32 = 400.0;
const PTERODACTYL_SCALE_FACTOR: f32 = 1.0;
const PTERODACTYL_WIDTH: f32 = 128.0;
const PTERODACTYL_HEIGHT: f32 = 112.0;
const SMOOTH_FACTOR: f32 = 2.0;
const ROCK_SCALE_FACTOR: f32 = 1.0;

#[derive(Component)]
pub struct Pterodactyl {
    exit_pos: Vec2,
    current_movement: PterodactylMovement,
    attack: bool,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct ChaseTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct ThrowTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum PterodactylMovement {
    Fly(PterodactylDirection),
    Throw,
}

impl Default for PterodactylMovement {
    fn default() -> Self {
        Self::Fly(PterodactylDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum PterodactylDirection {
    Left,
    #[default]
    Right,
}

pub struct PterodactylPlugin;

impl Plugin for PterodactylPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_pterodactyl)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_pterodactyl)
            .add_systems(
                Update,
                (
                    move_pterodactyl,
                    spawn_pterodactyl,
                    despawn_pterodactyl_on_restart,
                )
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = vec![(
        // body
        Vec2::new(5.0, -15.0),
        0.0,
        Collider::cuboid(PTERODACTYL_WIDTH / 2.6, PTERODACTYL_HEIGHT / 5.6),
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

fn spawn_pterodactyl(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut pterodactyl_sensor_collision: EventReader<PositionSensorCollisionStart>,
) {
    for collision_event in pterodactyl_sensor_collision.read() {
        if !collision_event.sensor_name.contains("pterodactyl") {
            return;
        }

        let texture = rock_run_assets.pterodactyl.clone();
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(PTERODACTYL_WIDTH as u32, PTERODACTYL_HEIGHT as u32),
            4,
            4,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlases.add(layout);

        let pterodactyl = match collision_event.sensor_name.contains("pterodactyl_attack") {
            true => Pterodactyl {
                exit_pos: collision_event.exit_pos,
                current_movement: PterodactylMovement::Fly(PterodactylDirection::Left),
                attack: true,
            },
            false => Pterodactyl {
                exit_pos: collision_event.exit_pos,
                current_movement: PterodactylMovement::Fly(PterodactylDirection::Left),
                attack: false,
            },
        };

        commands.spawn((
            SpriteBundle {
                texture,
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(PTERODACTYL_SCALE_FACTOR),
                    translation: collision_event.spawn_pos.extend(20.0),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ChaseTimer(Timer::from_seconds(20.0, TimerMode::Once)),
            ThrowTimer(Timer::from_seconds(0.3, TimerMode::Once)),
            Collider::compound(get_collider_shapes(false)),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::ONLY_KINEMATIC,
                ..default()
            },
            pterodactyl,
        ));

        commands.spawn(AudioBundle {
            source: rock_run_assets.pterodactyl_sound.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn move_pterodactyl(
    mut commands: Commands,
    time: Res<Time>,
    mut pterodactyl_query: Query<
        (
            Entity,
            &mut Collider,
            &mut KinematicCharacterController,
            &mut Transform,
            &mut Pterodactyl,
        ),
        With<Pterodactyl>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
    player_query: Query<&mut Transform, (With<Player>, Without<Pterodactyl>)>,
    mut chase_timer: Query<&mut ChaseTimer>,
    mut throw_timer: Query<&mut ThrowTimer>,
    rock_run_assets: Res<RockRunAssets>,
) {
    for (
        pterodactyl_entity,
        mut pterodactyl_collider,
        mut pterodactyl_controller,
        pterodactyl_pos,
        mut pterodactyl,
    ) in pterodactyl_query.iter_mut()
    {
        let player = player_query.single();
        let mut anim =
            |current_movement: PterodactylMovement, commands: &mut Commands| match current_movement
            {
                PterodactylMovement::Fly(pterodactyl_direction) => {
                    let (mut anim_timer, mut texture, mut sprite) =
                        animation_query.get_mut(pterodactyl_entity).unwrap();
                    anim_timer.tick(time.delta());
                    match pterodactyl_direction {
                        PterodactylDirection::Left => {
                            sprite.flip_x = true;
                            *pterodactyl_collider = Collider::compound(get_collider_shapes(true));
                        }
                        PterodactylDirection::Right => {
                            sprite.flip_x = false;
                            *pterodactyl_collider = Collider::compound(get_collider_shapes(false));
                        }
                    }
                    if anim_timer.just_finished() {
                        cycle_texture(&mut texture, 0..=4);
                    }
                }

                PterodactylMovement::Throw => {
                    let (mut _anim_timer, mut texture, mut _sprite) =
                        animation_query.get_mut(pterodactyl_entity).unwrap();
                    texture.index = 5;

                    commands.spawn(AudioBundle {
                        source: rock_run_assets.pterodactyl_sound.clone(),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Despawn,
                            ..default()
                        },
                    });
                }
            };

        let mut chase_timer = chase_timer.get_mut(pterodactyl_entity).unwrap();
        let mut throw_timer = throw_timer.get_mut(pterodactyl_entity).unwrap();
        let pterodactyl_pos = pterodactyl_pos.translation.xy();
        let player_pos = player.translation.xy();

        chase_timer.tick(time.delta());

        let direction = match pterodactyl.attack {
            false => {
                if chase_timer.finished() {
                    debug!("chase_timer finished");
                    debug!("pterodactyl_pos: {:?}", pterodactyl_pos);
                    (pterodactyl.exit_pos - pterodactyl_pos).normalize()
                        * PTERODACTYL_SPEED
                        * time.delta_seconds()
                } else {
                    // Lemniscate of Gerono above the player
                    let x = (WINDOW_WIDTH / 2.8)
                        * (time.elapsed_seconds() * PTERODACTYL_SPEED * 0.003).cos()
                        + player_pos.x;
                    let y = (WINDOW_HEIGHT / 2.8)
                        * (time.elapsed_seconds() * PTERODACTYL_SPEED * 0.003).sin()
                        * (time.elapsed_seconds() * PTERODACTYL_SPEED * 0.003).cos()
                        + player_pos.y
                        + 300.0;

                    pterodactyl_pos.lerp(Vec2::new(x, y), time.delta_seconds() * SMOOTH_FACTOR)
                        - pterodactyl_pos
                }
            }
            true => {
                (pterodactyl.exit_pos - pterodactyl_pos).normalize()
                    * PTERODACTYL_SPEED
                    * time.delta_seconds()
            }
        };

        if pterodactyl_pos.distance(pterodactyl.exit_pos) < 2.0 {
            commands.entity(pterodactyl_entity).despawn_recursive();
            return;
        }

        pterodactyl.current_movement = if direction.x >= 0.0 {
            PterodactylMovement::Fly(PterodactylDirection::Right)
        } else {
            PterodactylMovement::Fly(PterodactylDirection::Left)
        };

        if pterodactyl_pos.x < player_pos.x + 30.0
            && pterodactyl_pos.x > player_pos.x - 30.0
            && !pterodactyl.attack
        {
            if throw_timer.just_finished() {
                spawn_little_rock(&mut commands, pterodactyl_pos, &rock_run_assets);
                anim(pterodactyl.current_movement, &mut commands);
                pterodactyl_controller.translation = Some(Vec2::new(direction.x, direction.y));
                throw_timer.reset();
            } else {
                throw_timer.tick(time.delta());
                pterodactyl.current_movement = PterodactylMovement::Throw;
                anim(pterodactyl.current_movement, &mut commands);
                pterodactyl_controller.translation = None;
            }
        } else {
            anim(pterodactyl.current_movement, &mut commands);
            pterodactyl_controller.translation = Some(Vec2::new(direction.x, direction.y));
        };
    }
}

fn spawn_little_rock(
    commands: &mut Commands,
    current_pos: Vec2,
    rock_run_assets: &Res<RockRunAssets>,
) {
    let texture = rock_run_assets.rock_small.clone();

    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite { ..default() },
            transform: Transform {
                scale: Vec3::splat(ROCK_SCALE_FACTOR),
                translation: (current_pos + Vec2::new(0.0, -28.0)).extend(20.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        GravityScale(9.0),
        Velocity::zero(),
        Collider::ball(8.0),
        ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
        Ccd::enabled(),
        Rock,
    ));
}

fn despawn_pterodactyl(mut commands: Commands, pterodactyls: Query<Entity, With<Pterodactyl>>) {
    for pterodactyl in pterodactyls.iter() {
        commands.entity(pterodactyl).despawn_recursive();
    }
}

fn despawn_pterodactyl_on_restart(
    mut commands: Commands,
    pterodactyls: Query<Entity, With<Pterodactyl>>,
    restart_event: EventReader<Restart>,
) {
    if restart_event.is_empty() {
        return;
    }

    for pterodactyl in pterodactyls.iter() {
        commands.entity(pterodactyl).despawn_recursive();
    }
}
