use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{
    assets::RockRunAssets,
    coregame::{camera::CameraSet, state::AppState},
    events::{KeyCollision, Restart},
};

pub const KEY_SCALE_FACTOR: f32 = 2.0;
pub const KEY_WIDTH: f32 = 16.0;
pub const KEY_HEIGHT: f32 = 16.0;

#[derive(Resource, Default)]
pub struct Keys {
    pub numbers: u8,
}

#[derive(Component)]
pub struct Key;

pub struct KeyPlugin;

impl Plugin for KeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartMenu), despawn_key)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_key)
            .add_systems(
                Update,
                (check_get_key, despawn_key_on_restart)
                    .after(CameraSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .insert_resource(Keys::default());
    }
}

fn check_get_key(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut key_collision: EventReader<KeyCollision>,
    mut keys: ResMut<Keys>,
) {
    for ev in key_collision.read() {
        commands.entity(ev.entity).despawn();
        keys.numbers += 1;
        debug!("Collected keys {}", keys.numbers);
        commands.spawn((
            AudioPlayer::new(rock_run_assets.get_something_sound.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(0.8),
                ..default()
            },
        ));
    }
}

fn despawn_key_on_restart(
    mut commands: Commands,
    keys: Query<Entity, With<Key>>,
    restart_event: EventReader<Restart>,
) {
    if restart_event.is_empty() {
        return;
    }

    for key in keys.iter() {
        commands.entity(key).despawn();
    }
}

fn despawn_key(
    mut commands: Commands,
    entities: Query<Entity, With<Key>>,
    mut collected_keys: ResMut<Keys>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
    collected_keys.numbers = 0;
}
