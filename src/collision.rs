use bevy::prelude::*;
use bevy_rapier2d::control::KinematicCharacterControllerOutput;

use crate::{
    player::{Player, PlayerState},
    Ground, Platform,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_ground);
    }
}
fn player_ground(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    ground: Query<Entity, With<Ground>>,
    platforms: Query<Entity, With<Platform>>,
) {
    let ground_entity = ground.single();

    if let Ok((player_entity, output)) = controllers.get_single() {
        // info!(
        //     "Entity {:?} moved by {:?} and touches the ground: {:?}",
        //     player_entity, output.effective_translation, output.grounded
        // );
        // if output.collisions == ground_entity {
        for character_collision in output.collisions.iter() {
            // Player collides with ground or platforms
            if character_collision.entity == ground_entity
                || platforms.contains(character_collision.entity)
            {
                if output.grounded && state.get() != &PlayerState::Jumping {
                    next_state.set(PlayerState::Idling);
                }

                if !output.grounded && state.get() == &PlayerState::Idling {
                    next_state.set(PlayerState::Falling);
                }
            }
        }
    }
}
