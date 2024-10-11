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

const MONKEY_SPEED: f32 = 500.0;
const MONKEY_SCALE_FACTOR: f32 = 1.0;
const MONKEY_WIDTH: f32 = 100.0;
const MONKEY_HEIGHT: f32 = 140.0;

#[derive(Component)]
pub struct Monkey {
    current_movement: MonkeyMovement,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum MonkeyMovement {
    Run(MonkeyDirection),
    Idle,
}

impl Default for MonkeyMovement {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum MonkeyDirection {
    Left,
    #[default]
    Right,
}

enum ColliderType {
    Normal,
    Bite,
}

pub struct MonkeyPlugin;

impl Plugin for MonkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_monkey)
            .add_systems(OnEnter(AppState::NextLevel), setup_monkey)
            .add_systems(OnEnter(AppState::StartMenu), despawn_monkey)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_monkey);
        // .add_systems(
        //     Update,
        //     move_monkey
        //         .after(CollisionSet)
        //         .run_if(in_state(AppState::GameRunning)),
        // );
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

    let texture = rock_run_assets.monkey.clone();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(MONKEY_WIDTH as u32, MONKEY_HEIGHT as u32),
        11,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);
    let mut level_monkey_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    level_monkey_pos.insert(
        2,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(5600.0, 720.0))],
    );

    let start_positions = match level_monkey_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(MONKEY_SCALE_FACTOR),
                    translation: start_pos.extend(20.0),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Collider::compound(get_collider_shapes(ColliderType::Normal, false)),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                ..default()
            },
            Monkey {
                // current_movement: MonkeyMovement::Run(MonkeyDirection::default()),
                current_movement: MonkeyMovement::default(),
            },
        ));
    }
}

// #[allow(clippy::too_many_arguments)]
// fn move_monkey(
//     mut commands: Commands,
//     rock_run_assets: Res<RockRunAssets>,
//     time: Res<Time>,
//     mut monkey_query: Query<
//         (
//             Entity,
//             &mut Monkey,
//             &mut Collider,
//             &Transform,
//             &mut KinematicCharacterController,
//         ),
//         With<Monkey>,
//     >,
//     mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
//     player_query: Query<&mut Transform, (With<Player>, Without<Monkey>)>,
//     mut previous_direction: Local<MonkeyDirection>,
//     mut previous_movement: Local<MonkeyMovement>,
//     mut speed_coef: Local<f32>,
//     monkey_controller_query: Query<&KinematicCharacterControllerOutput, With<Monkey>>,
// ) {
//     let player = player_query.single();
//     for (monkey_entity, mut monkey, mut monkey_collider, monkey_pos, mut monkey_controller) in
//         monkey_query.iter_mut()
//     {
//         let mut anim =
//             |current_movement: MonkeyMovement, commands: &mut Commands| match current_movement {
//                 MonkeyMovement::Run(monkey_direction) => {
//                     let (mut anim_timer, mut texture, mut sprite) =
//                         animation_query.get_mut(monkey_entity).unwrap();
//                     anim_timer.tick(time.delta());
//                     match monkey_direction {
//                         MonkeyDirection::Left => {
//                             sprite.flip_x = true;
//                             if texture.index > 15 {
//                                 *monkey_collider = Collider::compound(get_collider_shapes(
//                                     ColliderType::Bite,
//                                     sprite.flip_x,
//                                 ));
//                             } else {
//                                 *monkey_collider = Collider::compound(get_collider_shapes(
//                                     ColliderType::Normal,
//                                     sprite.flip_x,
//                                 ));
//                             }
//                         }
//                         MonkeyDirection::Right => {
//                             sprite.flip_x = false;
//                             if texture.index > 15 {
//                                 *monkey_collider = Collider::compound(get_collider_shapes(
//                                     ColliderType::Bite,
//                                     sprite.flip_x,
//                                 ));
//                             } else {
//                                 *monkey_collider = Collider::compound(get_collider_shapes(
//                                     ColliderType::Normal,
//                                     sprite.flip_x,
//                                 ));
//                             }
//                         }
//                     }
//                     if anim_timer.just_finished() {
//                         cycle_texture(&mut texture, 6..=18);
//                         if texture.index == 16 {
//                             commands.spawn(AudioBundle {
//                                 source: rock_run_assets.monkey_bite_sound.clone(),
//                                 settings: PlaybackSettings {
//                                     mode: PlaybackMode::Despawn,
//                                     ..default()
//                                 },
//                             });
//                         }
//                     }
//                 }
//                 MonkeyMovement::Idle => {
//                     let (mut anim_timer, mut texture, mut _sprite) =
//                         animation_query.get_mut(monkey_entity).unwrap();
//                     anim_timer.tick(time.delta());
//                     if anim_timer.just_finished() {
//                         cycle_texture(&mut texture, 0..=5);
//                     }
//                 }
//             };
//
//         if monkey_pos
//             .translation
//             .xy()
//             .distance(player.translation.xy())
//             > WINDOW_WIDTH / 2.0 - 150.0
//         {
//             monkey.current_movement = MonkeyMovement::Idle;
//         } else {
//             let direction = if player.translation.x > monkey_pos.translation.x {
//                 MonkeyDirection::Right
//             } else {
//                 MonkeyDirection::Left
//             };
//
//             if *previous_direction != direction {
//                 *previous_direction = direction;
//                 *speed_coef = 0.0;
//             }
//
//             monkey.current_movement = MonkeyMovement::Run(direction);
//         }
//
//         if monkey.current_movement != MonkeyMovement::Idle {
//             // Smooth movement when monkey is changing direction.
//             let delta_pos_x = (player.translation.x - monkey_pos.translation.x).signum()
//                 * MONKEY_SPEED
//                 * *speed_coef
//                 * time.delta_seconds();
//
//             *speed_coef += 0.01;
//             if *speed_coef > 1.0 {
//                 *speed_coef = 1.0;
//             }
//
//             monkey_controller.translation =
//                 Some(Vec2::new(delta_pos_x, -MONKEY_SPEED * time.delta_seconds()));
//
//             if let Ok(monkey_controller_output) = monkey_controller_query.get(monkey_entity) {
//                 // If monkey is blocked, then set it to Idle.
//                 if monkey_controller_output.effective_translation.x.abs() < 1.0 {
//                     monkey.current_movement = MonkeyMovement::Idle;
//                 }
//             }
//         }
//
//         if *previous_movement != monkey.current_movement {
//             *previous_movement = monkey.current_movement;
//             commands.spawn(AudioBundle {
//                 source: rock_run_assets.monkey_rush_sound.clone(),
//                 settings: PlaybackSettings {
//                     mode: PlaybackMode::Despawn,
//                     ..default()
//                 },
//             });
//         }
//
//         anim(monkey.current_movement, &mut commands);
//     }
// }

fn despawn_monkey(mut commands: Commands, monkey: Query<Entity, With<Monkey>>) {
    for monkey in monkey.iter() {
        commands.entity(monkey).despawn_recursive();
    }
}
