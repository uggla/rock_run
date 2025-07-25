use std::env;

use bevy::audio::Volume;
use bevy::ecs::system::SystemParam;
use bevy::window::PrimaryWindow;
use bevy::{app::AppExit, audio::PlaybackMode};

use bevy::prelude::*;
use bevy_fluent::{BundleAsset, Locale};
use bevy_pkv::PkvStore;
use bevy_rapier2d::plugin::RapierConfiguration;
use leafwing_input_manager::axislike::AxisDirection;
use leafwing_input_manager::prelude::GamepadControlDirection;
use leafwing_input_manager::{
    Actionlike, action_state::ActionState, input_map::InputMap, plugin::InputManagerPlugin,
};
use unic_langid::langid;

use crate::WINDOW_HEIGHT;
use crate::events::{NextLevel, StartGame};
use crate::{
    WINDOW_WIDTH,
    assets::RockRunAssets,
    coregame::{
        level::CurrentLevel,
        localization::get_translation,
        state::{AppState, ForState},
    },
    elements::story::SelectionDirection,
    events::{LadderCollisionStop, NoMoreStoryMessages, SelectionChanged, StoryMessages},
};

const LAST_LEVEL: u8 = 3;

#[derive(Component)]
pub struct DrawBlinkTimer(pub Timer);

#[derive(Component)]
struct Sel0;

#[derive(Component)]
struct Sel1;

#[derive(Component)]
struct Sel2;

// List of user actions associated to menu/ui interaction
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum MenuAction {
    // Starts the game when in the start screen
    // Go to the start screen when in the game over screen
    Accept,
    // During gameplay, pause the game.
    // Also unpause the game when in the pause screen.
    PauseUnpause,
    // During gameplay, directly exit to the initial screen.
    ExitToMenu,
    // During non-gameplay screens, quit the game
    Quit,
    Up,
    Down,
    Right,
    Left,
}

#[derive(Debug, Resource)]
pub struct Godmode(pub bool);

#[derive(Debug, Resource)]
struct StartLevel(u8);

#[derive(Debug, Resource)]
pub struct StartPos(pub Option<Vec2>);

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<MenuAction>::default())
            .add_systems(OnEnter(AppState::StartMenu), start_menu)
            .add_systems(OnEnter(AppState::GamePaused), pause_menu)
            .add_systems(OnExit(AppState::GamePaused), exit_pause_menu)
            .add_systems(OnEnter(AppState::GameOver), gameover_menu)
            .add_systems(OnEnter(AppState::GameFinished), gamefinished_menu)
            .add_systems(
                Update,
                (menu_input_system, menu_blink_system, game_messages),
            )
            .add_systems(Update, update_menu.run_if(in_state(AppState::StartMenu)))
            .add_systems(OnEnter(AppState::Loading), setup)
            .add_event::<StartGame>()
            .insert_resource(Godmode(false))
            .insert_resource(StartLevel(1))
            .insert_resource(StartPos(None))
            .insert_resource(PkvStore::new("DamageInc", "RockRun_v0_3_0"));
    }
}

#[derive(SystemParam)]
struct ScreenResolution<'w, 's> {
    query: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
}

fn setup(
    mut commands: Commands,
    mut godmode: ResMut<Godmode>,
    mut start_level: ResMut<StartLevel>,
    mut start_position: ResMut<StartPos>,
) {
    info!("setup");

    match env::var("ROCKRUN_GOD_MODE") {
        Ok(_) => godmode.0 = true,
        Err(_) => godmode.0 = false,
    }

    match env::var("ROCKRUN_LEVEL") {
        Ok(level) => {
            let level = level.parse::<u8>().expect("ROCKRUN_LEVEL is not a number");
            match level {
                1..=3 => start_level.0 = level,
                _ => start_level.0 = 1,
            }
        }
        Err(_) => start_level.0 = 1,
    }

    match env::var("ROCKRUN_START_POSITION") {
        Ok(position) => {
            let msg = "ROCKRUN_START_POSITION is not formated properly";
            let position = position.split(',').collect::<Vec<&str>>();
            start_position.0 = Some(Vec2::new(
                position[0].trim().parse::<f32>().expect(msg),
                position[1].trim().parse::<f32>().expect(msg),
            ));
        }
        Err(_) => start_position.0 = None,
    }

    let mut input_map = InputMap::<MenuAction>::new([
        (MenuAction::Accept, KeyCode::Space),
        (MenuAction::PauseUnpause, KeyCode::Escape),
        (MenuAction::ExitToMenu, KeyCode::Backspace),
        (MenuAction::Up, KeyCode::ArrowUp),
        (MenuAction::Down, KeyCode::ArrowDown),
        (MenuAction::Right, KeyCode::ArrowRight),
        (MenuAction::Left, KeyCode::ArrowLeft),
    ]);
    input_map.insert(MenuAction::ExitToMenu, GamepadButton::Select);
    input_map.insert(MenuAction::PauseUnpause, GamepadButton::Start);
    input_map.insert(MenuAction::Accept, GamepadButton::South);
    input_map.insert(
        MenuAction::Right,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickX,

            direction: AxisDirection::Positive,
            threshold: 0.8,
        },
    );
    input_map.insert(
        MenuAction::Left,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickX,

            direction: AxisDirection::Negative,
            threshold: 0.8,
        },
    );
    input_map.insert(
        MenuAction::Up,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickY,

            direction: AxisDirection::Positive,
            threshold: 0.8,
        },
    );
    input_map.insert(
        MenuAction::Down,
        GamepadControlDirection {
            axis: GamepadAxis::LeftStickY,

            direction: AxisDirection::Negative,
            threshold: 0.8,
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    input_map.insert(MenuAction::Quit, GamepadButton::East);
    #[cfg(not(target_arch = "wasm32"))]
    input_map.insert(MenuAction::Quit, KeyCode::Escape);

    // Insert MenuAction resources
    commands.insert_resource(input_map);
    commands.insert_resource(ActionState::<MenuAction>::default());
}

fn start_menu(
    mut commands: Commands,
    mut locale: ResMut<Locale>,
    assets: Res<Assets<BundleAsset>>,
    rock_run_assets: Res<RockRunAssets>,
    pkv: ResMut<PkvStore>,
    mut screen: ScreenResolution,
) {
    info!("start_menu");
    if let Ok(mut window) = screen.query.single_mut() {
        info!("Resolution: {}x{}", window.width(), window.height());

        if window.width() != WINDOW_WIDTH || window.height() != WINDOW_HEIGHT {
            warn!("Incorrect resolution detected, reapplying the setting.");
            window
                .resolution
                .set_physical_resolution(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
            info!("New resolution: {}x{}", window.width(), window.height());
        }
    }

    const TOP_MARGINS: [f32; 4] = [185.0, 290.0, 395.0, 500.0];

    if let Ok(langid) = pkv.get::<String>("langid") {
        match langid.as_str() {
            "fr-FR" => locale.requested = langid!("fr-FR"),
            "en-US" => locale.requested = langid!("en-US"),
            _ => locale.requested = langid!("en-US"),
        }
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ForState {
                states: vec![AppState::StartMenu],
            },
        ))
        // Right column
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(720.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                ImageNode::new(rock_run_assets.menu.clone()),
            ));
        })
        // Left column
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(WINDOW_WIDTH - 720.0),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    ImageNode::new(rock_run_assets.menu2.clone()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(TOP_MARGINS[0]),
                            ..default()
                        },
                        Text::new("Menu"),
                        TextFont {
                            font: rock_run_assets.cute_dino_font.clone(),
                            font_size: 55.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(0x54, 0x2E, 0x0A)),
                    ));
                })
                // English box
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(TOP_MARGINS[1]),
                            ..default()
                        })
                        .with_children(|parent| {
                            // lang01 flag
                            parent.spawn((
                                Node {
                                    justify_content: JustifyContent::Start,
                                    width: Val::Px(66.0),
                                    right: Val::Px(30.0),
                                    ..default()
                                },
                                BackgroundColor(Color::WHITE),
                                ImageNode::new(rock_run_assets.en_flag.clone()),
                            ));

                            // lang01 text
                            parent.spawn((
                                Node {
                                    justify_content: JustifyContent::Start,
                                    ..default()
                                },
                                Text::new(get_translation(
                                    &locale,
                                    &assets,
                                    &rock_run_assets,
                                    "lang01",
                                    None,
                                )),
                                TextFont {
                                    font: rock_run_assets.cute_dino_font.clone(),
                                    font_size: 40.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(0x54, 0x2E, 0x0A)),
                                Sel2,
                            ));
                        });
                })
                // French box
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(TOP_MARGINS[2]),
                            ..default()
                        })
                        .with_children(|parent| {
                            let style_flag = if locale.requested == langid!("fr-FR") {
                                Node {
                                    justify_content: JustifyContent::Start,
                                    width: Val::Px(66.0),
                                    right: Val::Px(20.0),
                                    ..default()
                                }
                            } else {
                                Node {
                                    justify_content: JustifyContent::Start,
                                    width: Val::Px(66.0),
                                    right: Val::Px(33.0),
                                    ..default()
                                }
                            };

                            let style_lang = if locale.requested == langid!("fr-FR") {
                                Node {
                                    justify_content: JustifyContent::Start,
                                    left: Val::Px(13.0),
                                    ..default()
                                }
                            } else {
                                Node {
                                    justify_content: JustifyContent::Start,
                                    left: Val::Px(-2.0),
                                    ..default()
                                }
                            };

                            // lang02 flag
                            parent.spawn((
                                style_flag,
                                ImageNode::new(rock_run_assets.fr_flag.clone()),
                            ));

                            // lang02 text
                            parent.spawn((
                                style_lang,
                                Text::new(get_translation(
                                    &locale,
                                    &assets,
                                    &rock_run_assets,
                                    "lang02",
                                    None,
                                )),
                                TextFont {
                                    font: rock_run_assets.cute_dino_font.clone(),
                                    font_size: 40.0,
                                    ..default()
                                },
                                TextColor(Color::srgb_u8(0x54, 0x2E, 0x0A)),
                                Sel1,
                            ));
                        });
                })
                // start instruction
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(TOP_MARGINS[3]),
                            ..default()
                        },
                        Text::new(get_translation(
                            &locale,
                            &assets,
                            &rock_run_assets,
                            "start_game",
                            None,
                        )),
                        TextFont {
                            font: rock_run_assets.cute_dino_font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb_u8(0x54, 0x2E, 0x0A)),
                        Sel0,
                    ));
                });
        });
}

type Select0 = (With<Sel0>, Without<Sel1>, Without<Sel2>);
type Select1 = (With<Sel1>, Without<Sel0>, Without<Sel2>);
type Select2 = (With<Sel2>, Without<Sel0>, Without<Sel1>);

#[allow(clippy::too_many_arguments)]
fn update_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_sel: Local<i8>,
    mut locale: ResMut<Locale>,
    menu_action_state: Res<ActionState<MenuAction>>,
    mut query0: Query<(&mut Text, &mut TextColor), Select0>,
    mut query1: Query<(&mut Text, &mut TextColor), Select1>,
    mut query2: Query<(&mut Text, &mut TextColor), Select2>,
    assets: Res<Assets<BundleAsset>>,
    rock_run_assets: Res<RockRunAssets>,
    mut pkv: ResMut<PkvStore>,
) -> Result<()> {
    enum MenuColor {
        Selected,
        CurrentLang,
        OtherLang,
    }

    impl MenuColor {
        fn color(&self) -> Color {
            match self {
                MenuColor::Selected => Color::srgb_u8(0xD3, 0xCD, 0x39),
                MenuColor::CurrentLang => Color::srgb_u8(0xF4, 0x78, 0x04),
                MenuColor::OtherLang => Color::srgb_u8(0x54, 0x2E, 0x0A),
            }
        }
    }

    let (mut sel0_text, mut sel0_color) = query0.single_mut()?;
    let (mut sel1_text, mut sel1_color) = query1.single_mut()?;
    let (mut sel2_text, mut sel2_color) = query2.single_mut()?;

    if menu_action_state.just_pressed(&MenuAction::Up) {
        *menu_sel = (*menu_sel + 1) % 3;
        debug!("menu_sel: {}", *menu_sel);
    }

    if menu_action_state.just_pressed(&MenuAction::Down) {
        if *menu_sel == 0 {
            *menu_sel = 3;
        }
        *menu_sel = (*menu_sel - 1) % 3;
        debug!("menu_sel: {}", *menu_sel);
    }

    if menu_action_state.just_pressed(&MenuAction::Accept) {
        match *menu_sel {
            0 => {
                info!("start");
                next_state.set(AppState::GameCreate);
            }
            1 => {
                info!("French");
                locale.requested = langid!("fr-FR");
                pkv.set_string("langid", "fr-FR")
                    .expect("failed to store langid");
                refresh_menu_items(
                    &locale,
                    assets,
                    rock_run_assets,
                    &mut sel0_text,
                    &mut sel1_text,
                    &mut sel2_text,
                );
            }
            2 => {
                info!("English");
                locale.requested = langid!("en-US");
                pkv.set_string("langid", "en-US")
                    .expect("failed to store langid");
                refresh_menu_items(
                    &locale,
                    assets,
                    rock_run_assets,
                    &mut sel0_text,
                    &mut sel1_text,
                    &mut sel2_text,
                );
            }
            _ => {}
        }
    }

    match *menu_sel {
        0 => {
            if locale.requested == langid!("fr-FR") {
                *sel1_color = TextColor(MenuColor::color(&MenuColor::CurrentLang));
                *sel2_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
            } else {
                *sel2_color = TextColor(MenuColor::color(&MenuColor::CurrentLang));
                *sel1_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
            }
            *sel0_color = TextColor(MenuColor::color(&MenuColor::Selected));
        }
        1 => {
            if locale.requested == langid!("fr-FR") {
                *sel1_color = TextColor(MenuColor::color(&MenuColor::Selected));
                *sel2_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
            } else {
                *sel2_color = TextColor(MenuColor::color(&MenuColor::CurrentLang));
                *sel1_color = TextColor(MenuColor::color(&MenuColor::Selected));
            }
            *sel0_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
        }
        2 => {
            if locale.requested == langid!("fr-FR") {
                *sel1_color = TextColor(MenuColor::color(&MenuColor::CurrentLang));
                *sel2_color = TextColor(MenuColor::color(&MenuColor::Selected));
            } else {
                *sel2_color = TextColor(MenuColor::color(&MenuColor::Selected));
                *sel1_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
            }
            *sel0_color = TextColor(MenuColor::color(&MenuColor::OtherLang));
        }
        _ => {}
    }
    Ok(())
}

fn refresh_menu_items(
    locale: &ResMut<Locale>,
    assets: Res<Assets<BundleAsset>>,
    rock_run_assets: Res<RockRunAssets>,
    sel0: &mut Text,
    sel1: &mut Text,
    sel2: &mut Text,
) {
    // Refresh menu items in case we has just changed the locale
    *sel0 = Text::new(get_translation(
        locale,
        &assets,
        &rock_run_assets,
        "start_game",
        None,
    ));
    *sel1 = Text::new(get_translation(
        locale,
        &assets,
        &rock_run_assets,
        "lang02",
        None,
    ));
    *sel2 = Text::new(get_translation(
        locale,
        &assets,
        &rock_run_assets,
        "lang01",
        None,
    ));
}

fn gamefinished_menu(mut commands: Commands, rock_run_assets: Res<RockRunAssets>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            ForState {
                states: vec![AppState::GameOver],
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(720.0),
                    ..default()
                },
                ImageNode::new(rock_run_assets.victory.clone()),
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                AudioPlayer::new(rock_run_assets.victory_sound.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    // volume: Volume::Linear(4.3),
                    ..default()
                },
            ));
        });
}

fn gameover_menu(mut commands: Commands, rock_run_assets: Res<RockRunAssets>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            ForState {
                states: vec![AppState::GameOver],
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(720.0),
                    ..default()
                },
                ImageNode::new(rock_run_assets.gameover.clone()),
            ));
        });

    commands.spawn((
        AudioPlayer::new(rock_run_assets.loose_sound.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(2.5),
            ..default()
        },
    ));
}

fn pause_menu(mut commands: Commands, rock_run_assets: Res<RockRunAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ForState {
                states: vec![AppState::GamePaused],
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pause"),
                TextFont {
                    font: rock_run_assets.cute_dino_font.clone(),
                    font_size: 100.0,
                    ..default()
                },
                TextColor(Color::srgb_u8(0xF8, 0xE4, 0x73)),
                DrawBlinkTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                AudioPlayer::new(rock_run_assets.pause_in_sound.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::Linear(4.3),
                    ..default()
                },
            ));
        });
}

fn exit_pause_menu(mut commands: Commands, rock_run_assets: Res<RockRunAssets>) {
    commands.spawn((
        AudioPlayer::new(rock_run_assets.pause_out_sound.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(4.3),
            ..default()
        },
    ));
}

fn menu_blink_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DrawBlinkTimer, &ViewVisibility)>,
) {
    for (entity, mut timer, visibility) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let new_visibility = if visibility.get() {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
            commands.entity(entity).insert(new_visibility);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn menu_input_system(
    state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    menu_action_state: Res<ActionState<MenuAction>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut rapier_config: Query<&mut RapierConfiguration>,
    mut msg_event: EventWriter<StoryMessages>,
    mut selection_event: EventWriter<SelectionChanged>,
    mut no_more_msg_event: EventReader<NoMoreStoryMessages>,
    mut ladder_collision_stop: EventWriter<LadderCollisionStop>,
    mut game_event_start: EventWriter<StartGame>,
    mut game_event_level: EventWriter<NextLevel>,
    mut current_level: ResMut<CurrentLevel>,
    start_level: Res<StartLevel>,
) {
    let mut rapier_config = rapier_config
        .single_mut()
        .expect("Can't get rapier configuration.");
    if state.get() != &AppState::StartMenu
        && menu_action_state.just_pressed(&MenuAction::ExitToMenu)
    {
        next_state.set(AppState::StartMenu);
        rapier_config.physics_pipeline_active = true;
        msg_event.write(StoryMessages::Hide);
        ladder_collision_stop.write(LadderCollisionStop);
    } else {
        match state.get() {
            AppState::StartMenu => {
                current_level.id = start_level.0;
                if menu_action_state.just_pressed(&MenuAction::Quit) {
                    app_exit_events.write(AppExit::Success);
                }
            }
            AppState::GameCreate => {
                next_state.set(AppState::GameRunning);
                game_event_start.write(StartGame);
            }
            AppState::GameRunning => {
                if menu_action_state.just_pressed(&MenuAction::PauseUnpause) {
                    next_state.set(AppState::GamePaused);
                    rapier_config.physics_pipeline_active = false;
                }
            }
            AppState::GamePaused => {
                if menu_action_state.just_pressed(&MenuAction::PauseUnpause) {
                    next_state.set(AppState::GameRunning);
                    rapier_config.physics_pipeline_active = true;
                }
            }
            AppState::GameMessage => {
                if !no_more_msg_event.is_empty() {
                    // No more messages to display
                    next_state.set(AppState::GameRunning);
                    rapier_config.physics_pipeline_active = true;
                    debug!("no more message, hide messages window");
                    msg_event.write(StoryMessages::Hide);
                    no_more_msg_event.clear();
                }
                if menu_action_state.just_pressed(&MenuAction::Accept) {
                    // we still have messages to display
                    debug!("next message");
                    msg_event.write(StoryMessages::Next);
                }
                if menu_action_state.just_pressed(&MenuAction::Right) {
                    // Selection to the right
                    debug!("selection right");
                    selection_event.write(SelectionChanged {
                        movement: SelectionDirection::Right,
                    });
                }
                if menu_action_state.just_pressed(&MenuAction::Left) {
                    // Selection to the left
                    debug!("selection left");
                    selection_event.write(SelectionChanged {
                        movement: SelectionDirection::Left,
                    });
                }
                if menu_action_state.just_pressed(&MenuAction::Up) {
                    // Selection up
                    debug!("selection up");
                    selection_event.write(SelectionChanged {
                        movement: SelectionDirection::Up,
                    });
                }
                if menu_action_state.just_pressed(&MenuAction::Down) {
                    // Selection up
                    debug!("selection down");
                    selection_event.write(SelectionChanged {
                        movement: SelectionDirection::Down,
                    });
                }
                if menu_action_state.just_pressed(&MenuAction::PauseUnpause) {
                    // User request to close the messages window
                    next_state.set(AppState::GameRunning);
                    rapier_config.physics_pipeline_active = true;
                    debug!("hide messages window");
                    msg_event.write(StoryMessages::Hide);
                }
            }
            AppState::GameOver => {
                if menu_action_state.just_pressed(&MenuAction::Accept) {
                    next_state.set(AppState::StartMenu);
                }
            }
            AppState::Loading => {
                // This state is used to load assets.
            }
            AppState::FinishLevel => {
                // Mostly used to despawn stuff
                if current_level.id == LAST_LEVEL {
                    next_state.set(AppState::GameFinished);
                } else {
                    current_level.id += 1;
                    next_state.set(AppState::NextLevel);
                }
            }
            AppState::NextLevel => {
                next_state.set(AppState::GameRunning);
                game_event_level.write(NextLevel);
            }
            AppState::GameFinished => {
                if menu_action_state.just_pressed(&MenuAction::Accept) {
                    next_state.set(AppState::StartMenu);
                }
            }
        }
    }
}

fn game_messages(
    mut next_state: ResMut<NextState<AppState>>,
    mut msg_event: EventReader<StoryMessages>,
) {
    for ev in msg_event.read() {
        match ev {
            StoryMessages::Next => {}
            StoryMessages::Hide => {}
            StoryMessages::Display(_) => {
                next_state.set(AppState::GameMessage);
            }
        }
    }
}
