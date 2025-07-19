use bevy::{
    audio::{PlaybackMode, Volume},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

use crate::{
    WINDOW_HEIGHT, WINDOW_WIDTH,
    assets::RockRunAssets,
    coregame::{
        camera::CameraSet,
        colliders::ColliderName,
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{ExtraLifeCollision, LifeEvent},
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
            .add_systems(
                Update,
                life_management.run_if(not(in_state(AppState::Loading))),
            )
            .add_systems(
                Update,
                (show_life, check_get_extralife)
                    .after(CameraSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .insert_resource(Life::default())
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
    commands
        .spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            Transform {
                translation: Vec3::new(x_offset, 0.0, 0.0),
                ..default()
            },
        ))
        .id()
}

fn show_life(
    mut life_query: Query<&mut Transform, With<LifeUI>>,
    camera_query: Query<&mut Transform, (With<Camera2d>, Without<LifeUI>)>,
) -> Result<()> {
    let mut life_ui = match life_query.single_mut() {
        Ok(life) => life,
        Err(_) => return Ok(()),
    };

    let camera = camera_query.single()?;

    const TOP_MARGIN: f32 = 20.0;

    life_ui.translation = camera.translation
        + Vec3::new(
            -WINDOW_WIDTH / 2.0 + 20.0,
            WINDOW_HEIGHT / 2.0 - TOP_MARGIN,
            100.0,
        );
    life_ui.scale = Vec3::splat(LIFE_SCALE_FACTOR);
    Ok(())
}

fn life_management(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut life_ui: Query<&Sprite, With<LifeUI>>,
    mut life: ResMut<Life>,
    mut life_event: EventReader<LifeEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result<()> {
    for ev in life_event.read() {
        match ev {
            LifeEvent::Win => {
                let sprite = life_ui.single_mut()?;
                let child = spawn_life_entity(&mut commands, &life, &sprite.image);
                commands.entity(life.entities[0]).add_child(child);
                life.entities.push(child);
                commands.spawn((
                    AudioPlayer::new(rock_run_assets.get_something_sound.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::Linear(0.8),
                        ..default()
                    },
                ));
            }
            LifeEvent::Lost => match life.entities.pop() {
                Some(entity) => {
                    commands.entity(entity).despawn();
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
    Ok(())
}

fn despawn_life(
    mut commands: Commands,
    life_ui: Query<Entity, With<LifeUI>>,
    mut life: ResMut<Life>,
) {
    if let Ok(life_ui) = life_ui.single() {
        commands.entity(life_ui).despawn();
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

    extra_life_pos.insert(
        2,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(5416.0, 1518.0))],
    );

    let start_positions = match extra_life_pos.get(&current_level.id) {
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
                scale: Vec3::splat(LIFE_SCALE_FACTOR),
                translation: start_pos.extend(10.0),
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
        commands.entity(ev.entity).despawn();
        life_event.write(LifeEvent::Win);
    }
}

fn despawn_extralife(mut commands: Commands, entities: Query<Entity, With<ExtraLife>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
