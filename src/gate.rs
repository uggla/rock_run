use bevy::prelude::*;

pub struct EnigmaPlugin;

impl Plugin for EnigmaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_bat, spawn_bat, despawn_bat_on_restart)
                .after(CollisionSet)
                .run_if(in_state(AppState::GameRunning)),
        );
    }
}
