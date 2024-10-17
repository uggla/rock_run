use crate::{
    assets::RockRunAssets,
    coregame::{
        colliders::ColliderName,
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{NextLevel, PositionSensorCollisionStart, Restart, ShakeCamera, StartGame},
};

use bevy::{
    audio::PlaybackMode,
    color,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::HashMap,
};

use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, ExternalImpulse, RigidBody, Sensor,
};

use rand::{thread_rng, Rng};

const FIREBALL_SCALE_FACTOR: f32 = 1.0;
const VOLCANO_SCALE_FACTOR: f32 = 0.48;

#[derive(Component)]
pub struct Volcano;

#[derive(Component)]
pub struct Lava;

#[derive(Component)]
pub struct Fireball;

pub struct VolcanoPlugin;

impl Plugin for VolcanoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), (setup_volcano, setup_lava))
            .add_systems(OnEnter(AppState::NextLevel), (setup_volcano, setup_lava))
            .add_systems(
                OnEnter(AppState::StartMenu),
                (despawn_volcano, despawn_fireballs, despawn_lava),
            )
            .add_systems(
                OnEnter(AppState::FinishLevel),
                (despawn_volcano, despawn_fireballs, despawn_lava),
            )
            .add_systems(
                Update,
                (spawn_fireball).run_if(in_state(AppState::GameRunning)),
            );
        app.add_plugins(Material2dPlugin::<LavaMaterial>::default());
    }
}

fn setup_volcano(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_volcano");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.volcano.clone();
    let mut level_volcano_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    level_volcano_pos.insert(
        1,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(10000.0, 250.0))],
    );

    let start_positions = match level_volcano_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(VOLCANO_SCALE_FACTOR),
                    translation: start_pos.extend(2.0),
                    ..default()
                },
                ..default()
            },
            Volcano,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_fireball(
    mut commands: Commands,
    time: Res<Time>,
    rock_run_assets: Res<RockRunAssets>,
    mut fireball_sensor_collision: EventReader<PositionSensorCollisionStart>,
    mut fireballs: Local<bool>,
    mut spawn_timer: Local<Timer>,
    mut spawn_pos: Local<Vec2>,
    mut game_event: EventReader<StartGame>,
    mut restart_event: EventReader<Restart>,
    mut next_level_event: EventReader<NextLevel>,
    mut shake_event: EventWriter<ShakeCamera>,
) {
    if !game_event.is_empty() {
        *fireballs = false;
        game_event.clear();
        return;
    }

    if !restart_event.is_empty() {
        *fireballs = false;
        restart_event.clear();
        return;
    }

    if !next_level_event.is_empty() {
        *fireballs = false;
        next_level_event.clear();
        return;
    }

    spawn_timer.tick(time.delta());

    for collision_event in fireball_sensor_collision.read() {
        if !collision_event.sensor_name.contains("volcano") {
            return;
        }

        if collision_event.sensor_name.contains("volcano01_02") {
            shake_event.send(ShakeCamera);
            commands.spawn(AudioBundle {
                source: rock_run_assets.eruption_sound.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
            return;
        }

        *fireballs = true;
        *spawn_pos = collision_event.spawn_pos;
        *spawn_timer = Timer::from_seconds(0.1, TimerMode::Repeating);
        shake_event.send(ShakeCamera);
        commands.spawn(AudioBundle {
            source: rock_run_assets.eruption_sound.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }

    if *fireballs && spawn_timer.finished() {
        let mut rng = thread_rng();
        let impulse_x: f32 = rng.gen_range(-15.0..=15.0);
        let impulse_y: f32 = rng.gen_range(3.0..=4.0);
        let torque_impulse: f32 = rng.gen_range(-2.5..=2.5);

        commands.spawn((
            SpriteBundle {
                texture: rock_run_assets.fireball.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(FIREBALL_SCALE_FACTOR),
                    translation: spawn_pos.extend(20.0),
                    ..default()
                },
                ..default()
            },
            RigidBody::Dynamic,
            Collider::ball(16.0),
            ExternalImpulse {
                impulse: Vec2::new(50000.0 * impulse_x, 100000.0 * impulse_y),
                torque_impulse: 1000000.0 * torque_impulse,
            },
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::DYNAMIC_KINEMATIC,
            ColliderName("fireball".to_string()),
            Fireball,
        ));
    }
}

fn setup_lava(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lava: ResMut<Assets<LavaMaterial>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_lava");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let mut level_lava_pos: HashMap<u8, Vec<Vec2>> = HashMap::new();
    level_lava_pos.insert(
        2,
        vec![level.map.tiled_to_bevy_coord(Vec2::new(3520.0, 2128.0))],
    );

    let start_positions = match level_lava_pos.get(&current_level.id) {
        Some(positions) => positions,
        None => return,
    };

    for start_pos in start_positions {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                transform: Transform {
                    translation: start_pos.extend(5.0),
                    scale: Vec3::new(2000.0, 66.0, 1.0),
                    ..default()
                },
                material: lava.add(LavaMaterial {
                    // color_texture: Some(asset_server.load("icon.png")),
                    color: LinearRgba::from(color::palettes::css::GOLD),
                }),
                ..default()
            },
            Lava,
        ));
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct LavaMaterial {
    #[uniform(0)]
    color: LinearRgba,
    // #[texture(1)]
    // #[sampler(2)]
    // color_texture: Option<Handle<Image>>,
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for LavaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lava_material.wgsl".into()
    }
}

fn despawn_volcano(mut commands: Commands, volcano: Query<Entity, With<Volcano>>) {
    for volcano in volcano.iter() {
        commands.entity(volcano).despawn_recursive();
    }
}

fn despawn_fireballs(mut commands: Commands, fireballs: Query<Entity, With<Fireball>>) {
    for fireball in fireballs.iter() {
        commands.entity(fireball).despawn_recursive();
    }
}

fn despawn_lava(mut commands: Commands, lavas: Query<Entity, With<Lava>>) {
    for lava in lavas.iter() {
        commands.entity(lava).despawn_recursive();
    }
}
