use bevy::prelude::*;
use bevy_rapier2d::{
    control::{KinematicCharacterController, KinematicCharacterControllerOutput},
    pipeline::QueryFilterFlags,
};

use crate::{
    colliders::{Ground, Platform, Spike},
    events::{Hit, TriceratopsCollision},
    player::{Player, PlayerState},
    triceratops::Triceratops,
};

pub struct CollisionPlugin;

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct CollisionSet;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_collision, triceratops_collision).in_set(CollisionSet),
        )
        .add_event::<Hit>()
        .add_event::<TriceratopsCollision>();
    }
}

fn player_collision(
    player_controller: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    ground: Query<Entity, With<Ground>>,
    platforms: Query<Entity, With<Platform>>,
    spikes: Query<Entity, With<Spike>>,
    mut hit: EventWriter<Hit>,
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

    let (output, mut ctrl) = match triceratops_controller.get_single_mut() {
        Ok(controller) => controller,
        Err(_) => return,
    };

    // Maybe an event saying that the game start could be better than using a state here.
    // The following code is used right after the end of the restart event
    // It enable again the collision with the triceratops. The collision are disabled as soon as we
    // detect a collision with the player to keep only one collision and avoid multiple collisions.
    if state.get() == &PlayerState::Falling {
        ctrl.filter_flags
            .remove(QueryFilterFlags::EXCLUDE_KINEMATIC);
        debug!("Re-enabling collision with triceratops");
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
            && (output.effective_translation.x > -1.0 && output.effective_translation.x < 1.0)
        {
            collision_event.send(TriceratopsCollision);
        }
    }
}
