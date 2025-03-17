use std::f32::consts::PI;

use crate::{
    coregame::state::AppState,
    events::{Restart, ShakeCamera, StartGame},
    player::PlayerSet,
    screen_map::Transition,
};
use bevy::prelude::*;

use crate::{
    coregame::level::{CurrentLevel, Level},
    player::{Player, PlayerState, PLAYER_SPEED},
};
pub struct CameraPlugin;

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct CameraSet;

#[derive(Debug, Resource, Eq, PartialEq, Clone, Copy, Default)]
pub struct Shake(bool);

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), setup_camera);
        app.add_systems(
            OnEnter(AppState::GameCreate),
            move_camera_to_level_start_screen,
        )
        .add_systems(OnEnter(AppState::StartMenu), move_camera_to_center)
        .add_systems(
            Update,
            (camera_follows_player, shake_camera)
                .chain()
                .in_set(CameraSet)
                .after(PlayerSet)
                .run_if(in_state(AppState::GameRunning)),
        );
        app.add_event::<ShakeCamera>();
        app.insert_resource(Shake::default());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
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

#[allow(clippy::too_many_arguments)]
fn camera_follows_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    player_state: Res<State<PlayerState>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
    mut offset: Local<Vec2>,
    shake: Res<Shake>,
) {
    let mut camera = camera_query.single_mut();
    let player = player_query.single();

    levels
        .iter()
        .filter(|level| level.id == current_level.id)
        .for_each(|level| {
            let is_screen_above_exists = level
                .map
                .get_above_screen(camera.translation.xy())
                .is_some();

            let is_screen_below_exists = level
                .map
                .get_below_screen(camera.translation.xy())
                .is_some();

            if *shake == Shake(true) && (!is_screen_above_exists || !is_screen_below_exists) {
                camera.translation.y = level
                    .map
                    .get_screen(camera.translation.xy(), 0.0, 0.0)
                    .unwrap()
                    .get_center()
                    .y;
            }
            let (screen_center, screen_is_fixed, screen_transition) =
                match level.map.get_screen(player.translation.xy(), 0.0, 0.0) {
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
                                offset_tmp.y + PLAYER_SPEED / 2.0 * time.delta_secs(),
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
                                offset_tmp.y - PLAYER_SPEED / 2.0 * time.delta_secs(),
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
                .move_camera(&time, camera.translation.xy(), new_camera_pos)
                .extend(0.0);
        });
}

fn shake_camera(
    mut shake_events: EventReader<ShakeCamera>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    time: Res<Time>,
    mut shake: ResMut<Shake>,
    mut shake_timer: Local<Timer>,
    mut game_event: EventReader<StartGame>,
    mut restart_event: EventReader<Restart>,
) {
    let shake_duration_sec = 6.0;
    let shake_amplitude = 20.0;

    // Slightly zoom the camera to hide level boundaries
    let zoom_factor = match shake_timer.elapsed().as_secs_f32() {
        0.0..2.5 => -0.022 * shake_timer.elapsed().as_secs_f32() + 1.0,
        2.5..3.5 => 0.944,
        3.5..6.0 => 0.022 * shake_timer.elapsed().as_secs_f32() + 1.0 - 0.132,
        _ => 1.0,
    };

    let (mut camera_pos, mut camera_ortho) = camera_query.single_mut();

    if !game_event.is_empty() {
        *shake = Shake(false);
        game_event.clear();
        camera_ortho.scale = 1.0;
        return;
    }

    if !restart_event.is_empty() {
        *shake = Shake(false);
        restart_event.clear();
        camera_ortho.scale = 1.0;
        return;
    }

    for _ev in shake_events.read() {
        debug!("shake event received");
        *shake = Shake(true);
        *shake_timer = Timer::from_seconds(shake_duration_sec, TimerMode::Once);
    }

    if *shake == Shake(true) && !shake_timer.finished() {
        shake_timer.tick(time.delta());

        camera_ortho.scale = zoom_factor;

        camera_pos.translation.y += shake_amplitude
            * (PI * shake_timer.elapsed().as_secs_f32() / shake_duration_sec)
                .sin()
                .powi(2)
            * (5.0 * 2.0 * PI * shake_timer.elapsed().as_secs_f32()).cos();
    }

    if *shake == Shake(true) && shake_timer.finished() {
        *shake = Shake(false);
    }
}
