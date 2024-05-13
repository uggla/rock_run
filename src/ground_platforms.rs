use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::{Ccd, ExternalImpulse, GravityScale, RigidBody},
    geometry::{Collider, Restitution, Sensor},
};
use tiled::ObjectShape;

use crate::{
    helpers::tiled::TiledMap,
    level::{CurrentLevel, Level},
    player::PlayerState,
    state::AppState,
};

pub struct GroundAndPlatformsPlugin;

impl Plugin for GroundAndPlatformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_ground_platforms_spikes)
            .add_systems(
                OnEnter(AppState::StartMenu),
                despawn_ground_platforms_spikes,
            )
            .add_systems(
                Update,
                (print_ball_altitude, apply_forces).run_if(in_state(AppState::GameRunning)),
            );
    }
}

#[derive(Component, Clone, Debug)]
pub struct Ground;

#[derive(Component, Clone, Debug)]
pub struct Platform;

fn tiled_object_to_collider<T: Component + Clone>(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    level: &Level,
    bridge: LayerComponentBridge<T>,
) {
    tiled_map.map.layers().for_each(|layer| {
        if layer.name != bridge.layer {
            return;
        }

        info!("Found {} layer", bridge.layer);

        let object_data = match layer.layer_type() {
            tiled::LayerType::Objects(object_data) => object_data,
            _ => return,
        };

        object_data.objects().for_each(|object| {
            debug!("Found object {:?}", object.name);
            debug!("Shape {:?}", object.shape);

            match &object.shape {
                ObjectShape::Rect { width, height } => {
                    let Vec2 { x, y } = level.map.tiled_to_bevy_coord(Vec2::new(
                        object.x + *width / 2.0,
                        object.y + *height / 2.0,
                    ));
                    commands
                        .spawn((
                            Collider::cuboid(*width / 2.0, *height / 2.0),
                            bridge.component.clone(),
                            TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                        ))
                        .insert(Ccd::enabled());
                }
                ObjectShape::Polygon { points } => {
                    let points: Vec<Vec2> = points
                        .iter()
                        .map(|(x, y)| {
                            level.map.tiled_to_bevy_coord(
                                Vec2::new(*x, *y) + Vec2::new(object.x, object.y),
                            )
                        })
                        .collect();

                    debug!("Polygon points: {:?}", points);

                    match Collider::convex_hull(&points) {
                        Some(collider) => {
                            commands
                                .spawn((collider, bridge.component.clone()))
                                .insert(Ccd::enabled());
                        }
                        None => {
                            error!("Failed to create convex hull");
                        }
                    }
                }
                ObjectShape::Polyline { points } => {
                    let points: Vec<Vec2> = points
                        .iter()
                        .map(|(x, y)| {
                            level.map.tiled_to_bevy_coord(
                                Vec2::new(*x, *y) + Vec2::new(object.x, object.y),
                            )
                        })
                        .collect();

                    debug!("Polyline points: {:?}", points);

                    commands
                        .spawn((Collider::polyline(points, None), bridge.component.clone()))
                        .insert(Ccd::enabled());
                }
                ObjectShape::Text { .. } => {
                    warn!("Text shape not supported");
                }
                ObjectShape::Ellipse { width, height } => {
                    let Vec2 { x, y } = level.map.tiled_to_bevy_coord(Vec2::new(
                        object.x + *width / 2.0,
                        object.y + *height / 2.0,
                    ));
                    if *width != *height {
                        warn!("Ellipse shape not supported: {:?}x{:?}", width, height);
                    }
                    commands
                        .spawn((
                            Collider::ball(*width / 2.0),
                            bridge.component.clone(),
                            // Platform,
                            TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                        ))
                        .insert(Ccd::enabled());
                }
                ObjectShape::Point(x, y) => {
                    // Used as a sensor
                    let Vec2 { x, y } = level.map.tiled_to_bevy_coord(Vec2::new(*x, *y));
                    commands
                        .spawn((
                            Collider::cuboid(1.0, 1.0),
                            bridge.component.clone(),
                            TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                        ))
                        .insert(Sensor)
                        .insert(Ccd::enabled());
                }
            }
        })
    })
}

struct LayerComponentBridge<'a, T: Component + Clone> {
    layer: &'a str,
    component: T,
}

impl<'a, T: Component + Clone> LayerComponentBridge<'a, T> {
    fn new(layer: &'a str, component: T) -> Self {
        Self { layer, component }
    }
}

fn setup_ground_platforms_spikes(
    mut commands: Commands,
    assets: Res<Assets<TiledMap>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
) {
    info!("setup_ground_platforms");

    levels
        .iter()
        .filter(|level| level.id == current_level.id)
        .for_each(|level| {
            let tiled_map = match assets.get(&level.handle) {
                Some(tiled_map) => tiled_map,
                None => return,
            };

            trace!("tiled_map: {:#?}", &tiled_map.map);

            let ground = LayerComponentBridge::new("Ground", Ground);
            tiled_object_to_collider(&mut commands, tiled_map, level, ground);

            let platforms = LayerComponentBridge::new("Platforms", Platform);
            tiled_object_to_collider(&mut commands, tiled_map, level, platforms);
        });

    // TODO: Remove this collider
    // Simple ball collider for debugging
    commands
        .spawn((SpatialBundle::default(),))
        .insert(Ccd::enabled())
        .with_children(|parent| {
            // Create 2 x test platforms
            parent
                .spawn(RigidBody::Dynamic)
                .insert(GravityScale(20.0))
                .insert(Collider::ball(20.0))
                .insert(Restitution::coefficient(0.0))
                // .insert(ColliderMassProperties::Density(20.0))
                .insert(ExternalImpulse {
                    // impulse: Vec2::new(100.0, 200.0),
                    // torque_impulse: 14.0,
                    ..default()
                })
                .insert(Platform)
                // .insert(Damping {
                //     linear_damping: 100.0,
                //     angular_damping: 0.0,
                // })
                .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
        });
}

fn despawn_ground_platforms_spikes(
    mut commands: Commands,
    ground_query: Query<(Entity, &Collider), With<Ground>>,
    platforms_query: Query<(Entity, &Collider), With<Platform>>,
) {
    for (entity, _) in ground_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in platforms_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        debug!("Ball altitude: {}", transform.translation.y);
    }
}

/* Apply forces and impulses inside of a system. */
fn apply_forces(
    mut ext_impulses: Query<&mut ExternalImpulse>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: ResMut<State<PlayerState>>,
) {
    // Apply impulses.
    if keyboard_input.pressed(KeyCode::ArrowUp) && state.get() != &PlayerState::Jumping {
        for mut ext_impulse in ext_impulses.iter_mut() {
            ext_impulse.impulse = Vec2::new(0.0, 250.0);
            // ext_impulse.torque_impulse = 0.4;
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        for mut ext_impulse in ext_impulses.iter_mut() {
            ext_impulse.impulse = Vec2::new(20.0, 0.0);
            // ext_impulse.torque_impulse = 0.4;
        }
    }
}
