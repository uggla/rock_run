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

#[derive(Component, Default)]
struct PlayerPlatformCarry {
    entity: Option<Entity>,
    contact_frames_left: u8,
    horizontal_offset: f32,
    vertical_offset: f32,
}

#[derive(Component, Default)]
struct PlayerLadderState {
    touching_ladder: bool,
    climb_nudge_right: bool,
}

type MovingPlatformQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static PlatformDelta),
    (With<MovingPlatform>, Without<Player>),
>;

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

#[derive(Debug, Clone, Copy)]
struct PlayerIntent {
    movement: PlayerMovement,
    direction_x: f32,
    direction_y: f32,
    jump_requested: bool,
}

impl Default for PlayerIntent {
    fn default() -> Self {
        Self {
            movement: PlayerMovement::Idle,
            direction_x: 0.0,
            direction_y: 0.0,
            jump_requested: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PlayerMotion {
    controller_translation: Vec2,
    grounded_preview: bool,
    riding_platform: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PlayerVisualState {
    Idle,
    Run,
    Jump,
    Fall,
    Climb,
    ClimbIdle,
    Hit,
}

struct PlayerMotionContext<'a, 'w, 's> {
    controller_output: Option<&'a KinematicCharacterControllerOutput>,
    moving_platforms: &'a MovingPlatformQuery<'w, 's>,
    time: &'a Time,
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
        PlayerLadderState::default(),
        PlayerPlatformCarry::default(),
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
            &mut PlayerLadderState,
            &mut PlayerPlatformCarry,
            &PlayerAudio,
        ),
        (With<Player>, Without<MovingPlatform>),
    >,
    moving_platforms: MovingPlatformQuery,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut index_direction: Local<IndexDirection>,
    mut ladder_collision_start: MessageReader<LadderCollisionStart>,
    mut ladder_collision_stop: MessageReader<LadderCollisionStop>,
    restart_event: MessageReader<Restart>,
    mut game_event: MessageReader<StartGame>,
) -> Result<()> {
    let (
        mut player_collider,
        mut player_pos,
        mut player_controller,
        player_controller_output,
        mut jump_timer,
        mut ladder_state,
        mut platform_carry,
        player_audio,
    ) = player_query.single_mut()?;

    jump_timer.tick(time.delta());
    let input_state = input.single()?;

    // Handle hit state — early return
    if *state.get() == PlayerState::Hit {
        let _ = animate_player_visual(
            PlayerVisualState::Hit,
            false,
            &time,
            &mut animation_query,
            &mut index_direction,
        );
        player_controller.translation = Some(Vec2::new(0.0, PLAYER_SPEED * time.delta_secs()));
        return Ok(());
    }

    if !restart_event.is_empty() {
        reset_player_runtime_state(&mut ladder_state, &mut platform_carry);
    }

    if !game_event.is_empty() {
        game_event.clear();
        reset_player_runtime_state(&mut ladder_state, &mut platform_carry);
        next_state.set(PlayerState::Falling);
    }

    update_ladder_state(
        &mut ladder_state,
        &mut ladder_collision_start,
        &mut ladder_collision_stop,
        &mut next_state,
    );

    let intent = read_player_intent(input_state, ladder_state.touching_ladder);

    handle_jump_request(
        intent,
        state.get(),
        &mut next_state,
        &mut jump_timer,
        player_audio,
        &mut commands,
    );

    if matches!(
        intent.movement,
        PlayerMovement::Climb | PlayerMovement::Crouch
    ) && ladder_state.touching_ladder
    {
        next_state.set(PlayerState::Climbing);
    }

    // Update facing direction and collider
    update_player_direction(
        intent.direction_x,
        &mut animation_query,
        &mut player_collider,
    )?;

    let motion = compute_player_motion(
        state.get(),
        intent,
        &mut player_pos,
        &mut jump_timer,
        &mut ladder_state,
        &mut platform_carry,
        PlayerMotionContext {
            controller_output: player_controller_output,
            moving_platforms: &moving_platforms,
            time: &time,
        },
    );

    sync_player_state(
        state.get(),
        motion,
        jump_timer.just_finished(),
        &mut next_state,
    );

    let visual_state = compute_player_visual_state(state.get(), intent, motion);
    let _ = animate_player_visual(
        visual_state,
        motion.riding_platform,
        &time,
        &mut animation_query,
        &mut index_direction,
    );
    apply_player_motion(&mut player_controller, motion);
    Ok(())
}

fn reset_player_runtime_state(
    ladder_state: &mut PlayerLadderState,
    platform_carry: &mut PlayerPlatformCarry,
) {
    *ladder_state = PlayerLadderState::default();
    *platform_carry = PlayerPlatformCarry::default();
}

fn update_ladder_state(
    ladder_state: &mut PlayerLadderState,
    ladder_collision_start: &mut MessageReader<LadderCollisionStart>,
    ladder_collision_stop: &mut MessageReader<LadderCollisionStop>,
    next_state: &mut ResMut<NextState<PlayerState>>,
) {
    if !ladder_collision_start.is_empty() {
        ladder_state.touching_ladder = true;
        ladder_collision_start.clear();
    }

    if !ladder_collision_stop.is_empty() {
        ladder_state.touching_ladder = false;
        ladder_collision_stop.clear();
        next_state.set(PlayerState::Falling);
    }
}

fn handle_jump_request(
    intent: PlayerIntent,
    state: &PlayerState,
    next_state: &mut ResMut<NextState<PlayerState>>,
    jump_timer: &mut JumpTimer,
    player_audio: &PlayerAudio,
    commands: &mut Commands,
) {
    if !intent.jump_requested || matches!(state, PlayerState::Jumping | PlayerState::Falling) {
        return;
    }

    next_state.set(PlayerState::Jumping);
    jump_timer.reset();
    commands.spawn((
        AudioPlayer::new(player_audio.jump_sound.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));
}

fn read_player_intent(
    input_state: &ActionState<PlayerMovement>,
    touching_ladder: bool,
) -> PlayerIntent {
    let mut intent = PlayerIntent::default();

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Left)) {
        intent.direction_x = -1.0;
        intent.movement = PlayerMovement::Run(PlayerDirection::Left);
    }

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Right)) {
        intent.direction_x = 1.0;
        intent.movement = PlayerMovement::Run(PlayerDirection::Right);
    }

    if input_state.just_pressed(&PlayerMovement::Jump) {
        intent.jump_requested = true;
        intent.movement = PlayerMovement::Jump;
    }

    if touching_ladder && input_state.pressed(&PlayerMovement::Climb) {
        intent.direction_y = 1.0;
        intent.movement = PlayerMovement::Climb;
    }

    if touching_ladder && input_state.pressed(&PlayerMovement::Crouch) {
        intent.direction_y = -1.0;
        intent.movement = PlayerMovement::Crouch;
    }

    intent
}

fn update_player_direction(
    direction_x: f32,
    animation_query: &mut Query<(&mut AnimationTimer, &mut Sprite)>,
    collider: &mut Collider,
) -> Result<()> {
    if direction_x < 0.0 {
        let (_, mut sprite) = animation_query.single_mut()?;
        sprite.flip_x = true;
        *collider = Collider::capsule(
            PLAYER_HITBOX.0 + PLAYER_HITBOX_TRANSLATION,
            PLAYER_HITBOX.1 + PLAYER_HITBOX_TRANSLATION,
            PLAYER_HITBOX.2,
        );
    } else if direction_x > 0.0 {
        let (_, mut sprite) = animation_query.single_mut()?;
        sprite.flip_x = false;
        *collider = Collider::capsule(PLAYER_HITBOX.0, PLAYER_HITBOX.1, PLAYER_HITBOX.2);
    }
    Ok(())
}

fn compute_player_motion(
    state: &PlayerState,
    intent: PlayerIntent,
    player_pos: &mut Transform,
    jump_timer: &mut JumpTimer,
    ladder_state: &mut PlayerLadderState,
    platform_carry: &mut PlayerPlatformCarry,
    context: PlayerMotionContext,
) -> PlayerMotion {
    let PlayerMotionContext {
        controller_output,
        moving_platforms,
        time,
    } = context;

    if state == &PlayerState::Jumping {
        if jump_timer.just_finished() {
            return PlayerMotion {
                controller_translation: Vec2::ZERO,
                grounded_preview: false,
                riding_platform: false,
            };
        }

        return PlayerMotion {
            controller_translation: Vec2::new(
                intent.direction_x * PLAYER_SPEED * time.delta_secs(),
                PLAYER_SPEED * time.delta_secs(),
            ),
            grounded_preview: false,
            riding_platform: false,
        };
    }

    if ladder_state.touching_ladder && state == &PlayerState::Climbing {
        let controller_translation = if intent.direction_x == 0.0 && intent.direction_y == 0.0 {
            let direction = if ladder_state.climb_nudge_right {
                0.1
            } else {
                -0.1
            };
            ladder_state.climb_nudge_right = !ladder_state.climb_nudge_right;
            Vec2::new(direction * time.delta_secs(), 0.0)
        } else {
            Vec2::new(
                intent.direction_x * PLAYER_SPEED * time.delta_secs(),
                intent.direction_y * PLAYER_SPEED * time.delta_secs(),
            )
        };

        return PlayerMotion {
            controller_translation,
            grounded_preview: false,
            riding_platform: false,
        };
    }

    let (riding_platform, platform_movement) = update_platform_carry(
        platform_carry,
        player_pos,
        controller_output,
        moving_platforms,
        intent.direction_x,
        time,
    );
    let on_moving_platform = platform_movement != Vec2::ZERO;
    let grounded = controller_output.is_some_and(|output| output.grounded);
    let grounded_preview = grounded || riding_platform;

    let vertical_translation = if riding_platform
        || on_moving_platform
        || !grounded
        || controller_output.is_some_and(|output| output.is_sliding_down_slope)
    {
        -PLAYER_SPEED * time.delta_secs()
    } else {
        0.0
    };

    let horizontal_translation = if on_moving_platform {
        0.0
    } else {
        intent.direction_x * PLAYER_SPEED * time.delta_secs()
    };

    PlayerMotion {
        controller_translation: Vec2::new(horizontal_translation, vertical_translation),
        grounded_preview,
        riding_platform,
    }
}

fn compute_player_visual_state(
    state: &PlayerState,
    intent: PlayerIntent,
    motion: PlayerMotion,
) -> PlayerVisualState {
    match state {
        PlayerState::Hit => PlayerVisualState::Hit,
        PlayerState::Jumping => PlayerVisualState::Jump,
        PlayerState::Falling if !motion.grounded_preview => PlayerVisualState::Fall,
        PlayerState::Climbing => {
            if intent.direction_y == 0.0 {
                PlayerVisualState::ClimbIdle
            } else {
                PlayerVisualState::Climb
            }
        }
        _ => match intent.movement {
            PlayerMovement::Run(_) => PlayerVisualState::Run,
            PlayerMovement::Jump => PlayerVisualState::Jump,
            PlayerMovement::Climb | PlayerMovement::Crouch => PlayerVisualState::Climb,
            _ => PlayerVisualState::Idle,
        },
    }
}

fn apply_player_motion(player_controller: &mut KinematicCharacterController, motion: PlayerMotion) {
    player_controller.translation = Some(motion.controller_translation);
}

fn sync_player_state(
    state: &PlayerState,
    motion: PlayerMotion,
    jump_finished: bool,
    next_state: &mut ResMut<NextState<PlayerState>>,
) {
    if state == &PlayerState::Jumping && jump_finished {
        next_state.set(PlayerState::Falling);
    }

    if motion.grounded_preview && state == &PlayerState::Falling {
        next_state.set(PlayerState::Idling);
    }
}

fn animate_player_visual(
    visual_state: PlayerVisualState,
    riding_platform: bool,
    time: &Time,
    animation_query: &mut Query<(&mut AnimationTimer, &mut Sprite)>,
    index_direction: &mut Local<IndexDirection>,
) -> Result<()> {
    match visual_state {
        PlayerVisualState::Run => {
            let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
            anim_timer.tick(time.delta());
            if anim_timer.just_finished()
                && let Some(texture) = &mut sprite.texture_atlas
            {
                if riding_platform {
                    texture.index = 6;
                } else {
                    cycle_texture(texture, 6..=10);
                }
            }
        }
        PlayerVisualState::Idle => {
            let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
            anim_timer.tick(time.delta());
            if anim_timer.just_finished()
                && let Some(texture) = &mut sprite.texture_atlas
            {
                if riding_platform {
                    texture.index = 0;
                } else {
                    swing_texture(texture, 0..=4, index_direction);
                }
            }
        }
        PlayerVisualState::Jump => {
            let (_, mut sprite) = animation_query.single_mut()?;
            if let Some(texture) = &mut sprite.texture_atlas {
                texture.index = 11;
            }
        }
        PlayerVisualState::Fall => {
            let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
            anim_timer.tick(time.delta());
            if anim_timer.just_finished()
                && let Some(texture) = &mut sprite.texture_atlas
            {
                cycle_texture(texture, 14..=16);
            }
        }
        PlayerVisualState::Climb => {
            let (mut anim_timer, mut sprite) = animation_query.single_mut()?;
            anim_timer.tick(time.delta());
            if anim_timer.just_finished()
                && let Some(texture) = &mut sprite.texture_atlas
            {
                cycle_texture(texture, 33..=36);
            }
        }
        PlayerVisualState::ClimbIdle => {
            let (_, mut sprite) = animation_query.single_mut()?;
            if let Some(texture) = &mut sprite.texture_atlas {
                texture.index = 34;
            }
        }
        PlayerVisualState::Hit => {
            let (_, mut sprite) = animation_query.single_mut()?;
            if let Some(texture) = &mut sprite.texture_atlas {
                texture.index = 26;
            }
        }
    }
    Ok(())
}

/// Updates the player's moving-platform attachment state and applies the
/// platform carry directly to the player's transform.
///
/// The function does three things:
/// - detect which moving platform, if any, is currently supporting the player
/// - preserve a short contact grace period to avoid dropping the player during
///   direction changes or brief collision misses
/// - keep the player's relative offset on the platform while still returning
///   the platform delta used by the controller motion logic
///
/// Horizontal carry is applied by re-anchoring the player's `x` position to
/// the platform plus the stored offset. Vertical carry is asymmetric on
/// purpose: when the platform moves upward we snap to the saved offset to avoid
/// visible gaps under the feet, and when it moves downward we apply the delta
/// incrementally to reduce the risk of pushing the player into the platform.
#[allow(clippy::type_complexity)]
fn update_platform_carry(
    platform_carry: &mut PlayerPlatformCarry,
    player_pos: &mut Transform,
    player_controller_output: Option<&KinematicCharacterControllerOutput>,
    moving_platforms: &MovingPlatformQuery,
    direction_x: f32,
    time: &Time,
) -> (bool, Vec2) {
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

    (riding_platform, platform_movement)
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
