use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::{
    control::KinematicCharacterControllerOutput, dynamics::Velocity,
    geometry::ActiveCollisionTypes, pipeline::CollisionEvent,
};

use crate::{
    assets::RockRunAssets,
    beasts::{bat::Bat, pterodactyl::Pterodactyl, triceratops::Triceratops},
    coregame::{
        colliders::{ColliderName, Ground, Ladder, Platform, PositionSensor, Spike, Story},
        level::{CurrentLevel, Level},
        state::AppState,
    },
    elements::{
        enigma::{EnigmaKind, Enigmas, RockGate},
        moving_platform::MovingPlatform,
        rock::Rock,
    },
    events::{
        ExtraLifeCollision, Hit, LadderCollisionStart, LadderCollisionStop,
        MovingPlatformCollision, PositionSensorCollisionStart, PositionSensorCollisionStop,
        Restart, StoryMessages, TriceratopsCollision,
    },
    life::ExtraLife,
    player::{self, Player, PlayerState, PLAYER_HEIGHT},
};

pub struct CollisionsPlugin;

struct SensorValues {
    start_pos: Vec2,
    end_pos: Vec2,
    disable_next_collision: bool,
}

#[derive(Debug, Component)]
pub struct StoryQM(String);

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct CollisionSet;

impl Plugin for CollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_collisions,
                triceratops_collisions,
                story_collisions,
                display_story,
                position_sensor_collisions,
                ladder_collisions,
                extra_life_collisions,
            )
                .in_set(CollisionSet)
                .run_if(in_state(AppState::GameRunning)),
        )
        .add_systems(OnEnter(AppState::StartMenu), despawn_qm)
        .add_event::<Hit>()
        .add_event::<TriceratopsCollision>()
        .add_event::<PositionSensorCollisionStart>()
        .add_event::<PositionSensorCollisionStop>()
        .add_event::<LadderCollisionStart>()
        .add_event::<LadderCollisionStop>()
        .add_event::<MovingPlatformCollision>()
        .add_event::<ExtraLifeCollision>();
    }
}

#[allow(clippy::too_many_arguments)]
fn player_collisions(
    player_controller: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    ground: Query<Entity, With<Ground>>,
    platforms: Query<Entity, With<Platform>>,
    spikes: Query<Entity, With<Spike>>,
    moving_platforms: Query<Entity, With<MovingPlatform>>,
    bats: Query<Entity, With<Bat>>,
    pterodactyls: Query<Entity, With<Pterodactyl>>,
    triceratops: Query<Entity, With<Triceratops>>,
    rocks: Query<(Entity, &Velocity), With<Rock>>,
    rockgates: Query<(Entity, &Velocity), With<RockGate>>,
    mut hit: EventWriter<Hit>,
    mut moving_platform_collision: EventWriter<MovingPlatformCollision>,
) {
    if state.get() == &PlayerState::Hit {
        return;
    }

    let ground_entity = match ground.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    let (_player_entity, output) = match player_controller.get_single() {
        Ok(controller) => controller,
        Err(_) => return,
    };

    // info!(
    //     "Entity {:?} moved by {:?} and touches the ground: {:?}",
    //     player_entity, output.effective_translation, output.grounded
    // );
    for character_collision in output.collisions.iter() {
        // Player collides with ground or platforms
        if (character_collision.entity == ground_entity
            || platforms.contains(character_collision.entity))
            && output.grounded
            && state.get() != &PlayerState::Jumping
        {
            next_state.set(PlayerState::Idling);
        }

        // Player collides with spikes
        if spikes.contains(character_collision.entity) {
            hit.send(Hit);
        }

        // Player collides with moving platforms
        for moving_platform in moving_platforms.iter() {
            if character_collision.entity == moving_platform && state.get() != &PlayerState::Jumping
            {
                next_state.set(PlayerState::Idling);
                moving_platform_collision.send(MovingPlatformCollision {
                    entity: moving_platform,
                });
            }
        }
        // Player collides with bats
        for bat in bats.iter() {
            if character_collision.entity == bat {
                debug!("hit bat {:?}", bat);
                hit.send(Hit);
            }
        }

        // Player collides with pterodactyls
        for pterodactyl in pterodactyls.iter() {
            if character_collision.entity == pterodactyl {
                debug!("hit pterodactyl {:?}", pterodactyl);
                hit.send(Hit);
            }
        }

        // Player collides with trieratops
        for triceratops in triceratops.iter() {
            if character_collision.entity == triceratops {
                debug!("hit triceratops {:?}", triceratops);
                hit.send(Hit);
            }
        }

        // Player collides with fast moving rocks or rockgates
        // If rocks are moving slowly, we can stay on it
        for (rock, velocity) in rocks.iter().chain(rockgates.iter()) {
            if character_collision.entity == rock {
                debug!("hit velocity: {:?}", velocity);
                if velocity.linvel.x.abs() > 175.0 || velocity.linvel.y.abs() > 20.0 {
                    hit.send(Hit);
                }

                if output.grounded && state.get() != &PlayerState::Jumping {
                    next_state.set(PlayerState::Idling);
                }
            }
        }
    }
    // Player is falling
    if !output.grounded
        && state.get() == &PlayerState::Idling
        && output.effective_translation.y < -1.0
    {
        next_state.set(PlayerState::Falling);
    }
}

fn triceratops_collisions(
    mut triceratops_controller: Query<
        (Entity, &KinematicCharacterControllerOutput),
        With<Triceratops>,
    >,
    ground: Query<Entity, With<Ground>>,
    mut collision_event: EventWriter<TriceratopsCollision>,
) {
    let ground_entity = match ground.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    for (triceratops_entity, output) in triceratops_controller.iter_mut() {
        for character_collision in output.collisions.iter() {
            // Triceratops collides with ground and can not move on x axis
            if (character_collision.entity == ground_entity)
                && output.grounded
                && (output.effective_translation.x > -0.5 && output.effective_translation.x < 0.5)
            {
                collision_event.send(TriceratopsCollision {
                    id: triceratops_entity,
                });
            }
        }
    }
}

fn story_collisions(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    stories: Query<(Entity, &ColliderName), With<Story>>,
    mut collision_events: EventReader<CollisionEvent>,
    entity_pos: Query<&Transform>,
    qm_entity: Query<(Entity, &StoryQM)>,
    player: Query<Entity, With<Player>>,
) {
    let player_entity = match player.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((entity, collider_name)) =
                    stories.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    let pos = entity_pos.get(entity).unwrap();
                    debug!("Collision: {:?}", pos);
                    commands
                        .spawn(SpriteBundle {
                            texture: rock_run_assets.story_qm.clone(),
                            transform: Transform {
                                translation: pos.translation
                                    + Vec3::new(0.0, PLAYER_HEIGHT / 2.0 + 20.0, 20.0),
                                scale: Vec3::splat(1.5),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(StoryQM(collider_name.0.clone()));
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) =
                    stories.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    for (entity, _) in qm_entity.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn display_story(
    mut commands: Commands,
    qm_entity: Query<(Entity, &StoryQM)>,
    mut msg_event: EventWriter<StoryMessages>,
    enigmas: ResMut<Enigmas>,
    input: Query<
        &leafwing_input_manager::action_state::ActionState<player::PlayerMovement>,
        With<player::Player>,
    >,
) {
    let (entity, story_name) = match qm_entity.get_single() {
        Ok((entity, qm)) => (entity, qm.0.clone()),
        Err(_) => return,
    };

    let input_state = match input.get_single() {
        Ok(state) => state,
        Err(_) => return,
    };

    if input_state.just_pressed(&player::PlayerMovement::Climb) {
        commands.entity(entity).despawn_recursive();
        match story_name.as_str() {
            "story01" => {
                msg_event.send(StoryMessages::Display(vec![
                    ("story01-01".to_string(), None),
                    ("story01-02".to_string(), None),
                    ("story01-03".to_string(), None),
                ]));
            }
            "story02" => {
                msg_event.send(StoryMessages::Display(vec![
                    ("story02-01".to_string(), None),
                    ("story02-02".to_string(), None),
                ]));
            }
            "story03" => {
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == "story03-03")
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Qcm(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                msg_event.send(StoryMessages::Display(vec![
                    ("story03-01".to_string(), None),
                    ("story03-02".to_string(), None),
                    ("story03-03".to_string(), Some(numbers)),
                ]));
            }
            "story04" => {
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == "story04-03")
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Qcm(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                msg_event.send(StoryMessages::Display(vec![
                    ("story04-01".to_string(), None),
                    ("story04-02".to_string(), None),
                    ("story04-03".to_string(), Some(numbers)),
                ]));
            }
            _ => {}
        };
    }
}

fn ladder_collisions(
    ladders: Query<(Entity, &ColliderName), With<Ladder>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut ladder_collision_start: EventWriter<LadderCollisionStart>,
    mut ladder_collision_stop: EventWriter<LadderCollisionStop>,
    player: Query<Entity, With<Player>>,
) {
    let player_entity = match player.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) =
                    ladders.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );
                    ladder_collision_start.send(LadderCollisionStart);
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) =
                    ladders.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    ladder_collision_stop.send(LadderCollisionStop);
                };
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn position_sensor_collisions(
    mut position_sensors: Query<
        (Entity, &ColliderName, &mut ActiveCollisionTypes),
        With<PositionSensor>,
    >,
    mut collision_events: EventReader<CollisionEvent>,
    mut event_start: EventWriter<PositionSensorCollisionStart>,
    mut event_stop: EventWriter<PositionSensorCollisionStop>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut restart_event: EventReader<Restart>,
    player: Query<Entity, With<Player>>,
) {
    if !restart_event.is_empty() {
        for (_position_sensor, _collider_name, mut active_collision_type) in
            position_sensors.iter_mut()
        {
            *active_collision_type = ActiveCollisionTypes::KINEMATIC_STATIC;
        }
        restart_event.clear();
    }

    let player_entity = match player.get_single() {
        Ok(player_entity) => player_entity,
        Err(_) => return,
    };

    for collision_event in collision_events.read() {
        let level = levels
            .iter()
            .find(|level| level.id == current_level.id)
            .unwrap();

        let mut level_sensor_pos: HashMap<u8, HashMap<String, SensorValues>> = HashMap::new();
        level_sensor_pos.insert(
            1,
            HashMap::from([
                (
                    "exit01".to_string(),
                    SensorValues {
                        start_pos: Vec2::ZERO,
                        end_pos: Vec2::ZERO,
                        disable_next_collision: false,
                    },
                ),
                (
                    "pterodactyl_attack01".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(1596.0, 455.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(0.0, 455.0)),
                        disable_next_collision: true,
                    },
                ),
            ]),
        );

        level_sensor_pos.insert(
            2,
            HashMap::from([
                (
                    "bat01".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(3940.0, 850.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(3060.0, 1460.0)),
                        disable_next_collision: false,
                    },
                ),
                (
                    "pterodactyl01".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(1400.0, 320.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(-30.0, 320.0)),
                        disable_next_collision: true,
                    },
                ),
                (
                    "rock01".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                        disable_next_collision: false,
                    },
                ),
                (
                    "rock02".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                        disable_next_collision: false,
                    },
                ),
                (
                    "rock03".to_string(),
                    SensorValues {
                        start_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        end_pos: level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                        disable_next_collision: false,
                    },
                ),
                (
                    "exit01".to_string(),
                    SensorValues {
                        start_pos: Vec2::ZERO,
                        end_pos: Vec2::ZERO,
                        disable_next_collision: false,
                    },
                ),
            ]),
        );

        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name, mut active_collision_type)) = position_sensors
                    .iter_mut()
                    .find(|(entity, _collider_name, _active_collision_type)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    if let Some(collider) = level_sensor_pos.get(&current_level.id) {
                        if let Some(sensor_values) = collider.get(&collider_name.0) {
                            if sensor_values.disable_next_collision {
                                *active_collision_type = ActiveCollisionTypes::STATIC_STATIC;
                            }
                            event_start.send(PositionSensorCollisionStart {
                                sensor_name: collider_name.0.clone(),
                                spawn_pos: sensor_values.start_pos,
                                exit_pos: sensor_values.end_pos,
                            });
                        }
                    }
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name, _active_collision_type)) = position_sensors
                    .iter()
                    .find(|(entity, _collider_name, _active_collision_type)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    event_stop.send(PositionSensorCollisionStop {
                        sensor_name: collider_name.0.clone(),
                    });
                };
            }
        }
    }
}

fn extra_life_collisions(
    extralifes: Query<(Entity, &ColliderName), With<ExtraLife>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut extralife_collision: EventWriter<ExtraLifeCollision>,
    player: Query<Entity, With<Player>>,
) {
    let player_entity = match player.get_single() {
        Ok(player_entity) => player_entity,
        Err(_) => return,
    };
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((entity, collider_name)) =
                    extralifes.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    extralife_collision.send(ExtraLifeCollision { entity });
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) =
                    extralifes.iter().find(|(entity, _collider_name)| {
                        (entity == e1 && player_entity == *e2)
                            || (entity == e2 && player_entity == *e1)
                    })
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );
                }
            }
        }
    }
}

// Remove QM entities if player goes to menu and question mark is displayed
fn despawn_qm(mut commands: Commands, qm_entity: Query<(Entity, &StoryQM)>) {
    for (entity, _) in qm_entity.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
