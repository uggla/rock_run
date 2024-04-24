use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::{Ccd, RigidBody},
    geometry::Collider,
};

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SCALE_FACTOR: f32 = 1.0;
pub const PLAYER_WIDTH: f32 = 100.0;
pub const PLAYER_HEIGHT: f32 = 75.0;
const PLAYER_HITBOX: (Vec2, Vec2, f32) = (Vec2::new(-4.0, -9.0), Vec2::new(-4.0, 8.0), 22.0);
const PLAYER_HITBOX_TRANSLATION: Vec2 = Vec2::new(8.0, 0.0);

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct JumpTimer(Timer);

// Player movement is used to define current movement requested by the player and a state in order
// to know if the player is in a jumping phase or not.

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PlayerState {
    Idling,
    Jumping,
    #[default]
    Falling,
    // Ascend,
    // Descent,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PlayerMovement {
    Idle,
    Jump,
    Crouch,
    Run(PlayerDirection),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("girl.png");
    let layout =
        TextureAtlasLayout::from_grid(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT), 6, 7, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);

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
                translation: Vec3::new(0.0, 200.0, 20.0),
                ..default()
            },
            ..default()
        },
        RigidBody::KinematicPositionBased,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        JumpTimer(Timer::from_seconds(0.250, TimerMode::Once)),
        Player,
        Collider::capsule(PLAYER_HITBOX.0, PLAYER_HITBOX.1, PLAYER_HITBOX.2),
        KinematicCharacterController::default(),
        Ccd::enabled(),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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
        PlayerMovement::Run(direction) => {
            let (mut timer, mut texture, mut sprite) = animation_query.single_mut();
            timer.tick(time.delta());
            if timer.just_finished() {
                match direction {
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
            let (mut timer, mut texture, _) = animation_query.single_mut();
            timer.tick(time.delta());
            if timer.just_finished() {
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
    };

    let mut current_movement: PlayerMovement = PlayerMovement::Idle;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction_x -= 1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Left);
        anim(current_movement);
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction_x += 1.0;
        current_movement = PlayerMovement::Run(PlayerDirection::Right);
        anim(current_movement);
    }
    if keyboard_input.pressed(KeyCode::ArrowUp)
        && !(state.get() == &PlayerState::Jumping || state.get() == &PlayerState::Falling)
    {
        next_state.set(PlayerState::Jumping);
        jump_timer.reset();
        current_movement = PlayerMovement::Jump;
        anim(current_movement);
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
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

    jump_timer.tick(time.delta());
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
