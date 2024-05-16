use crate::screen_map::Transition;
use bevy::prelude::*;

use crate::{
    level::{CurrentLevel, Level},
    player::{Player, PlayerState, PLAYER_SPEED},
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
        .add_systems(OnEnter(AppState::StartMenu), move_camera_to_center)
        .add_systems(
            Update,
            camera_follows_player.run_if(in_state(AppState::GameRunning)),
        );
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

fn camera_follows_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    player_state: Res<State<PlayerState>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
    mut offset: Local<Vec2>,
) {
    let mut camera = camera_query.single_mut();
    let player = player_query.single();

    levels
        .iter()
        .filter(|level| level.id == current_level.id)
        .for_each(|level| {
            let (screen_center, screen_is_fixed, screen_transition) =
                match level.map.get_screen(player.translation.xy()) {
                    Some(screen) => (
                        screen.get_center(),
                        screen.is_fixed_screen(),
                        screen.get_transition(),
                    ),
                    None => (player.translation.xy(), false, Transition::Smooth),
                };

            let (above_screen_is_fixed, above_screen_transition) =
                match level.map.get_above_screen(player.translation.xy()) {
                    Some(above_screen) => (
                        above_screen.is_fixed_screen(),
                        above_screen.get_transition(),
                    ),
                    None => (true, Transition::Smooth),
                };

            let dist = screen_center - player.translation.xy();

            let new_camera_pos = match (
                screen_is_fixed,
                screen_transition,
                above_screen_is_fixed,
                above_screen_transition,
            ) {
                (true, Transition::Hard, _, _) => {
                    // Hard camera transition going down
                    Vec2::new(player.translation.x, player.translation.y + dist.y)
                }
                (false, _, true, Transition::Hard) => {
                    // Hard camera transition going up
                    Vec2::new(player.translation.x, player.translation.y + dist.y)
                }
                (true, Transition::Smooth, _, _) => {
                    // Smooth camera transition going down
                    if dist.y > 0.0
                        && !(*player_state == PlayerState::Falling
                            || *player_state == PlayerState::Jumping)
                    {
                        if camera.translation.y < screen_center.y {
                            let offset_tmp = *offset;
                            *offset = Vec2::new(
                                offset_tmp.x,
                                offset_tmp.y + PLAYER_SPEED / 2.0 * time.delta_seconds(),
                            );
                        } else {
                            *offset = dist;
                        }
                    }

                    trace!("player_state: {:?}", player_state);
                    trace!("offset: {:?}", offset);
                    Vec2::new(player.translation.x, player.translation.y + offset.y)
                }
                (false, _, true, Transition::Smooth) => {
                    // Smooth camera transition going up
                    if dist.y < 0.0
                        && !(*player_state == PlayerState::Falling
                            || *player_state == PlayerState::Jumping)
                    {
                        let offset_tmp = *offset;
                        if offset.y > 0.0 {
                            *offset = Vec2::new(
                                offset_tmp.x,
                                offset_tmp.y - PLAYER_SPEED / 2.0 * time.delta_seconds(),
                            );
                        } else {
                            *offset = Vec2::new(offset_tmp.x, 0.0);
                        }
                    }

                    trace!("player_state: {:?}", player_state);
                    trace!("offset: {:?}", offset);
                    Vec2::new(player.translation.x, player.translation.y + offset.y)
                }
                _ => {
                    // The camera follows the player
                    Vec2::new(player.translation.x, player.translation.y)
                }
            };

            camera.translation = level
                .map
                .move_camera(camera.translation.xy(), new_camera_pos)
                .extend(0.0);
        })
}
