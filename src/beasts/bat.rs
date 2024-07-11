use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
    pipeline::QueryFilterFlags,
};

use crate::{
    collisions::CollisionSet,
    coregame::state::AppState,
    events::{Hit, PositionSensorCollisionStart, Restart},
    helpers::texture::cycle_texture,
    player::Player,
};

const BAT_SPEED: f32 = 100.0;
const BAT_SCALE_FACTOR: f32 = 1.0;
const BAT_WIDTH: f32 = 50.0;
const BAT_HEIGHT: f32 = 57.0;

#[derive(Component)]
pub struct Bat {
    exit_pos: Vec2,
    current_movement: BatMovement,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct ChaseTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum BatMovement {
    Fly(BatDirection),
    Crunch,
}

impl Default for BatMovement {
    fn default() -> Self {
        Self::Fly(BatDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum BatDirection {
    Left,
    #[default]
    Right,
}

pub struct BatPlugin;

impl Plugin for BatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_bat)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_bat)
            .add_systems(
                Update,
                (move_bat, spawn_bat, despawn_bat_on_restart)
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = vec![(
        // body
        Vec2::new(3.0, -10.0),
        0.0,
        Collider::cuboid(BAT_WIDTH / 4.0, BAT_HEIGHT / 4.5),
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

fn spawn_bat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut bat_sensor_collision: EventReader<PositionSensorCollisionStart>,
) {
    for collision_event in bat_sensor_collision.read() {
        if !collision_event.sensor_name.contains("bat") {
            return;
        }

        let texture = asset_server.load("bat-1.png");
        let layout =
            TextureAtlasLayout::from_grid(Vec2::new(BAT_WIDTH, BAT_HEIGHT), 7, 2, None, None);
        let texture_atlas_layout = texture_atlases.add(layout);

        commands.spawn((
            SpriteSheetBundle {
                texture,
                sprite: Sprite { ..default() },
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
                transform: Transform {
                    scale: Vec3::splat(BAT_SCALE_FACTOR),
                    translation: collision_event.spawn_pos.extend(20.0),
                    ..default()
                },
                ..default()
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ChaseTimer(Timer::from_seconds(15.0, TimerMode::Once)),
            Collider::compound(get_collider_shapes(false)),
            KinematicCharacterController { ..default() },
            Bat {
                exit_pos: collision_event.exit_pos,
                current_movement: BatMovement::Fly(BatDirection::default()),
            },
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn move_bat(
    mut commands: Commands,
    time: Res<Time>,
    mut bat_query: Query<
        (
            Entity,
            &mut Collider,
            &mut KinematicCharacterController,
            &mut Transform,
            &mut Bat,
        ),
        With<Bat>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
    // mut current_movement: Local<BatMovement>,
    player_query: Query<&mut Transform, (With<Player>, Without<Bat>)>,
    hit: EventReader<Hit>,
    mut chase_timer: Query<&mut ChaseTimer>,
) {
    for (bat_entity, mut bat_collider, mut bat_controller, bat_pos, mut bat) in bat_query.iter_mut()
    {
        let player = player_query.single();
        let mut anim = |current_movement: BatMovement| match current_movement {
            BatMovement::Fly(bat_direction) => {
                let (mut anim_timer, mut texture, mut sprite) =
                    animation_query.get_mut(bat_entity).unwrap();
                anim_timer.tick(time.delta());
                match bat_direction {
                    BatDirection::Left => {
                        sprite.flip_x = true;
                        *bat_collider = Collider::compound(get_collider_shapes(true));
                    }
                    BatDirection::Right => {
                        sprite.flip_x = false;
                        *bat_collider = Collider::compound(get_collider_shapes(false));
                    }
                }
                if anim_timer.just_finished() {
                    cycle_texture(&mut texture, 0..=2);
                }
            }

            BatMovement::Crunch => {
                let (mut _anim_timer, mut texture, mut _sprite) =
                    animation_query.get_mut(bat_entity).unwrap();
                texture.index = 5;
            }
        };

        if !hit.is_empty() {
            bat.current_movement = BatMovement::Crunch;
            anim(bat.current_movement);
            return;
        }

        let mut chase_timer = chase_timer.get_mut(bat_entity).unwrap();
        let bat_pos = bat_pos.translation.xy();
        let player_pos = player.translation.xy();

        chase_timer.tick(time.delta());

        let direction = if chase_timer.finished() {
            debug!("chase_timer finished");
            debug!("bat_pos: {:?}", bat_pos);
            bat_controller.filter_flags = QueryFilterFlags::ONLY_KINEMATIC;
            (bat.exit_pos - bat_pos).normalize() * BAT_SPEED * time.delta_seconds()
        } else {
            (player_pos - bat_pos).normalize() * BAT_SPEED * time.delta_seconds()
        };

        if bat_pos.distance(bat.exit_pos) < 2.0 {
            commands.entity(bat_entity).despawn_recursive();
            return;
        }

        bat.current_movement = if direction.x >= 0.0 {
            BatMovement::Fly(BatDirection::Right)
        } else {
            BatMovement::Fly(BatDirection::Left)
        };

        anim(bat.current_movement);
        bat_controller.translation = Some(Vec2::new(direction.x, direction.y));
    }
}

fn despawn_bat(mut commands: Commands, bats: Query<Entity, With<Bat>>) {
    for bat in bats.iter() {
        commands.entity(bat).despawn_recursive();
    }
}

fn despawn_bat_on_restart(
    mut commands: Commands,
    bats: Query<Entity, With<Bat>>,
    restart_event: EventReader<Restart>,
) {
    if restart_event.is_empty() {
        return;
    }

    for bat in bats.iter() {
        commands.entity(bat).despawn_recursive();
    }
}
