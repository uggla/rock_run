use bevy::{audio::PlaybackMode, prelude::*};
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
};
use leafwing_input_manager::{
    Actionlike, action_state::ActionState, axislike::AxisDirection, input_map::InputMap,
    plugin::InputManagerPlugin, prelude::GamepadControlDirection,
};

use crate::{
    assets::RockRunAssets,
    collisions::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        menu::StartPos,
        state::AppState,
    },
    events::{
        Hit, LadderCollisionStart, LadderCollisionStop, LifeEvent, MovingPlatformDescending,
        Restart, StartGame,
    },
    helpers::texture::{IndexDirection, cycle_texture, swing_texture},
};

pub const PLAYER_SPEED: f32 = 500.0;
const PLAYER_SCALE_FACTOR: f32 = 1.0;
pub const PLAYER_WIDTH: f32 = 100.0;
pub const PLAYER_HEIGHT: f32 = 75.0;
const PLAYER_HITBOX: (Vec2, Vec2, f32) = (Vec2::new(-4.0, -9.0), Vec2::new(-4.0, 8.0), 22.0);
const PLAYER_HITBOX_TRANSLATION: Vec2 = Vec2::new(8.0, 0.0);
const PLAYER_START_OFFSET: Vec3 = Vec3::new(-480.0, 0.0, 0.0);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlayerAudio {
    jump_sound: Handle<AudioSource>,
    hit_sound: Handle<AudioSource>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct JumpTimer(Timer);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PlayerState {
    Idling,
    Jumping,
    #[default]
    Falling,
    Hit,
    Climbing,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Actionlike, Hash, Reflect)]
pub enum PlayerMovement {
    Idle,
    Jump,
    Climb,
    Crouch,
    Run(PlayerDirection),
    Hit,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Actionlike, Hash, Reflect)]
pub enum PlayerDirection {
    Left,
    Right,
}

pub struct PlayerPlugin;

#[derive(SystemSet, Clone, Hash, Debug, PartialEq, Eq)]
pub struct PlayerSet;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerMovement>::default())
            .init_state::<PlayerState>()
            .add_systems(OnEnter(AppState::GameCreate), setup_player)
            .add_systems(OnEnter(AppState::NextLevel), setup_player)
            .add_systems(OnEnter(AppState::StartMenu), despawn_player)
            .add_systems(OnEnter(AppState::FinishLevel), despawn_player)
            .add_systems(
                Update,
                (move_player, check_out_of_screen, check_hit, restart_level)
                    .in_set(PlayerSet)
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn setup_player(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    start_position: Res<StartPos>,
) {
    info!("setup_player");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = rock_run_assets.player.clone();

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(PLAYER_WIDTH as u32, PLAYER_HEIGHT as u32),
        6,
        7,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlases.add(layout);

    let mut input_map = InputMap::new([
        (PlayerMovement::Jump, KeyCode::Space),
        (
            PlayerMovement::Run(PlayerDirection::Left),
            KeyCode::ArrowLeft,
        ),
        (
            PlayerMovement::Run(PlayerDirection::Right),
            KeyCode::ArrowRight,
        ),
        (PlayerMovement::Climb, KeyCode::ArrowUp),
        (PlayerMovement::Crouch, KeyCode::ArrowDown),
    ]);

    input_map.insert(PlayerMovement::Jump, GamepadButton::South);
    input_map.insert(
        PlayerMovement::Run(PlayerDirection::Right),
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickX,

            direction: AxisDirection::Positive,
            threshold: 0.8,
        },
    );
    input_map.insert(
        PlayerMovement::Run(PlayerDirection::Left),
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickX,

            direction: AxisDirection::Negative,
            threshold: 0.8,
        },
    );
    input_map.insert(
        PlayerMovement::Climb,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickY,

            direction: AxisDirection::Positive,
            threshold: 0.8,
        },
    );
    input_map.insert(
        PlayerMovement::Crouch,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickY,

            direction: AxisDirection::Negative,
            threshold: 0.8,
        },
    );

    let start_position: Vec3 = match start_position.0 {
        Some(position) => {
            info!("Tiled start_position: {:?}", position);
            level.map.tiled_to_bevy_coord(position).extend(20.0)
        }
        None => level.map.get_start_screen().get_center().extend(20.0) + PLAYER_START_OFFSET,
    };

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform {
            scale: Vec3::splat(PLAYER_SCALE_FACTOR),
            translation: start_position,
            ..default()
        },
        RigidBody::KinematicPositionBased,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        JumpTimer(Timer::from_seconds(0.250, TimerMode::Once)),
        Collider::capsule(PLAYER_HITBOX.0, PLAYER_HITBOX.1, PLAYER_HITBOX.2),
        KinematicCharacterController {
            max_slope_climb_angle: 30.0f32.to_radians(),
            // Automatically slide down on slopes smaller than 30 degrees.
            min_slope_slide_angle: 30.0f32.to_radians(),
            normal_nudge_factor: 1.0,
            ..default()
        },
        // Ccd::enabled(),
        Player,
        PlayerAudio {
            jump_sound: rock_run_assets.jump_sound.clone(),
            hit_sound: rock_run_assets.hit_sound.clone(),
        },
        input_map,
    ));
}

#[allow(clippy::too_many_arguments)]
fn move_player(
    mut commands: Commands,
    time: Res<Time>,
    input: Query<&ActionState<PlayerMovement>, With<Player>>,
    mut player_query: Query<
        (
            &mut Collider,
            &mut Transform,
            &mut KinematicCharacterController,
            &PlayerAudio,
        ),
        With<Player>,
    >,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut jump_timer: Query<&mut JumpTimer>,
    mut index_direction: Local<IndexDirection>,
    mut ladder_collision_start: EventReader<LadderCollisionStart>,
    mut ladder_collision_stop: EventReader<LadderCollisionStop>,
    mut moving_platform_descending: EventReader<MovingPlatformDescending>,
    mut game_event: EventReader<StartGame>,
    mut ladder_collision: Local<bool>,
    mut toggle: Local<bool>,
) -> Result<()> {
    let (mut player_collider, mut player_pos, mut player_controller, player_audio) =
        player_query.single_mut()?;
    let mut jump_timer = jump_timer.single_mut()?;
    let mut direction_x = 0.0;
    let mut direction_y = 0.0;
    let mut anim = |current_movement: PlayerMovement| -> Result<()> {
        match current_movement {
            PlayerMovement::Run(player_direction) => {
                let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
                anim_timer.tick(time.delta());
                match player_direction {
                    PlayerDirection::Left => {
                        sprite.flip_x = true;
                        *player_collider = Collider::capsule(
                            PLAYER_HITBOX.0 + PLAYER_HITBOX_TRANSLATION,
                            PLAYER_HITBOX.1 + PLAYER_HITBOX_TRANSLATION,
                            PLAYER_HITBOX.2,
                        );
                    }
                    PlayerDirection::Right => {
                        sprite.flip_x = false;
                        *player_collider =
                            Collider::capsule(PLAYER_HITBOX.0, PLAYER_HITBOX.1, PLAYER_HITBOX.2);
                    }
                }
                if anim_timer.just_finished() {
                    match state.get() {
                        PlayerState::Jumping => {}
                        PlayerState::Falling => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                cycle_texture(texture, 14..=16);
                            }
                        }
                        PlayerState::Climbing => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                cycle_texture(texture, 33..=36);
                            }
                        }
                        _ => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                cycle_texture(texture, 6..=10);
                            }
                        }
                    }
                }
            }
            PlayerMovement::Idle => {
                let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    match state.get() {
                        PlayerState::Jumping => {}
                        PlayerState::Climbing => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                texture.index = 34;
                            }
                        }
                        PlayerState::Falling => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                cycle_texture(texture, 14..=16);
                            }
                        }
                        _ => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                swing_texture(texture, 0..=4, &mut index_direction);
                            }
                        }
                    }
                }
            }
            PlayerMovement::Jump => {
                let (_, mut sprite) = animation_query.single_mut()?;
                if let Some(texture) = &mut sprite.texture_atlas {
                    texture.index = 11;
                }
            }

            PlayerMovement::Climb => {
                let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    if let Some(texture) = &mut sprite.texture_atlas {
                        cycle_texture(texture, 33..=36);
                    }
                }
            }
            PlayerMovement::Crouch => {
                let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    if let Some(texture) = &mut sprite.texture_atlas {
                        cycle_texture(texture, 33..=36);
                    }
                }
            }

            PlayerMovement::Hit => {
                let (_, mut sprite) = animation_query.single_mut()?;
                if let Some(texture) = &mut sprite.texture_atlas {
                    texture.index = 26;
                }
            }
        }
        Ok(())
    };

    jump_timer.tick(time.delta());
    let input_state = input.single()?;
    let mut current_movement: PlayerMovement = PlayerMovement::Idle;

    if *state.get() == PlayerState::Hit {
        current_movement = PlayerMovement::Hit;
        let _ = anim(current_movement);
        player_controller.translation = Some(Vec2::new(0.0, PLAYER_SPEED * time.delta_secs()));
        return Ok(());
    }

    if !game_event.is_empty() {
        game_event.clear();
        *ladder_collision = false;
        next_state.set(PlayerState::Falling);
    }

    if !ladder_collision_start.is_empty() {
        *ladder_collision = true;
        ladder_collision_start.clear();
    }

    if !ladder_collision_stop.is_empty() {
        *ladder_collision = false;
        ladder_collision_stop.clear();
        next_state.set(PlayerState::Falling);
    }

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Left)) {
        direction_x = -1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Left);
        let _ = anim(current_movement);
    }

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Right)) {
        direction_x = 1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Right);
        let _ = anim(current_movement);
    }

    if input_state.just_pressed(&PlayerMovement::Jump)
        && !(state.get() == &PlayerState::Jumping || state.get() == &PlayerState::Falling)
    {
        next_state.set(PlayerState::Jumping);
        jump_timer.reset();
        commands.spawn((
            AudioPlayer::new(player_audio.jump_sound.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        ));
        current_movement = PlayerMovement::Jump;
        let _ = anim(current_movement);
    }

    if input_state.pressed(&PlayerMovement::Climb) && *ladder_collision {
        next_state.set(PlayerState::Climbing);
        direction_y = 1.0;
        current_movement = PlayerMovement::Climb;
        let _ = anim(current_movement);
    }

    if input_state.pressed(&PlayerMovement::Crouch) && *ladder_collision {
        next_state.set(PlayerState::Climbing);
        direction_y = -1.0;
        current_movement = PlayerMovement::Crouch;
        let _ = anim(current_movement);
    }

    if current_movement == PlayerMovement::Idle {
        let _ = anim(PlayerMovement::Idle);
    }

    if state.get() == &PlayerState::Jumping {
        if jump_timer.just_finished() {
            next_state.set(PlayerState::Falling);
        } else {
            player_controller.translation = Some(Vec2::new(
                direction_x * PLAYER_SPEED * time.delta_secs(),
                PLAYER_SPEED * time.delta_secs(),
            ));
        }
    } else if *ladder_collision && state.get() == &PlayerState::Climbing {
        // If the player is stationary, beasts are blocked by the player's
        // hitbox and collision is not detected. Therefore, initiate a slight,
        // imperceptible movement to trigger the collision.
        if direction_x == 0.0 && direction_y == 0.0 {
            if *toggle {
                player_controller.translation = Some(Vec2::new(0.1 * time.delta_secs(), 0.0));
                *toggle = false;
            } else {
                player_controller.translation = Some(Vec2::new(-0.1 * time.delta_secs(), 0.0));
                *toggle = true;
            }
        } else {
            player_controller.translation = Some(Vec2::new(
                direction_x * PLAYER_SPEED * time.delta_secs(),
                direction_y * PLAYER_SPEED * time.delta_secs(),
            ));
        }
    } else {
        // Check if we are on a moving platform that goes down
        let events: Vec<&MovingPlatformDescending> = moving_platform_descending.read().collect();

        if let Some(event) = events.first() {
            // Move the player alongside the moving platform
            player_pos.translation += Vec3::new(event.movement.x, event.movement.y, 0.0);
            // Add the player movement
            player_pos.translation +=
                Vec3::new(direction_x * PLAYER_SPEED * time.delta_secs(), 0.0, 0.0);
        }
        // Normal movement, if the player is on a moving platform following line will not move the
        // player but is required to detect collisions
        player_controller.translation = Some(Vec2::new(
            direction_x * PLAYER_SPEED * time.delta_secs(),
            -PLAYER_SPEED * time.delta_secs(),
        ));
    }
    Ok(())
}

fn check_out_of_screen(
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut restart: EventWriter<Restart>,
) -> Result<()> {
    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let player = player_query.single_mut()?;

    if level
        .map
        .get_screen(
            (player.translation.x, player.translation.y).into(),
            0.0,
            2.0 * PLAYER_HEIGHT,
        )
        .is_none()
    {
        restart.write(Restart);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_hit(
    mut commands: Commands,
    mut hit_event: EventReader<Hit>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut jump_timer: Query<&mut JumpTimer>,
    mut just_hit: Local<bool>,
    mut restart: EventWriter<Restart>,
    mut player_query: Query<&PlayerAudio, With<Player>>,
) -> Result<()> {
    let mut jump_timer = jump_timer.single_mut()?;
    if !hit_event.is_empty() && state.get() != &PlayerState::Hit {
        debug!("hit event received");
        hit_event.clear();
        next_state.set(PlayerState::Hit);
        debug!("justhit {}", *just_hit);
        if !*just_hit {
            let player_audio = player_query.single_mut()?;
            jump_timer.reset();
            *just_hit = true;
            commands.spawn((
                AudioPlayer::new(player_audio.hit_sound.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            ));
            debug!("justhit reset timer");
        }
    }

    if state.get() == &PlayerState::Hit && jump_timer.finished() && *just_hit {
        debug!("timer finished");
        *just_hit = false;
        restart.write(Restart);
    }
    Ok(())
}

fn restart_level(
    mut restart: EventReader<Restart>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut life_event: EventWriter<LifeEvent>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut ladder_collision_stop: EventWriter<LadderCollisionStop>,
) -> Result<()> {
    if restart.is_empty() {
        return Ok(());
    }

    restart.clear();
    info!("restart level");
    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let mut player = player_query.single_mut()?;

    life_event.write(LifeEvent::Lost);
    ladder_collision_stop.write(LadderCollisionStop);
    player.translation =
        level.map.get_start_screen().get_center().extend(20.00) + PLAYER_START_OFFSET;
    next_state.set(PlayerState::Falling);
    Ok(())
}

fn despawn_player(mut commands: Commands, player: Query<Entity, With<Player>>) {
    if let Ok(player) = player.single() {
        commands.entity(player).despawn();
    }
}
