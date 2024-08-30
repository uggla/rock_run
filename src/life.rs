use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

use crate::{
    assets::RockRunAssets,
    coregame::{
        camera::CameraSet,
        colliders::ColliderName,
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{ExtraLifeCollision, LifeEvent},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};

const LIFE_SCALE_FACTOR: f32 = 2.0;
const LIFE_WIDTH: f32 = 16.0;
const LIFE_HEIGHT: f32 = 16.0;

#[derive(Resource, Default)]
pub struct Life {
    entities: Vec<Entity>,
}

#[derive(Component)]
pub struct LifeUI;

#[derive(Component)]
pub struct ExtraLife;

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), (setup_life, setup_extralife))
            .add_systems(OnEnter(AppState::NextLevel), setup_extralife)
            .add_systems(
                OnEnter(AppState::StartMenu),
                (despawn_life, despawn_extralife),
            )
            .add_systems(OnEnter(AppState::FinishLevel), despawn_extralife)
            .add_systems(Update, life_management)
            .insert_resource(Life::default())
            .add_systems(
                Update,
                (show_life, check_get_extralife)
                    .after(CameraSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .add_event::<LifeEvent>();
    }
}

fn setup_life(mut commands: Commands, rock_run_assets: Res<RockRunAssets>, mut life: ResMut<Life>) {
    let texture = &rock_run_assets.life;

    let parent = spawn_life_entity(&mut commands, &life, texture);
    commands.entity(parent).insert(LifeUI);
    life.entities.push(parent);

    for _ in 0..2 {
        let child = spawn_life_entity(&mut commands, &life, texture);
        commands.entity(parent).add_child(child);
        life.entities.push(child);
    }
}

fn spawn_life_entity(
    commands: &mut Commands,
    life: &ResMut<Life>,
    texture: &Handle<Image>,
) -> Entity {
    let x_offset = life.entities.len() as f32 * 20.0;
    let child = commands
        .spawn(SpriteBundle {
            texture: texture.clone(),
            sprite: Sprite { ..default() },
            transform: Transform {
                translation: Vec3::new(x_offset, 0.0, 0.0),
                ..default()
            },
            ..default()
        })
        .id();
    child
}

fn show_life(
    mut life_query: Query<&mut Transform, With<LifeUI>>,
    camera_query: Query<&mut Transform, (With<Camera2d>, Without<LifeUI>)>,
) {
    let mut life_ui = match life_query.get_single_mut() {
        Ok(life) => life,
        Err(_) => return,
    };

    let camera = camera_query.single();

    #[cfg(not(target_arch = "wasm32"))]
    const TOP_MARGIN: f32 = 38.0;

    #[cfg(target_arch = "wasm32")]
    const TOP_MARGIN: f32 = 20.0;

    life_ui.translation = camera.translation
        + Vec3::new(
            -WINDOW_WIDTH / 2.0 + 20.0,
            WINDOW_HEIGHT / 2.0 - TOP_MARGIN,
            100.0,
        );
    life_ui.scale = Vec3::splat(LIFE_SCALE_FACTOR);
}

fn life_management(
    mut commands: Commands,
    mut life_ui: Query<&Handle<Image>, With<LifeUI>>,
    mut life: ResMut<Life>,
    mut life_event: EventReader<LifeEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in life_event.read() {
        match ev {
            LifeEvent::Win => {
                let texture = life_ui.single_mut();
                let child = spawn_life_entity(&mut commands, &life, texture);
                commands.entity(life.entities[0]).add_child(child);
                life.entities.push(child);
            }
            LifeEvent::Lost => match life.entities.pop() {
                Some(entity) => {
                    commands.entity(entity).despawn_recursive();
                    debug!("life left: {}", life.entities.len());
                    if life.entities.is_empty() {
                        next_state.set(AppState::GameOver);
                    }
                }
                None => {
                    unreachable!("No life left to despawn");
                }
            },
        }
    }
}

fn despawn_life(
    mut commands: Commands,
    life_ui: Query<Entity, With<LifeUI>>,
    mut life: ResMut<Life>,
) {
    if let Ok(life_ui) = life_ui.get_single() {
        commands.entity(life_ui).despawn_recursive();
        life.entities.clear();
    }
}

fn setup_extralife(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_extralife");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = &rock_run_assets.life;
    let mut extra_life_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    extra_life_pos.insert(
        1,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(1056.0, 112.0))],
    );

    let start_positions = match extra_life_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(LIFE_SCALE_FACTOR),
                    translation: start_pos.extend(10.0),
                    ..default()
                },
                ..default()
            },
            Collider::cuboid(LIFE_WIDTH / 2.0, LIFE_HEIGHT / 2.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::KINEMATIC_STATIC,
            ExtraLife,
            ColliderName("extralife01".to_string()),
        ));
    }
}

fn check_get_extralife(
    mut commands: Commands,
    mut life_event: EventWriter<LifeEvent>,
    mut extralive_collision: EventReader<ExtraLifeCollision>,
) {
    for ev in extralive_collision.read() {
        commands.entity(ev.entity).despawn_recursive();
        life_event.send(LifeEvent::Win);
    }
}

fn despawn_extralife(mut commands: Commands, entities: Query<Entity, With<ExtraLife>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
