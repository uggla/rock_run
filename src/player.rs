use bevy::{
    audio::PlaybackMode,
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use bevy_rapier2d::{
    control::{KinematicCharacterController, KinematicCharacterControllerOutput},
    dynamics::RigidBody,
    geometry::Collider,
    prelude::Ccd,
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
    elements::moving_platform::{MovingPlatform, PlatformDelta},
    helpers::texture::{IndexDirection, cycle_texture, swing_texture},
    messages::{Hit, LadderCollisionStart, LadderCollisionStop, LifeEvent, Restart, StartGame},
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

#[derive(Default)]
struct PlatformCarryState {
    entity: Option<Entity>,
    contact_frames_left: u8,
    horizontal_offset: f32,
    vertical_offset: f32,
}

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
                    .chain()
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

    spawn_player(
        &mut commands,
        &rock_run_assets,
        &mut texture_atlases,
        level,
        &start_position,
    );
}

fn spawn_player(
    commands: &mut Commands,
    rock_run_assets: &Res<RockRunAssets>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    level: &Level,
    start_position: &StartPos,
) {
    info!("spawn_player");

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

    let start_position = player_start_translation(level, start_position);

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
            // normal_nudge_factor: 0.03,
            ..default()
        },
        Ccd::enabled(),
        Player,
        PlayerAudio {
            jump_sound: rock_run_assets.jump_sound.clone(),
            hit_sound: rock_run_assets.hit_sound.clone(),
        },
        input_map,
    ));
}

fn player_start_translation(level: &Level, start_position: &StartPos) -> Vec3 {
    match start_position.0 {
        Some(position) => {
            info!("Tiled start_position: {:?}", position);
            level.map.tiled_to_bevy_coord(position).extend(20.0)
        }
        None => level.map.get_start_screen().get_center().extend(20.0) + PLAYER_START_OFFSET,
    }
}

#[allow(clippy::type_complexity)]
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
            Option<&KinematicCharacterControllerOutput>,
            &mut JumpTimer,
            &PlayerAudio,
        ),
        (With<Player>, Without<MovingPlatform>),
    >,
    moving_platforms: Query<(&Transform, &PlatformDelta), (With<MovingPlatform>, Without<Player>)>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut index_direction: Local<IndexDirection>,
    mut ladder_collision_start: MessageReader<LadderCollisionStart>,
    mut ladder_collision_stop: MessageReader<LadderCollisionStop>,
    restart_event: MessageReader<Restart>,
    mut game_event: MessageReader<StartGame>,
    mut ladder_collision: Local<bool>,
    mut toggle: Local<bool>,
    mut platform_carry: Local<PlatformCarryState>,
) -> Result<()> {
    let (
        mut player_collider,
        mut player_pos,
        mut player_controller,
        player_controller_output,
        mut jump_timer,
        player_audio,
    ) = player_query.single_mut()?;
    let mut direction_x = 0.0;
    let mut direction_y = 0.0;
    let riding_platform_preview = platform_carry.contact_frames_left > 0;
    let grounded_preview =
        player_controller_output.is_some_and(|output| output.grounded) || riding_platform_preview;
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
                        PlayerState::Falling if !grounded_preview => {
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
                                if riding_platform_preview {
                                    texture.index = 6;
                                } else {
                                    cycle_texture(texture, 6..=10);
                                }
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
                        PlayerState::Falling if !grounded_preview => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                cycle_texture(texture, 14..=16);
                            }
                        }
                        _ => {
                            if let Some(texture) = &mut sprite.texture_atlas {
                                if riding_platform_preview {
                                    texture.index = 0;
                                } else {
                                    swing_texture(texture, 0..=4, &mut index_direction);
                                }
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
                if anim_timer.just_finished()
                    && let Some(texture) = &mut sprite.texture_atlas
                {
                    cycle_texture(texture, 33..=36);
                }
            }
            PlayerMovement::Crouch => {
                let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
                anim_timer.tick(time.delta());
                if anim_timer.just_finished()
                    && let Some(texture) = &mut sprite.texture_atlas
                {
                    cycle_texture(texture, 33..=36);
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

    if !restart_event.is_empty() {
        *ladder_collision = false;
        *toggle = false;
        *platform_carry = PlatformCarryState::default();
    }

    if !game_event.is_empty() {
        game_event.clear();
        *ladder_collision = false;
        *toggle = false;
        *platform_carry = PlatformCarryState::default();
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
        // Moving platform collision can miss a frame around direction changes.
        // Keep contact and carry for a short grace period to avoid dropping off.
        let current_platform = player_controller_output.and_then(|output| {
            output.collisions.iter().find_map(|collision| {
                moving_platforms
                    .get(collision.entity)
                    .ok()
                    .map(|_| collision.entity)
            })
        });

        if let Some(entity) = current_platform {
            if platform_carry.entity != Some(entity)
                && let Ok((platform_transform, _)) = moving_platforms.get(entity)
            {
                platform_carry.horizontal_offset =
                    player_pos.translation.x - platform_transform.translation.x;
                platform_carry.vertical_offset =
                    player_pos.translation.y - platform_transform.translation.y;
            }
            platform_carry.entity = Some(entity);
            platform_carry.contact_frames_left = 3;
        } else if platform_carry.contact_frames_left > 0
            && player_controller_output.is_some_and(|output| output.grounded)
        {
            platform_carry.contact_frames_left -= 1;
        } else {
            platform_carry.entity = None;
            platform_carry.contact_frames_left = 0;
        }

        let riding_platform = platform_carry.contact_frames_left > 0;
        let platform_state = platform_carry
            .entity
            .and_then(|entity| moving_platforms.get(entity).ok())
            .filter(|_| riding_platform);
        let platform_movement = platform_state
            .map(|(_, delta)| **delta)
            .unwrap_or(Vec2::ZERO);
        let on_moving_platform = platform_movement != Vec2::ZERO;

        if riding_platform {
            platform_carry.horizontal_offset += direction_x * PLAYER_SPEED * time.delta_secs();
        }

        if let Some((platform_transform, _)) = platform_state {
            // Keep the player horizontally anchored to the current platform while
            // preserving the relative offset measured when contact was acquired.
            player_pos.translation.x =
                platform_transform.translation.x + platform_carry.horizontal_offset;
            if platform_movement.y > 0.0 {
                player_pos.translation.y =
                    platform_transform.translation.y + platform_carry.vertical_offset;
            } else {
                player_pos.translation.y += platform_movement.y;
            }
        }

        let vertical_translation = if riding_platform
            || on_moving_platform
            || !player_controller_output.is_some_and(|output| output.grounded)
            || player_controller_output.is_some_and(|output| output.is_sliding_down_slope)
        {
            -PLAYER_SPEED * time.delta_secs()
        } else {
            0.0
        };

        if (player_controller_output.is_some_and(|output| output.grounded) || riding_platform)
            && state.get() == &PlayerState::Falling
        {
            next_state.set(PlayerState::Idling);
        }

        // Normal movement, if the player is on a moving platform following line will not move the
        // player but is required to detect collisions
        player_controller.translation = Some(Vec2::new(
            if on_moving_platform {
                0.0
            } else {
                direction_x * PLAYER_SPEED * time.delta_secs()
            },
            vertical_translation,
        ));
    }
    Ok(())
}

fn check_out_of_screen(
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut restart: MessageWriter<Restart>,
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
    mut hit_event: MessageReader<Hit>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut jump_timer: Query<&mut JumpTimer>,
    mut just_hit: Local<bool>,
    mut restart: MessageWriter<Restart>,
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

    if state.get() == &PlayerState::Hit && jump_timer.is_finished() && *just_hit {
        debug!("timer finished");
        *just_hit = false;
        restart.write(Restart);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn restart_level(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut restart: MessageReader<Restart>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    start_position: Res<StartPos>,
    player_query: Query<Entity, With<Player>>,
    mut life_event: MessageWriter<LifeEvent>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut ladder_collision_stop: MessageWriter<LadderCollisionStop>,
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

    let player = player_query.single()?;

    life_event.write(LifeEvent::Lost);
    ladder_collision_stop.write(LadderCollisionStop);
    commands.entity(player).despawn();
    spawn_player(
        &mut commands,
        &rock_run_assets,
        &mut texture_atlases,
        level,
        &start_position,
    );
    next_state.set(PlayerState::Falling);
    Ok(())
}

fn despawn_player(mut commands: Commands, player: Query<Entity, With<Player>>) {
    if let Ok(player) = player.single() {
        commands.entity(player).despawn();
    }
}
