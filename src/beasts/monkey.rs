use bevy::{platform::collections::HashMap, prelude::*};
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
    events::SmallRockAboutToRelease,
};

const MONKEY_SCALE_FACTOR: f32 = 1.0;
const MONKEY_WIDTH: f32 = 100.0;
const MONKEY_HEIGHT: f32 = 140.0;

#[derive(Component)]
pub struct Monkey {
    start_pos: Vec2,
    current_movement: MonkeyMovement,
    initial_movement: MonkeyMovement,
    texture: Handle<Image>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum MonkeyMovement {
    Look(MonkeyDirection),
    Throw,
}

impl Default for MonkeyMovement {
    fn default() -> Self {
        Self::Look(MonkeyDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum MonkeyDirection {
    Left,
    #[default]
    Right,
}

pub struct MonkeyPlugin;

impl Plugin for MonkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_monkey)
            .add_systems(OnEnter(AppState::NextLevel), setup_monkey)
            .add_systems(OnEnter(AppState::StartMenu), despawn_monkey)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_monkey)
            .add_systems(
                Update,
                move_monkey
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn setup_monkey(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_monkey");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(MONKEY_WIDTH as u32, MONKEY_HEIGHT as u32),
        11,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);
    let mut level_monkey: HashMap<u8, Vec<Monkey>> = HashMap::new();
    level_monkey.insert(
        2,
        vec![
            Monkey {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(1415.0, 705.0)),
                current_movement: MonkeyMovement::Look(MonkeyDirection::Right),
                initial_movement: MonkeyMovement::Look(MonkeyDirection::Right),
                texture: rock_run_assets.monkey2.clone(),
            },
            Monkey {
                start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5650.0, 705.0)),
                current_movement: MonkeyMovement::Look(MonkeyDirection::Left),
                initial_movement: MonkeyMovement::Look(MonkeyDirection::Left),
                texture: rock_run_assets.monkey.clone(),
            },
        ],
    );

    let monkeys = match level_monkey.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for monkey in monkeys {
        commands.spawn((
            Sprite {
                image: monkey.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform {
                scale: Vec3::splat(MONKEY_SCALE_FACTOR),
                translation: monkey.start_pos.extend(20.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Collider::compound(vec![(
                Vec2::new(0.0, -30.0),
                0.0,
                Collider::cuboid(MONKEY_WIDTH / 2.0, MONKEY_HEIGHT / 3.70),
            )]),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                ..default()
            },
            Monkey {
                current_movement: monkey.current_movement,
                initial_movement: monkey.current_movement,
                start_pos: monkey.start_pos,
                texture: monkey.texture.clone(),
            },
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn move_monkey(
    mut commands: Commands,
    time: Res<Time>,
    mut monkey_query: Query<(Entity, &mut Monkey), With<Monkey>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    mut small_rock_event: EventReader<SmallRockAboutToRelease>,
) {
    let mut event_received = false;

    if !small_rock_event.is_empty() {
        event_received = true;
        small_rock_event.clear();
    }

    for (monkey_entity, mut monkey) in monkey_query.iter_mut() {
        let (mut anim_timer, mut sprite) = animation_query.get_mut(monkey_entity).unwrap();

        let mut anim =
            |current_movement: MonkeyMovement, _commands: &mut Commands| match current_movement {
                MonkeyMovement::Look(monkey_direction) => {
                    anim_timer.tick(time.delta());
                    match monkey_direction {
                        MonkeyDirection::Left => {
                            sprite.flip_x = false;
                        }
                        MonkeyDirection::Right => {
                            sprite.flip_x = true;
                        }
                    }
                    if anim_timer.just_finished() {
                        if let Some(texture) = &mut sprite.texture_atlas {
                            texture.index = 9;
                        }
                    }
                }
                MonkeyMovement::Throw => {
                    anim_timer.tick(time.delta());
                    if anim_timer.just_finished() {
                        if let Some(texture) = &mut sprite.texture_atlas {
                            texture.index += 1;
                        }
                    }
                }
            };

        anim(monkey.current_movement, &mut commands);

        if event_received {
            monkey.current_movement = MonkeyMovement::Throw;
            if let Some(texture) = &mut sprite.texture_atlas {
                texture.index = 0;
            }
        }

        if let Some(texture) = &mut sprite.texture_atlas {
            if texture.index == 9 {
                monkey.current_movement = monkey.initial_movement;
            }
        }
    }
}

fn despawn_monkey(mut commands: Commands, monkey: Query<Entity, With<Monkey>>) {
    for monkey in monkey.iter() {
        commands.entity(monkey).despawn();
    }
}
