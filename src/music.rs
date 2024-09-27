use crate::{
    assets::RockRunAssets,
    coregame::{level::CurrentLevel, state::AppState},
};
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

#[derive(Component)]
struct Music;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_music)
            .add_systems(OnEnter(AppState::NextLevel), setup_music)
            .add_systems(OnEnter(AppState::StartMenu), despawn_music)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_music)
            .add_systems(OnEnter(AppState::GameOver), despawn_music);
    }
}

fn setup_music(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    // levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_music");

    // let level = levels
    //     .iter()
    //     .find(|level| level.id == current_level.id)
    //     .unwrap();

    match current_level.id {
        1 => {
            commands.spawn((
                AudioBundle {
                    source: rock_run_assets.music_level01.clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        volume: Volume::new(0.3),
                        ..default()
                    },
                },
                Music,
            ));
        }
        2 => {
            commands.spawn((
                AudioBundle {
                    source: rock_run_assets.music_level02.clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        volume: Volume::new(0.3),
                        ..default()
                    },
                },
                Music,
            ));
        }
        _ => {}
    }
}

fn despawn_music(mut commands: Commands, musics: Query<Entity, With<Music>>) {
    for music in musics.iter() {
        commands.entity(music).despawn_recursive();
    }
}
