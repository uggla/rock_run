use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
};

use crate::{
    collision::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::TriceratopsCollision,
    helpers::texture::cycle_texture,
};

pub const TRICERATOPS_SPEED: f32 = 300.0;
pub const TRICERATOPS_SCALE_FACTOR: f32 = 1.0;
pub const TRICERATOPS_WIDTH: f32 = 175.0;
pub const TRICERATOPS_HEIGHT: f32 = 120.0;

#[derive(Component)]
pub struct Triceratops {
    current_movement: TriceratopsMovement,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect)]
pub enum TriceratopsMovement {
    Run(TriceratopsDirection),
}

impl Default for TriceratopsMovement {
    fn default() -> Self {
        Self::Run(TriceratopsDirection::default())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Reflect, Default)]
pub enum TriceratopsDirection {
    Left,
    #[default]
    Right,
}

pub struct TriceratopsPlugin;

impl Plugin for TriceratopsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_triceratops)
            .add_systems(OnEnter(AppState::StartMenu), despawn_triceratops)
            .add_systems(
                Update,
                move_triceratops
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn get_collider_shapes(y_mirror: bool) -> Vec<(Vec2, f32, Collider)> {
    let shapes = vec![
        (
            //head
            Vec2::new(37.0, 16.0),
            0.0,
            Collider::cuboid(74.0 / 2.0, 55.0 / 2.0),
        ),
        (
            // body
            Vec2::new(4.0, -25.0),
            0.0,
            Collider::cuboid(68.0 / 2.0, 68.0 / 2.0),
        ),
        (
            // tail
            Vec2::new(-48.0, -30.0),
            0.0,
            Collider::cuboid(34.0 / 2.0, 26.0 / 2.0),
        ),
    ];

    if y_mirror {
        shapes
            .into_iter()
            .map(|(pos, angle, shape)| (pos * Vec2::new(-1.0, 1.0), angle, shape))
            .collect()
    } else {
        shapes
    }
}

fn setup_triceratops(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_triceratops");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = asset_server.load("triceratops-1.png");
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new(TRICERATOPS_WIDTH, TRICERATOPS_HEIGHT),
        5,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);
    let mut level_triceratops_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    level_triceratops_pos.insert(
        1,
        vec![
            level.map.tiled_to_bevy_coord(Vec2::new(2400.0, 480.0)),
            level.map.tiled_to_bevy_coord(Vec2::new(6020.0, 1050.0)),
        ],
    );

    let start_positions = match level_triceratops_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            SpriteSheetBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                atlas: TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                },
                transform: Transform {
                    scale: Vec3::splat(TRICERATOPS_SCALE_FACTOR),
                    translation: start_pos.extend(20.0),
                    ..default()
                },
                ..default()
            },
            RigidBody::KinematicPositionBased,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Collider::compound(get_collider_shapes(false)),
            KinematicCharacterController { ..default() },
            Triceratops {
                current_movement: TriceratopsMovement::Run(TriceratopsDirection::default()),
            },
        ));
    }
}

fn move_triceratops(
    time: Res<Time>,
    mut triceratops_query: Query<
        (
            Entity,
            &mut Triceratops,
            &mut Collider,
            &mut KinematicCharacterController,
        ),
        With<Triceratops>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
    mut collision_event: EventReader<TriceratopsCollision>,
) {
    let collided_triceratops_entities: Vec<Entity> =
        collision_event.read().map(|ev| ev.id).collect();
    for (triceratos_entity, mut triceratos, mut triceratops_collider, mut triceratops_controller) in
        triceratops_query.iter_mut()
    {
        let mut anim = |current_movement: TriceratopsMovement| match current_movement {
            TriceratopsMovement::Run(triceratops_direction) => {
                let (mut anim_timer, mut texture, mut sprite) =
                    animation_query.get_mut(triceratos_entity).unwrap();
                anim_timer.tick(time.delta());
                match triceratops_direction {
                    TriceratopsDirection::Left => {
                        sprite.flip_x = true;
                        *triceratops_collider = Collider::compound(get_collider_shapes(true));
                    }
                    TriceratopsDirection::Right => {
                        sprite.flip_x = false;
                        *triceratops_collider = Collider::compound(get_collider_shapes(false));
                    }
                }
                if anim_timer.just_finished() {
                    cycle_texture(&mut texture, 0..=4);
                }
            }
        };

        if collided_triceratops_entities.contains(&triceratos_entity) {
            if triceratos.current_movement == TriceratopsMovement::Run(TriceratopsDirection::Right)
            {
                triceratos.current_movement = TriceratopsMovement::Run(TriceratopsDirection::Left);
            } else {
                triceratos.current_movement = TriceratopsMovement::Run(TriceratopsDirection::Right);
            }
        }

        let direction_x = match triceratos.current_movement {
            TriceratopsMovement::Run(triceratops_direction) => match triceratops_direction {
                TriceratopsDirection::Left => -1.0,
                TriceratopsDirection::Right => 1.0,
            },
        };
        anim(triceratos.current_movement);
        triceratops_controller.translation = Some(Vec2::new(
            direction_x * TRICERATOPS_SPEED * time.delta_seconds(),
            -TRICERATOPS_SPEED * time.delta_seconds(),
        ));
    }
}

fn despawn_triceratops(mut commands: Commands, triceratops: Query<Entity, With<Triceratops>>) {
    for triceratops in triceratops.iter() {
        commands.entity(triceratops).despawn_recursive();
    }
}
