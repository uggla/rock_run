use bevy::prelude::*;
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};
use tiled::ObjectShape;

use crate::{
    coregame::level::{CurrentLevel, Level},
    coregame::state::AppState,
    helpers::tiled::TiledMap,
};

pub struct CollidersPlugin;

impl Plugin for CollidersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_colliders)
            .add_systems(OnEnter(AppState::NextLevel), setup_colliders)
            .add_systems(OnEnter(AppState::StartMenu), despawn_colliders)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_colliders);
    }
}

#[derive(Component, Clone, Debug)]
pub struct Ground;

#[derive(Component, Clone, Debug)]
pub struct Platform;

#[derive(Component, Clone, Debug)]
pub struct Spike;

#[derive(Component, Clone, Debug)]
pub struct Story;

#[derive(Component, Clone, Debug)]
pub struct PositionSensor;

#[derive(Component, Clone, Debug)]
pub struct Ladder;

#[derive(Component, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ColliderName(pub String);

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

                    if bridge.sensor {
                        commands
                            .spawn((
                                Collider::cuboid(*width / 2.0, *height / 2.0),
                                bridge.component.clone(),
                                TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                            ))
                            .insert(Sensor)
                            .insert(ActiveEvents::COLLISION_EVENTS)
                            .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
                            .insert(ColliderName(object.name.clone()));
                    } else {
                        commands
                            .spawn((
                                Collider::cuboid(*width / 2.0, *height / 2.0),
                                bridge.component.clone(),
                                TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                            ))
                            .insert(ColliderName(object.name.clone()));
                    }
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
                            if bridge.sensor {
                                commands
                                    .spawn((collider, bridge.component.clone()))
                                    .insert(Sensor)
                                    .insert(ActiveEvents::COLLISION_EVENTS)
                                    .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
                                    .insert(ColliderName(object.name.clone()));
                            } else {
                                commands
                                    .spawn((collider, bridge.component.clone()))
                                    .insert(ColliderName(object.name.clone()));
                            }
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

                    if bridge.sensor {
                        commands
                            .spawn((Collider::polyline(points, None), bridge.component.clone()))
                            .insert(Sensor)
                            .insert(ActiveEvents::COLLISION_EVENTS)
                            .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
                            .insert(ColliderName(object.name.clone()));
                    } else {
                        commands
                            .spawn((Collider::polyline(points, None), bridge.component.clone()))
                            .insert(ColliderName(object.name.clone()));
                    }
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
                    if bridge.sensor {
                        commands
                            .spawn((
                                Collider::ball(*width / 2.0),
                                bridge.component.clone(),
                                // Platform,
                                TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                            ))
                            .insert(Sensor)
                            .insert(ActiveEvents::COLLISION_EVENTS)
                            .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
                            .insert(ColliderName(object.name.clone()));
                    } else {
                        commands
                            .spawn((
                                Collider::ball(*width / 2.0),
                                bridge.component.clone(),
                                // Platform,
                                TransformBundle::from(Transform::from_xyz(x, y, 0.0)),
                            ))
                            .insert(ColliderName(object.name.clone()));
                    }
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
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
                        .insert(ColliderName(object.name.clone()));
                }
            };
        })
    })
}

struct LayerComponentBridge<'a, T: Component + Clone> {
    layer: &'a str,
    component: T,
    sensor: bool,
}

impl<'a, T: Component + Clone> LayerComponentBridge<'a, T> {
    fn new(layer: &'a str, component: T, sensor: bool) -> Self {
        Self {
            layer,
            component,
            sensor,
        }
    }
}

fn setup_colliders(
    mut commands: Commands,
    assets: Res<Assets<TiledMap>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
) {
    info!("setup_ground_platforms_spikes");

    levels
        .iter()
        .filter(|level| level.id == current_level.id)
        .for_each(|level| {
            let tiled_map = match assets.get(&level.handle) {
                Some(tiled_map) => tiled_map,
                None => return,
            };

            trace!("tiled_map: {:#?}", &tiled_map.map);

            // Ground must contain only 1 object.
            let ground = LayerComponentBridge::new("Ground", Ground, false);
            tiled_object_to_collider(&mut commands, tiled_map, level, ground);

            let platforms = LayerComponentBridge::new("Platforms", Platform, false);
            tiled_object_to_collider(&mut commands, tiled_map, level, platforms);

            let spikes = LayerComponentBridge::new("Spikes", Spike, false);
            tiled_object_to_collider(&mut commands, tiled_map, level, spikes);

            let stories = LayerComponentBridge::new("Stories", Story, true);
            tiled_object_to_collider(&mut commands, tiled_map, level, stories);

            let position_sensors =
                LayerComponentBridge::new("PositionSensors", PositionSensor, true);
            tiled_object_to_collider(&mut commands, tiled_map, level, position_sensors);

            let ladders = LayerComponentBridge::new("Ladders", Ladder, true);
            tiled_object_to_collider(&mut commands, tiled_map, level, ladders);
        });
}

fn despawn_colliders(
    mut commands: Commands,
    ground_query: Query<(Entity, &Collider), With<Ground>>,
    platforms_query: Query<(Entity, &Collider), With<Platform>>,
    spikes_query: Query<(Entity, &Collider), With<Spike>>,
    stories_query: Query<(Entity, &Collider), With<Story>>,
    position_sensors_query: Query<(Entity, &Collider), With<PositionSensor>>,
    ladders_query: Query<(Entity, &Collider), With<Ladder>>,
) {
    for (entity, _) in ground_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in platforms_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in spikes_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in stories_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in position_sensors_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, _) in ladders_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
