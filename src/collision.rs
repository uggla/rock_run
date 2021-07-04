use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::{
    control::{KinematicCharacterController, KinematicCharacterControllerOutput},
    dynamics::Velocity,
    pipeline::{CollisionEvent, QueryFilterFlags},
};

use crate::{
    bat::Bat,
    colliders::{ColliderName, Ground, Ladder, Platform, PositionSensor, Spike, Story},
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    enigma::{EnigmaKind, Enigmas},
    events::{
        ExtraLifeCollision, Hit, LadderCollisionStart, LadderCollisionStop,
        MovingPlatformCollision, PositionSensorCollisionStart, PositionSensorCollisionStop,
        StoryMessages, TriceratopsCollision,
    },
    life::ExtraLife,
    moving_platform::MovingPlatform,
    player::{self, Player, PlayerState, PLAYER_HEIGHT},
    rock::Rock,
    triceratops::Triceratops,
};

pub struct CollisionPlugin;

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct CollisionSet;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_collision,
                triceratops_collision,
                bat_collision,
                story_collision,
                display_story,
                position_sensor_collision,
                ladder_collision,
                extra_life_collision,
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
fn player_collision(
    player_controller: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    ground: Query<Entity, With<Ground>>,
    platforms: Query<Entity, With<Platform>>,
    spikes: Query<Entity, With<Spike>>,
    moving_platforms: Query<Entity, With<MovingPlatform>>,
    rocks: Query<(Entity, &Velocity), With<Rock>>,
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

        // Player collides with fast moving rocks
        // If rocks are moving slowly, we can stay on it
        for (rock, velocity) in rocks.iter() {
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

fn triceratops_collision(
    state: Res<State<PlayerState>>,
    mut triceratops_controller: Query<
        (
            Entity,
            &KinematicCharacterControllerOutput,
            &mut KinematicCharacterController,
        ),
        With<Triceratops>,
    >,
    ground: Query<Entity, With<Ground>>,
    player: Query<Entity, With<Player>>,
    mut collision_event: EventWriter<TriceratopsCollision>,
    mut hit: EventWriter<Hit>,
) {
    if state.get() == &PlayerState::Hit {
        return;
    }

    let ground_entity = match ground.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    let player_entity = match player.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    for (triceratops_entity, output, mut ctrl) in triceratops_controller.iter_mut() {
        // Maybe an event saying that the game start could be better than using a state here.
        // The following code is used right after the end of the restart event
        // It enable again the collision with the triceratops. The collision are disabled as soon as we
        // detect a collision with the player to keep only one collision and avoid multiple collisions.
        if state.get() == &PlayerState::Falling {
            ctrl.filter_flags
                .remove(QueryFilterFlags::EXCLUDE_KINEMATIC);
            // debug!("Re-enabling collision with triceratops");
        }
        for character_collision in output.collisions.iter() {
            // triceratops hits player
            if character_collision.entity == player_entity {
                hit.send(Hit);
                ctrl.filter_flags = QueryFilterFlags::EXCLUDE_KINEMATIC;
                debug!("Triceratops hits player, disabling further collision with triceratops");
            }
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

#[derive(Debug, Component)]
pub struct StoryQM(String);

fn story_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stories: Query<(Entity, &ColliderName), With<Story>>,
    mut collision_events: EventReader<CollisionEvent>,
    entity_pos: Query<&Transform>,
    qm_entity: Query<(Entity, &StoryQM)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((entity, collider_name)) = stories
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
                {
                    debug!("Received collision event: {:?}", collision_event);

                    let pos = entity_pos.get(entity).unwrap();
                    debug!("Collision: {:?}", pos);
                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("qm.png"),
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
                if stories.contains(*e1) || stories.contains(*e2) {
                    debug!("Received collision event: {:?}", collision_event);
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
                    .filter(|e| e.associated_story == "story03-01")
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Qcm(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                msg_event.send(StoryMessages::Display(vec![(
                    "story03-01".to_string(),
                    Some(numbers),
                )]));
            }
            _ => {}
        };
    }
}

fn ladder_collision(
    ladders: Query<(Entity, &ColliderName), With<Ladder>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut ladder_collision_start: EventWriter<LadderCollisionStart>,
    mut ladder_collision_stop: EventWriter<LadderCollisionStop>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) = ladders
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );
                    ladder_collision_start.send(LadderCollisionStart);
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                if ladders.contains(*e1) || ladders.contains(*e2) {
                    debug!("Received collision event: {:?}", collision_event);
                    ladder_collision_stop.send(LadderCollisionStop);
                }
            }
        }
    }
}

fn bat_collision(
    state: Res<State<PlayerState>>,
    mut bat_controller: Query<
        (
            &KinematicCharacterControllerOutput,
            &mut KinematicCharacterController,
        ),
        With<Bat>,
    >,
    player: Query<Entity, With<Player>>,
    mut hit: EventWriter<Hit>,
) {
    if state.get() == &PlayerState::Hit {
        return;
    }

    let player_entity = match player.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    for (output, mut ctrl) in bat_controller.iter_mut() {
        if state.get() == &PlayerState::Falling {
            ctrl.filter_flags
                .remove(QueryFilterFlags::EXCLUDE_KINEMATIC);
            // debug!("Re-enabling collision with bat");
        }

        for character_collision in output.collisions.iter() {
            // bat hits player
            if character_collision.entity == player_entity {
                hit.send(Hit);
                ctrl.filter_flags = QueryFilterFlags::EXCLUDE_KINEMATIC;
                debug!("Bat hits player, disabling further collision with bat");
            }
        }
    }
}

fn position_sensor_collision(
    position_sensors: Query<(Entity, &ColliderName), With<PositionSensor>>,
    collision_events: EventReader<CollisionEvent>,
    mut event_start: EventWriter<PositionSensorCollisionStart>,
    mut event_stop: EventWriter<PositionSensorCollisionStop>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    let mut collision_events = collision_events;
    for collision_event in collision_events.read() {
        let level = levels
            .iter()
            .find(|level| level.id == current_level.id)
            .unwrap();

        let mut level_sensor_pos: HashMap<u8, HashMap<String, [Vec2; 2]>> = HashMap::new();
        level_sensor_pos.insert(
            1,
            HashMap::from([("exit01".to_string(), [Vec2::ZERO, Vec2::ZERO])]),
        );

        level_sensor_pos.insert(
            2,
            HashMap::from([
                (
                    "bat01".to_string(),
                    [
                        level.map.tiled_to_bevy_coord(Vec2::new(3940.0, 850.0)),
                        level.map.tiled_to_bevy_coord(Vec2::new(3060.0, 1460.0)),
                    ],
                ),
                (
                    "rock01".to_string(),
                    [
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                    ],
                ),
                (
                    "rock02".to_string(),
                    [
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                    ],
                ),
                (
                    "rock03".to_string(),
                    [
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 800.0)),
                        level.map.tiled_to_bevy_coord(Vec2::new(5300.0, 600.0)),
                    ],
                ),
                ("exit01".to_string(), [Vec2::ZERO, Vec2::ZERO]),
            ]),
        );

        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) = position_sensors
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
                {
                    debug!(
                        "Received collision event: {:?}, collider name: {:?}",
                        collision_event, collider_name
                    );

                    if let Some(collider) = level_sensor_pos.get(&current_level.id) {
                        if let Some(pos) = collider.get(&collider_name.0) {
                            event_start.send(PositionSensorCollisionStart {
                                sensor_name: collider_name.0.clone(),
                                spawn_pos: pos[0],
                                exit_pos: pos[1],
                            });
                        }
                    }
                };
            }
            CollisionEvent::Stopped(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((_entity, collider_name)) = position_sensors
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
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

fn extra_life_collision(
    extralifes: Query<(Entity, &ColliderName), With<ExtraLife>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut extralife_collision: EventWriter<ExtraLifeCollision>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _cf) => {
                // Warning, e1 and e2 can be swapped.
                if let Some((entity, collider_name)) = extralifes
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
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
                if let Some((_entity, collider_name)) = extralifes
                    .iter()
                    .find(|(entity, _collider_name)| entity == e1 || entity == e2)
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
