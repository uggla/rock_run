use bevy::prelude::*;
use bevy_rapier2d::control::KinematicCharacterControllerOutput;

use crate::{
    ground_platforms::{Ground, Platform},
    player::{Player, PlayerState},
};

pub struct CollisionPlugin;

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct CollisionState;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_collision.in_set(CollisionState));
    }
}
fn player_collision(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    ground: Query<Entity, With<Ground>>,
    platforms: Query<Entity, With<Platform>>,
) {
    if let Ok(ground_entity) = ground.get_single() {
        if let Ok((_player_entity, output)) = controllers.get_single() {
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
            }
            // Player is falling
            if !output.grounded && state.get() == &PlayerState::Idling {
                next_state.set(PlayerState::Falling);
            }
        }
    }
}
