use bevy::prelude::*;

use crate::{
    level::{CurrentLevel, Level},
    state::AppState,
};
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(
            OnEnter(AppState::GameCreate),
            move_camera_to_level_start_screen,
        )
        .add_systems(OnEnter(AppState::StartMenu), move_camera_to_center);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn move_camera_to_center(mut camera_query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera = camera_query.single_mut();
    camera.translation = Vec3::new(0.0, 0.0, 0.0);
}

fn move_camera_to_level_start_screen(
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut camera = camera_query.single_mut();

    for level in levels.iter() {
        if level.id == current_level.id {
            camera.translation = level.map.get_start_screen().get_center().extend(0.0);
            break;
        }
    }
}
