use bevy::{
    audio::{PlaybackMode, Volume},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::RigidBody,
    geometry::Collider,
    pipeline::QueryFilterFlags,
    prelude::{ActiveCollisionTypes, ActiveEvents, CollisionGroups, Group, Sensor},
};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::{
        colliders::{ColliderName, Ladder, Spike},
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{EnigmaResult, NextLevel, NutCollision, StartGame},
    helpers::texture::cycle_texture,
};

const SQUIREL_SPEED: f32 = 500.0;
const SQUIREL_SCALE_FACTOR: f32 = 1.5;
const SQUIREL_WIDTH: f32 = 58.0;
const SQUIREL_HEIGHT: f32 = 32.0;
const NUT_SCALE_FACTOR: f32 = 2.0;
const NUT_WIDTH: f32 = 12.0;
const NUT_HEIGHT: f32 = 12.0;
const VINE_SCALE_FACTOR: f32 = 1.0;
const VINE_HEIGHT: f32 = 32.0;
const VINE_ROOT_HEIGHT: f32 = 24.0;
const VINE_CHUNK_HEIGHT: f32 = 16.0;

#[derive(Component)]
pub struct Squirel {
    current_movement: SquirelMovement,
    end_pos: Vec2,
}

struct Squirels {
    pos: Vec2,
    end_pos: Vec2,
    visibility: Visibility,
}

#[derive(Clone)]
struct VineData {
    pos: Vec2,
    size: usize,
    associated_enigma: String,
}

#[derive(Component)]
struct Vine {
    count: usize,
    associated_enigma: String,
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

#[derive(Component)]
pub struct Nut;

#[derive(Resource, Default, Debug)]
pub struct Nuts {
    entities: Vec<Entity>,
}

impl Nuts {
    pub fn len(&self) -> usize {
        self.entities.len()
    }
}

pub struct SquirelPlugin;

impl Plugin for SquirelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::StartMenu),
            (despawn_squirel, despawn_nuts, despawn_vines),
        )
        .add_systems(
            OnEnter(AppState::FinishLevel),
            (despawn_squirel, despawn_nuts, despawn_vines),
        )
        .add_systems(OnEnter(AppState::GameCreate), (setup_squirels, setup_nuts))
        .add_systems(
            Update,
            (move_squirel, check_get_nut, unroll_vine)
                .after(CollisionSet)
                // .run_if(in_state(AppState::GameRunning)),
                .run_if(not(in_state(AppState::Loading))),
        )
        .insert_resource(Nuts::default());
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

fn setup_squirels(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_squirels");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.squirel.clone();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(SQUIREL_WIDTH as u32, SQUIREL_HEIGHT as u32),
        14,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);

    let mut level_squirels: HashMap<u8, Vec<Squirels>> = HashMap::new();
    level_squirels.insert(
        1,
        vec![
            Squirels {
                pos: level.map.tiled_to_bevy_coord(Vec2::new(
                    5189.0,
                    461.0 - SQUIREL_HEIGHT * SQUIREL_SCALE_FACTOR / 2.0,
                )),
                end_pos: level.map.tiled_to_bevy_coord(Vec2::new(7300.0, 600.0)),
                visibility: Visibility::Visible,
            },
            Squirels {
                pos: level.map.tiled_to_bevy_coord(Vec2::new(
                    7290.0,
                    70.0 - SQUIREL_HEIGHT * SQUIREL_SCALE_FACTOR / 2.0,
                )),
                end_pos: level.map.tiled_to_bevy_coord(Vec2::new(12714.0, 610.0)),
                visibility: Visibility::Hidden,
            },
        ],
    );

    let squirels = match level_squirels.get(&current_level.id) {
        Some(squirels) => squirels,
        None => return,
    };

    for squirel in squirels {
        commands.spawn((
            Sprite {
                image: texture.clone(),
                flip_x: true,
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            squirel.visibility,
            Transform {
                scale: Vec3::splat(SQUIREL_SCALE_FACTOR),
                translation: squirel.pos.extend(10.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.12, TimerMode::Repeating)),
            Collider::compound(get_collider_shapes(false)),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::EXCLUDE_SENSORS,
                filter_groups: Some(CollisionGroups::new(Group::GROUP_3, Group::GROUP_3)),
                max_slope_climb_angle: 85.0f32.to_radians(),
                min_slope_slide_angle: 85.0f32.to_radians(),
                ..default()
            },
            CollisionGroups::new(Group::GROUP_3, Group::GROUP_3),
            Squirel {
                current_movement: SquirelMovement::Idle,
                end_pos: squirel.end_pos,
            },
        ));
    }
}

fn setup_nuts(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut collected_nuts: ResMut<Nuts>,
) {
    info!("setup_nuts");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = &rock_run_assets.nut;
    let mut nuts_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    nuts_pos.insert(
        1,
        vec![
            level.map.tiled_to_bevy_coord(Vec2::new(5600.0, 446.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5528.0, 302.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5258.0, 248.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5312.0, 452.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5760.0, 612.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(6100.0, 612.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(6300.0, 612.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(6496.0, 612.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(6625.0, 612.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5323.0, 300.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(5427.0, 288.0)),
        ],
    );

    let start_positions = match nuts_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            Transform {
                scale: Vec3::splat(NUT_SCALE_FACTOR),
                translation: start_pos.extend(10.0),
                ..default()
            },
            Collider::cuboid(NUT_WIDTH / 2.0, NUT_HEIGHT / 2.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::KINEMATIC_STATIC,
            Nut,
            ColliderName("nut01".to_string()),
        ));

        collected_nuts.entities.clear();
    }
}

fn check_get_nut(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut extralive_collision: EventReader<NutCollision>,
    mut collected_nuts: ResMut<Nuts>,
) {
    for ev in extralive_collision.read() {
        collected_nuts.entities.push(ev.entity);
        debug!("Collected nuts {}", collected_nuts.entities.len());
        commands.entity(ev.entity).despawn();
        commands.spawn((
            AudioPlayer::new(rock_run_assets.get_something_sound.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(0.8),
                ..default()
            },
        ));
    }
}

type SquirelData<'a> = (
    Entity,
    &'a mut Collider,
    &'a mut KinematicCharacterController,
    &'a mut Transform,
    &'a mut Visibility,
    &'a mut Squirel,
);

#[allow(clippy::too_many_arguments)]
fn move_squirel(
    mut commands: Commands,
    time: Res<Time>,
    mut squirel_query: Query<SquirelData, With<Squirel>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    mut enigna_result: EventReader<EnigmaResult>,
    spikes: Query<&Transform, (With<Spike>, Without<Squirel>)>,
) {
    let nb_squirels = squirel_query.iter().count();

    for (
        squirel_entity,
        mut squirel_collider,
        mut squirel_controller,
        squirel_pos,
        mut squirel_visibility,
        mut squirel,
    ) in squirel_query.iter_mut()
    {
        let mut anim = |current_movement: SquirelMovement| match current_movement {
            SquirelMovement::Run(squirel_direction) => {
                let (mut anim_timer, mut sprite) = animation_query.get_mut(squirel_entity).unwrap();
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
                    if let Some(texture) = &mut sprite.texture_atlas {
                        cycle_texture(texture, 8..=13);
                    }
                }
            }

            SquirelMovement::Idle => {
                let (mut anim_timer, mut sprite) = animation_query.get_mut(squirel_entity).unwrap();
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    if let Some(texture) = &mut sprite.texture_atlas {
                        cycle_texture(texture, 0..=7);
                    }
                }
            }
        };

        let squirel_pos = squirel_pos.translation.xy();

        if nb_squirels == 1 {
            *squirel_visibility = Visibility::Visible;
        }

        for ev in enigna_result.read() {
            if let EnigmaResult::Correct(enigma) = ev {
                if enigma == "story05-04" || enigma == "story100-03" {
                    squirel.current_movement = SquirelMovement::Run(SquirelDirection::Right);
                }
            }
        }

        anim(squirel.current_movement);

        if squirel.current_movement == SquirelMovement::Run(SquirelDirection::Right) {
            if squirel_pos.distance(squirel.end_pos) < 32.0 {
                commands.entity(squirel_entity).despawn();
            }
            if spikes
                .iter()
                .any(|s| (s.translation.x - squirel_pos.x).abs() < 3.0 * 16.0)
            {
                squirel_controller.translation = Some(Vec2::new(
                    SQUIREL_SPEED * time.delta_secs(),
                    150.0 * time.delta_secs(),
                ));
            } else {
                squirel_controller.translation = Some(Vec2::new(
                    SQUIREL_SPEED * time.delta_secs(),
                    -400.0 * time.delta_secs(),
                ));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn unroll_vine(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    time: Res<Time>,
    mut animation_timer: Local<Timer>,
    mut vines: Local<Vec<VineData>>,
    mut enigna_result: EventReader<EnigmaResult>,
    mut vine_query: Query<(Entity, &mut Vine)>,
    mut game_event_start: EventReader<StartGame>,
    mut game_event_level: EventReader<NextLevel>,
    state: Res<State<AppState>>,
) {
    if !game_event_start.is_empty() || !game_event_level.is_empty() {
        vines.clear();
        game_event_start.clear();
        game_event_level.clear();
        return;
    }

    for ev in enigna_result.read() {
        if let EnigmaResult::Correct(enigma) = ev {
            let level = levels
                .iter()
                .find(|level| level.id == current_level.id)
                .unwrap();

            let mut enigma_vine: HashMap<String, VineData> = HashMap::new();
            enigma_vine.insert(
                "story100-03".to_string(),
                VineData {
                    pos: level.map.tiled_to_bevy_coord(Vec2::new(
                        7272.0,
                        66.0 - SQUIREL_HEIGHT * SQUIREL_SCALE_FACTOR / 2.0,
                    )),
                    size: 35,
                    associated_enigma: "story100-03".to_string(),
                },
            );

            enigma_vine.insert(
                "story101-03".to_string(),
                VineData {
                    pos: level.map.tiled_to_bevy_coord(Vec2::new(12568.0, 1538.0)),
                    size: 31,
                    associated_enigma: "story101-03".to_string(),
                },
            );
            match enigma_vine.get(enigma) {
                Some(vine_data) => {
                    *animation_timer = Timer::from_seconds(0.1, TimerMode::Repeating);
                    vines.push(vine_data.clone());
                    display_vine(&mut commands, &rock_run_assets, vine_data, &mut vine_query)
                }
                None => return,
            };
        }
    }

    animation_timer.tick(time.delta());
    if *state == AppState::GameRunning && animation_timer.just_finished() {
        for vine_data in vines.iter() {
            display_vine(&mut commands, &rock_run_assets, vine_data, &mut vine_query)
        }
    }
}

fn display_vine(
    commands: &mut Commands,
    rock_run_assets: &Res<RockRunAssets>,
    vine_data: &VineData,
    vine_query: &mut Query<(Entity, &mut Vine)>,
) {
    let texture_vine1 = rock_run_assets.vine1.clone();
    let texture_vine2 = rock_run_assets.vine2.clone();
    let texture_vine2_end = rock_run_assets.vine2_end.clone();
    let texture_vine_left = rock_run_assets.vine_left.clone();
    let texture_vine_left_end = rock_run_assets.vine_left_end.clone();
    let texture_vine_right = rock_run_assets.vine_right.clone();
    let texture_vine_right_end = rock_run_assets.vine_right_end.clone();

    let (vine_entity, vine) = match vine_query
        .iter_mut()
        .find(|(_entity, vine)| vine.associated_enigma == vine_data.associated_enigma)
    {
        Some(vine) => vine,
        None => {
            commands.spawn((
                get_vine_sprite_bundle(&texture_vine1, vine_data.pos.extend(10.0)),
                Vine {
                    count: 0,
                    associated_enigma: vine_data.associated_enigma.clone(),
                },
                Ladder,
                Collider::compound(vec![(
                    Vec2::new(0.0, -0.0),
                    0.0,
                    Collider::cuboid(1.0, VINE_HEIGHT / 2.0),
                )]),
                ColliderName(vine_data.associated_enigma.to_string()),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::KINEMATIC_STATIC,
            ));

            return;
        }
    };

    if vine.count == vine_data.size {
        return;
    }

    if vine.count == 0 {
        commands.entity(vine_entity).despawn();

        commands
            .spawn((
                get_vine_sprite_bundle(&texture_vine2, vine_data.pos.extend(10.0)),
                Vine {
                    count: 1,
                    associated_enigma: vine_data.associated_enigma.clone(),
                },
                Ladder,
                Collider::compound(vec![(
                    Vec2::new(0.0, -8.0),
                    0.0,
                    Collider::cuboid(1.0, VINE_HEIGHT / 2.0),
                )]),
                ColliderName(vine_data.associated_enigma.to_string()),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::KINEMATIC_STATIC,
            ))
            .with_children(|builder| {
                builder.spawn(get_vine_sprite_bundle(
                    &texture_vine2_end,
                    Vec3::new(0.0, -VINE_ROOT_HEIGHT, 0.0),
                ));
            });
    }

    if vine.count >= 1 {
        commands.entity(vine_entity).despawn();

        // Display vine root
        let parent = commands
            .spawn((
                get_vine_sprite_bundle(&texture_vine2, vine_data.pos.extend(10.0)),
                Vine {
                    count: vine.count + 1,
                    associated_enigma: vine_data.associated_enigma.clone(),
                },
            ))
            .id();

        // Display vine chunks
        let mut count = 0;
        let offset = match count {
            0 => VINE_ROOT_HEIGHT,
            _ => 0.0,
        };
        for i in 0..vine.count {
            let texture = match i % 2 {
                0 => texture_vine_right.clone(),
                _ => texture_vine_left.clone(),
            };
            let child = commands
                .spawn(get_vine_sprite_bundle(
                    &texture,
                    Vec3::new(0.0, -VINE_CHUNK_HEIGHT * count as f32 - offset, 0.0),
                ))
                .id();
            commands.entity(parent).add_child(child);
            count += 1;
        }

        // Display end of vine
        let texture = match count % 2 {
            0 => texture_vine_left_end.clone(),
            _ => texture_vine_right_end.clone(),
        };
        let child = commands
            .spawn(get_vine_sprite_bundle(
                &texture,
                Vec3::new(
                    0.0,
                    -VINE_CHUNK_HEIGHT * count as f32 - VINE_ROOT_HEIGHT,
                    0.0,
                ),
            ))
            .id();
        commands.entity(parent).add_child(child);
        commands.entity(parent).insert(Ladder);
        commands.entity(parent).insert(Collider::compound(vec![(
            Vec2::new(
                0.0,
                -(VINE_CHUNK_HEIGHT * count as f32 + VINE_CHUNK_HEIGHT / 2.0) / 2.0,
            ),
            0.0,
            Collider::cuboid(
                1.0,
                (VINE_CHUNK_HEIGHT * count as f32 + VINE_ROOT_HEIGHT + VINE_CHUNK_HEIGHT) / 2.0,
            ),
        )]));
        commands
            .entity(parent)
            .insert(ColliderName(vine_data.associated_enigma.to_string()));
        commands.entity(parent).insert(Sensor);
        commands
            .entity(parent)
            .insert(ActiveEvents::COLLISION_EVENTS);
        commands
            .entity(parent)
            .insert(ActiveCollisionTypes::KINEMATIC_STATIC);
    }
}

fn get_vine_sprite_bundle(texture: &Handle<Image>, vine_pos: Vec3) -> (Sprite, Transform) {
    (
        Sprite {
            image: texture.clone(),
            ..default()
        },
        Transform {
            scale: Vec3::splat(VINE_SCALE_FACTOR),
            translation: vine_pos,
            ..default()
        },
    )
}

fn despawn_squirel(mut commands: Commands, squirels: Query<Entity, With<Squirel>>) {
    for squirel in squirels.iter() {
        commands.entity(squirel).despawn();
    }
}

fn despawn_nuts(mut commands: Commands, nuts: Query<Entity, With<Nut>>) {
    for nut in nuts.iter() {
        commands.entity(nut).despawn();
    }
}

fn despawn_vines(mut commands: Commands, vines: Query<Entity, With<Vine>>) {
    for vine in vines.iter() {
        commands.entity(vine).despawn();
    }
}
