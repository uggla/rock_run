use bevy::{audio::PlaybackMode, prelude::*};
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
    pipeline::QueryFilterFlags,
};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{Hit, PositionSensorCollisionStart, Restart},
    helpers::texture::cycle_texture,
    player::Player,
};

const SQUIREL_SPEED: f32 = 100.0;
const SQUIREL_SCALE_FACTOR: f32 = 1.5;
const SQUIREL_WIDTH: f32 = 40.0;
const SQUIREL_HEIGHT: f32 = 32.0;

#[derive(Component)]
pub struct Squirel {
    current_movement: SquirelMovement,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct ChaseTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum SquirelMovement {
    Run(SquirelDirection),
    Idle,
}

impl Default for SquirelMovement {
    fn default() -> Self {
        Self::Run(SquirelDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum SquirelDirection {
    Left,
    #[default]
    Right,
}

pub struct SquirelPlugin;

impl Plugin for SquirelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_squirel)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_squirel)
            .add_systems(OnEnter(AppState::GameCreate), spawn_squirel)
            .add_systems(
                Update,
                (move_squirel, despawn_squirel_on_restart)
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = vec![(
        // body
        Vec2::new(0.0, -SQUIREL_HEIGHT / 4.0),
        0.0,
        Collider::cuboid(SQUIREL_WIDTH / 4.0, SQUIREL_HEIGHT / 3.0),
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

fn spawn_squirel(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("spawn_squirel");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.squirel_idle.clone();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(SQUIREL_WIDTH as u32, SQUIREL_HEIGHT as u32),
        8,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);

    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                flip_x: true,
                ..default()
            },
            transform: Transform {
                scale: Vec3::splat(SQUIREL_SCALE_FACTOR),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(5214.0, 464.0 - SQUIREL_HEIGHT / 2.0))
                    .extend(10.0),
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: 0,
        },
        RigidBody::KinematicPositionBased,
        AnimationTimer(Timer::from_seconds(0.12, TimerMode::Repeating)),
        ChaseTimer(Timer::from_seconds(15.0, TimerMode::Once)),
        Collider::compound(get_collider_shapes(false)),
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::ONLY_FIXED,
            ..default()
        },
        Squirel {
            current_movement: SquirelMovement::Idle,
        },
    ));

    commands.spawn((
        SpriteBundle {
            texture: rock_run_assets.nut.clone(),
            sprite: Sprite { ..default() },
            transform: Transform {
                scale: Vec3::splat(2.0),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(5514.0, 470.0))
                    .extend(10.0),
                ..default()
            },
            ..default()
        },
        RigidBody::KinematicPositionBased,
        // AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        // ChaseTimer(Timer::from_seconds(15.0, TimerMode::Once)),
        // Collider::compound(get_collider_shapes(false)),
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::ONLY_FIXED,
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)]
fn move_squirel(
    mut commands: Commands,
    time: Res<Time>,
    mut squirel_query: Query<
        (
            Entity,
            &mut Collider,
            &mut KinematicCharacterController,
            &mut Transform,
            &mut Squirel,
        ),
        With<Squirel>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
    player_query: Query<&mut Transform, (With<Player>, Without<Squirel>)>,
    hit: EventReader<Hit>,
    mut chase_timer: Query<&mut ChaseTimer>,
) {
    for (squirel_entity, mut squirel_collider, mut squirel_controller, squirel_pos, mut squirel) in
        squirel_query.iter_mut()
    {
        let player = player_query.single();
        let mut anim = |current_movement: SquirelMovement| match current_movement {
            SquirelMovement::Run(squirel_direction) => {
                let (mut anim_timer, mut texture, mut sprite) =
                    animation_query.get_mut(squirel_entity).unwrap();
                anim_timer.tick(time.delta());
                match squirel_direction {
                    SquirelDirection::Left => {
                        sprite.flip_x = true;
                        *squirel_collider = Collider::compound(get_collider_shapes(true));
                    }
                    SquirelDirection::Right => {
                        sprite.flip_x = false;
                        *squirel_collider = Collider::compound(get_collider_shapes(false));
                    }
                }
                if anim_timer.just_finished() {
                    cycle_texture(&mut texture, 0..=7);
                }
            }

            SquirelMovement::Idle => {
                let (mut anim_timer, mut texture, mut _sprite) =
                    animation_query.get_mut(squirel_entity).unwrap();
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    cycle_texture(&mut texture, 0..=7);
                }
            }
        };

        let mut chase_timer = chase_timer.get_mut(squirel_entity).unwrap();
        let squirel_pos = squirel_pos.translation.xy();
        let player_pos = player.translation.xy();

        // chase_timer.tick(time.delta());
        //
        // let direction = if chase_timer.finished() {
        //     debug!("chase_timer finished");
        //     debug!("squirel_pos: {:?}", squirel_pos);
        //     squirel_controller.filter_flags = QueryFilterFlags::ONLY_KINEMATIC;
        //     (squirel_pos).normalize() * SQUIREL_SPEED * time.delta_seconds()
        // } else {
        //     (player_pos - squirel_pos).normalize() * SQUIREL_SPEED * time.delta_seconds()
        // };
        //
        // squirel.current_movement = if direction.x >= 0.0 {
        //     SquirelMovement::Run(SquirelDirection::Right)
        // } else {
        //     SquirelMovement::Run(SquirelDirection::Left)
        // };

        anim(squirel.current_movement);
        // squirel_controller.translation = Some(Vec2::new(direction.x, direction.y));
    }
}

fn despawn_squirel(mut commands: Commands, squirels: Query<Entity, With<Squirel>>) {
    for squirel in squirels.iter() {
        commands.entity(squirel).despawn_recursive();
    }
}

fn despawn_squirel_on_restart(
    mut commands: Commands,
    squirels: Query<Entity, With<Squirel>>,
    restart_event: EventReader<Restart>,
) {
    if restart_event.is_empty() {
        return;
    }

    for squirel in squirels.iter() {
        commands.entity(squirel).despawn_recursive();
    }
}
