use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterController, dynamics::RigidBody, geometry::Collider,
};
use leafwing_input_manager::{
    action_state::ActionState, axislike::SingleAxis, input_map::InputMap,
    plugin::InputManagerPlugin, Actionlike, InputManagerBundle,
};

use crate::{
    collision::CollisionSet,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{Hit, LifeEvent, Restart},
};

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SCALE_FACTOR: f32 = 1.0;
pub const PLAYER_WIDTH: f32 = 100.0;
pub const PLAYER_HEIGHT: f32 = 75.0;
const PLAYER_HITBOX: (Vec2, Vec2, f32) = (Vec2::new(-4.0, -9.0), Vec2::new(-4.0, 8.0), 22.0);
const PLAYER_HITBOX_TRANSLATION: Vec2 = Vec2::new(8.0, 0.0);
const PLAYER_START_OFFSET: Vec3 = Vec3::new(-480.0, 0.0, 0.0);

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct JumpTimer(Timer);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PlayerState {
    Idling,
    Jumping,
    #[default]
    Falling,
    Hit,
    // Ascend,
    // Descent,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Actionlike, Hash, Reflect)]
pub enum PlayerMovement {
    Idle,
    Jump,
    Crouch,
    Run(PlayerDirection),
    Hit,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Actionlike, Hash, Reflect)]
pub enum PlayerDirection {
    Left,
    Right,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum IndexDirection {
    #[default]
    Up,
    Down,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerMovement>::default())
            .init_state::<PlayerState>()
            .add_systems(OnEnter(AppState::GameCreate), setup_player)
            .add_systems(OnEnter(AppState::StartMenu), despawn_player)
            .add_systems(
                Update,
                (move_player, check_out_of_screen, check_hit, restart_level)
                    .after(CollisionSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
) {
    info!("setup_player");

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = asset_server.load("girl.png");
    let layout =
        TextureAtlasLayout::from_grid(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT), 6, 7, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);

    let mut input_map = InputMap::new([
        (PlayerMovement::Jump, KeyCode::ArrowUp),
        (
            PlayerMovement::Run(PlayerDirection::Left),
            KeyCode::ArrowLeft,
        ),
        (
            PlayerMovement::Run(PlayerDirection::Right),
            KeyCode::ArrowRight,
        ),
        (PlayerMovement::Crouch, KeyCode::ArrowDown),
    ]);

    input_map.insert(PlayerMovement::Jump, GamepadButtonType::South);
    input_map.insert(
        PlayerMovement::Run(PlayerDirection::Right),
        SingleAxis::positive_only(GamepadAxisType::LeftStickX, 0.4),
    );
    input_map.insert(
        PlayerMovement::Run(PlayerDirection::Left),
        SingleAxis::negative_only(GamepadAxisType::LeftStickX, -0.4),
    );

    commands.spawn((
        SpriteSheetBundle {
            texture,
            sprite: Sprite { ..default() },
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
            transform: Transform {
                scale: Vec3::splat(PLAYER_SCALE_FACTOR),
                translation: level.map.get_start_screen().get_center().extend(20.0)
                    + PLAYER_START_OFFSET,
                ..default()
            },
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
            // offset: CharacterLength::Absolute(1.0),
            normal_nudge_factor: 1.0,
            ..default()
        },
        // Ccd::enabled(),
        InputManagerBundle::with_map(input_map),
        Player,
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn move_player(
    time: Res<Time>,
    input: Query<&ActionState<PlayerMovement>, With<Player>>,
    mut player_query: Query<(&mut Collider, &mut KinematicCharacterController), With<Player>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut jump_timer: Query<&mut JumpTimer>,
    mut direction: Local<IndexDirection>,
) {
    let (mut player_collider, mut player_controller) = player_query.single_mut();
    let mut jump_timer = jump_timer.single_mut();
    let mut direction_x = 0.0;
    let mut anim = |current_movement: PlayerMovement| match current_movement {
        PlayerMovement::Run(player_direction) => {
            let (mut anim_timer, mut texture, mut sprite) = animation_query.single_mut();
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
                        cycle_texture(&mut texture, 14..=16);
                    }
                    _ => {
                        cycle_texture(&mut texture, 6..=10);
                    }
                }
            }
        }
        PlayerMovement::Idle => {
            let (mut anim_timer, mut texture, _) = animation_query.single_mut();
            anim_timer.tick(time.delta());
            if anim_timer.just_finished() {
                match state.get() {
                    PlayerState::Jumping => {}
                    PlayerState::Falling => {
                        cycle_texture(&mut texture, 14..=16);
                    }
                    _ => {
                        swing_texture(&mut texture, 0..=4, &mut direction);
                    }
                }
            }
        }
        PlayerMovement::Jump => {
            let (_, mut texture, _) = animation_query.single_mut();
            texture.index = 11;
        }

        PlayerMovement::Crouch => {}

        PlayerMovement::Hit => {
            let (_, mut texture, _) = animation_query.single_mut();
            texture.index = 26;
        }
    };

    jump_timer.tick(time.delta());
    let input_state = input.single();

    let mut current_movement: PlayerMovement = PlayerMovement::Idle;

    if *state.get() == PlayerState::Hit {
        current_movement = PlayerMovement::Hit;
        anim(current_movement);
        player_controller.translation = Some(Vec2::new(0.0, PLAYER_SPEED * time.delta_seconds()));
        return;
    }

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Left)) {
        direction_x = -1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Left);
        anim(current_movement);
    }

    if input_state.pressed(&PlayerMovement::Run(PlayerDirection::Right)) {
        direction_x = 1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Right);
        anim(current_movement);
    }

    if input_state.just_pressed(&PlayerMovement::Jump)
        && !(state.get() == &PlayerState::Jumping || state.get() == &PlayerState::Falling)
    {
        next_state.set(PlayerState::Jumping);
        jump_timer.reset();
        current_movement = PlayerMovement::Jump;
        anim(current_movement);
    }

    if input_state.just_pressed(&PlayerMovement::Crouch) {
        current_movement = PlayerMovement::Crouch;
        anim(current_movement);
    }

    if current_movement == PlayerMovement::Idle {
        anim(PlayerMovement::Idle);
    }

    if state.get() == &PlayerState::Jumping {
        if jump_timer.just_finished() {
            next_state.set(PlayerState::Falling);
            player_controller.translation = Some(Vec2::new(
                direction_x * PLAYER_SPEED * time.delta_seconds(),
                -PLAYER_SPEED * time.delta_seconds(),
            ));
        } else {
            player_controller.translation = Some(Vec2::new(
                direction_x * PLAYER_SPEED * time.delta_seconds(),
                PLAYER_SPEED * time.delta_seconds(),
            ));
        }
    } else {
        player_controller.translation = Some(Vec2::new(
            direction_x * PLAYER_SPEED * time.delta_seconds(),
            -PLAYER_SPEED * time.delta_seconds(),
        ));
    }
}

fn cycle_texture(texture: &mut TextureAtlas, texture_index_range: RangeInclusive<usize>) {
    if !texture_index_range.contains(&texture.index) {
        texture.index = *texture_index_range.start();
    }
    texture.index = if texture.index == *texture_index_range.end() {
        *texture_index_range.start()
    } else {
        texture.index + 1
    };
}

fn swing_texture(
    texture: &mut TextureAtlas,
    texture_index_range: RangeInclusive<usize>,
    direction: &mut Local<IndexDirection>,
) {
    if !texture_index_range.contains(&texture.index) {
        texture.index = *texture_index_range.start();
    }

    if texture.index == *texture_index_range.end() && **direction == IndexDirection::Up {
        **direction = IndexDirection::Down;
    }

    if texture.index == *texture_index_range.start() && **direction == IndexDirection::Down {
        **direction = IndexDirection::Up;
    }

    trace!("tdirection: {:?}", direction);
    trace!("tindex: {}", texture.index);
    texture.index = if **direction == IndexDirection::Up {
        texture.index + 1
    } else {
        texture.index - 1
    };
}

fn check_out_of_screen(
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut restart: EventWriter<Restart>,
) {
    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let player = player_query.single_mut();

    if level
        .map
        .get_screen((player.translation.x, player.translation.y + PLAYER_HEIGHT).into())
        .is_none()
    {
        restart.send(Restart);
    }
}

fn check_hit(
    mut hit_event: EventReader<Hit>,
    state: Res<State<PlayerState>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut jump_timer: Query<&mut JumpTimer>,
    mut just_hit: Local<bool>,
    mut restart: EventWriter<Restart>,
) {
    if !hit_event.is_empty() {
        next_state.set(PlayerState::Hit);
        hit_event.clear();
    }

    if state.get() == &PlayerState::Hit {
        let mut jump_timer = jump_timer.single_mut();
        if !*just_hit {
            jump_timer.reset();
            *just_hit = true;
        }

        if jump_timer.just_finished() {
            *just_hit = false;
            restart.send(Restart);
        }
    }
}

fn restart_level(
    mut restart: EventReader<Restart>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut life_event: EventWriter<LifeEvent>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    if restart.is_empty() {
        return;
    }

    restart.clear();
    info!("restart level");
    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let mut player = player_query.single_mut();

    next_state.set(PlayerState::Falling);

    life_event.send(LifeEvent::Lost);
    player.translation =
        level.map.get_start_screen().get_center().extend(20.00) + PLAYER_START_OFFSET;
}

fn despawn_player(mut commands: Commands, player: Query<Entity, With<Player>>) {
    if let Ok(player) = player.get_single() {
        commands.entity(player).despawn_recursive();
    }
}
