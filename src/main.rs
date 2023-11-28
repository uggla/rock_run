use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_state::<PlayerMovement>()
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct JumpTimer(Timer);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(3.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        JumpTimer(Timer::from_seconds(0.5, TimerMode::Once)),
        Player,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(100.0, 10.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        },
        Ground,
    ));
}

const PLAYER_SPEED: f32 = 500.0;

// Player movement is used to define current movement requested by the player and a state in order
// to know if the player is in a jumping phase or not.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum PlayerMovement {
    #[default]
    Idle,
    Jump,
    Run,
}

enum PlayerDirection {
    Left,
    Right,
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut animation_query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
    state: ResMut<State<PlayerMovement>>,
    mut next_state: ResMut<NextState<PlayerMovement>>,
    mut jump_timer: Query<&mut JumpTimer>,
) {
    let mut player_transform = player_query.single_mut();
    let mut jump_timer = jump_timer.single_mut();
    let mut direction_x = 0.0;
    // let mut direction_y = 0.0;
    let mut anim =
        |current_movement: PlayerMovement, direction: PlayerDirection| match current_movement {
            PlayerMovement::Run => {
                for (indices, mut timer, mut sprite) in &mut animation_query {
                    timer.tick(time.delta());
                    if timer.just_finished() {
                        match direction {
                            PlayerDirection::Left => sprite.flip_x = true,
                            PlayerDirection::Right => sprite.flip_x = false,
                        }
                        match state.get() {
                            PlayerMovement::Jump => {}
                            _ => {
                                sprite.index = if sprite.index == indices.last {
                                    indices.first
                                } else {
                                    sprite.index + 1
                                };
                            }
                        }
                    }
                }
            }
            PlayerMovement::Idle => {
                let (_, _, mut sprite) = animation_query.single_mut();
                match state.get() {
                    PlayerMovement::Jump => {}
                    _ => {
                        sprite.index = 0;
                    }
                }
            }
            PlayerMovement::Jump => {
                let (_, _, mut sprite) = animation_query.single_mut();
                sprite.index = 3;
            }
        };

    let mut current_movement: PlayerMovement = PlayerMovement::Idle;
    if keyboard_input.pressed(KeyCode::Left) {
        direction_x -= 1.0;
        current_movement = PlayerMovement::Run;
        anim(current_movement, PlayerDirection::Left);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        direction_x += 1.0;
        current_movement = PlayerMovement::Run;
        anim(current_movement, PlayerDirection::Right);
    }
    if keyboard_input.pressed(KeyCode::Up) && state.get() != &PlayerMovement::Jump {
        next_state.set(PlayerMovement::Jump);
        jump_timer.reset();
        current_movement = PlayerMovement::Jump;
        anim(current_movement, PlayerDirection::Right);
    }

    if current_movement == PlayerMovement::Idle {
        anim(PlayerMovement::Idle, PlayerDirection::Right);
    }

    // Calculate the new horizontal player position based on player input
    // let new_player_position =
    player_transform.translation.x += direction_x * PLAYER_SPEED * time.delta_seconds();

    if state.get() == &PlayerMovement::Jump {
        if jump_timer.percent() > 0.5 {
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

    // Update the player position,
    // making sure it doesn't cause the player to leave the arena
    // let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + player_SIZE.x / 2.0 + PADDLE_PADDING;
    // let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - player_SIZE.x / 2.0 - PADDLE_PADDING;

    // player_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}
