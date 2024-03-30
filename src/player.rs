use bevy::prelude::*;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SCALE_FACTOR: f32 = 3.0;
pub const PLAYER_WIDTH: f32 = 24.0;
pub const PLAYER_HEIGHT: f32 = 24.0;
pub const PLAYER_HITBOX_WIDTH: f32 = 14.0;
pub const PLAYER_HITBOX_HEIGHT: f32 = 24.0;

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]

pub struct Player;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct JumpTimer(Timer);

// Player movement is used to define current movement requested by the player and a state in order
// to know if the player is in a jumping phase or not.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PlayerMovement {
    #[default]
    Idle,
    Jump,
    Run(PlayerDirection),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlayerDirection {
    Left,
    Right,
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("gabe-idle-run.png");
    let layout =
        TextureAtlasLayout::from_grid(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT), 7, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands.spawn((
        SpriteSheetBundle {
            texture,
            sprite: Sprite { ..default() },
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            transform: Transform {
                scale: Vec3::splat(PLAYER_SCALE_FACTOR),
                translation: Vec3::new(0.0, 200.0, 1.0),
                ..default()
            },
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        JumpTimer(Timer::from_seconds(0.5, TimerMode::Once)),
        Player,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut animation_query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlas,
        &mut Sprite,
    )>,
    state: ResMut<State<PlayerMovement>>,
    mut next_state: ResMut<NextState<PlayerMovement>>,
    mut jump_timer: Query<&mut JumpTimer>,
) {
    let mut player_transform = player_query.single_mut();
    let mut jump_timer = jump_timer.single_mut();
    let mut direction_x = 0.0;
    let mut anim = |current_movement: PlayerMovement| match current_movement {
        PlayerMovement::Run(direction) => {
            for (indices, mut timer, mut texture, mut sprite) in &mut animation_query {
                timer.tick(time.delta());
                if timer.just_finished() {
                    match direction {
                        PlayerDirection::Left => sprite.flip_x = true,
                        PlayerDirection::Right => sprite.flip_x = false,
                    }
                    match state.get() {
                        PlayerMovement::Jump => {}
                        _ => {
                            texture.index = if texture.index == indices.last {
                                indices.first
                            } else {
                                texture.index + 1
                            };
                        }
                    }
                }
            }
        }
        PlayerMovement::Idle => {
            let (_, _, mut texture, _) = animation_query.single_mut();
            match state.get() {
                PlayerMovement::Jump => {}
                _ => {
                    texture.index = 0;
                }
            }
        }
        PlayerMovement::Jump => {
            let (_, _, mut texture, _) = animation_query.single_mut();
            texture.index = 3;
        }
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
    if keyboard_input.pressed(KeyCode::ArrowUp) && state.get() != &PlayerMovement::Jump {
        next_state.set(PlayerMovement::Jump);
        jump_timer.reset();
        current_movement = PlayerMovement::Jump;
        anim(current_movement);
    }

    if current_movement == PlayerMovement::Idle {
        anim(PlayerMovement::Idle);
    }

    // Calculate the new horizontal player position based on player input
    // let new_player_position =
    player_transform.translation.x += direction_x * PLAYER_SPEED * time.delta_seconds();

    if state.get() == &PlayerMovement::Jump {
        if jump_timer.fraction() > 0.5 {
            player_transform.translation.y -= PLAYER_SPEED * time.delta_seconds();
        } else {
            player_transform.translation.y += PLAYER_SPEED * time.delta_seconds();
        }
    }

    jump_timer.tick(time.delta());
    if jump_timer.just_finished() {
        next_state.set(PlayerMovement::Idle);
        player_transform.translation.y -= PLAYER_SPEED * time.delta_seconds();
    }
}
