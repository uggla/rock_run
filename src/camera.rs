use bevy::prelude::*;

use crate::{state::AppState, WINDOW_WIDTH};
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(OnEnter(AppState::GameCreate), move_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn move_camera(mut camera_query: Query<&mut Transform, With<Camera2d>>) {
    let mut camera = camera_query.single_mut();
    camera.translation.x = -WINDOW_WIDTH / 2.0;
}
