use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::tiles::{TileStorage, TileVisible};
use bevy_fluent::{BundleAsset, Locale};

use crate::{
    assets::RockRunAssets,
    coregame::{
        localization::{convert_to_fluent_args, get_translation},
        state::AppState,
    },
    events::{NextLevel, PositionSensorCollisionStart, PositionSensorCollisionStop, Restart},
    helpers::{
        self,
        tiled::{TiledMap, TilesetLayerToStorageEntity},
    },
    player, WINDOW_HEIGHT, WINDOW_WIDTH,
};

use crate::screen_map::Map;

#[derive(Resource, PartialEq)]
pub struct CurrentLevel {
    pub id: u8,
}

#[derive(Component, PartialEq)]
pub struct Level {
    pub id: u8,
    pub handle: Handle<TiledMap>,
    pub map: Map,
}

#[derive(Component)]
struct DisplayLevel;

#[derive(Component)]
struct DisplayLevelText;

#[derive(Component, Deref, DerefMut)]
struct DisplayLevelTimer(Timer);

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Loading), setup_background)
            .add_systems(
                OnEnter(AppState::GameCreate),
                (show_level_background, show_current_level),
            )
            .add_systems(
                OnEnter(AppState::NextLevel),
                (show_level_background, show_current_level),
            )
            .add_systems(
                OnEnter(AppState::StartMenu),
                (hide_level_background, despawn_display_level),
            )
            .add_systems(OnEnter(AppState::FinishLevel), hide_level_background)
            .add_systems(
                Update,
                (check_exit, fade_display_level).run_if(in_state(AppState::GameRunning)),
            )
            .insert_resource(CurrentLevel { id: 1 })
            .add_event::<Restart>()
            .add_event::<NextLevel>();
    }
}

fn setup_background(mut commands: Commands, rock_run_assets: Res<RockRunAssets>) {
    let map_handle: Handle<helpers::tiled::TiledMap> = rock_run_assets.level01.clone();
    commands.spawn((
        helpers::tiled::TiledMapBundle {
            tiled_map: map_handle.clone(),
            ..Default::default()
        },
        Level {
            id: 1,
            handle: map_handle.clone(),
            map: Map::new("SOOOOOOOOO", WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize),
        },
    ));

    let map_handle: Handle<helpers::tiled::TiledMap> = rock_run_assets.level02.clone();
    commands.spawn((
        helpers::tiled::TiledMapBundle {
            tiled_map: map_handle.clone(),
            ..Default::default()
        },
        Level {
            id: 2,
            handle: map_handle.clone(),
            map: Map::new(
                "SHFXF\nXOFOO",
                WINDOW_WIDTH as usize,
                WINDOW_HEIGHT as usize,
            ),
        },
    ));
}

fn show_current_level(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    rock_run_assets: Res<RockRunAssets>,
    locale: Res<Locale>,
    assets: Res<Assets<BundleAsset>>,
    levels: Query<&Level, With<Level>>,
) {
    info!("show_current_level {:?}", current_level.id);

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite { ..default() },
                transform: Transform::from_translation(
                    level.map.get_start_screen().get_center().xy().extend(10.0),
                ),
                ..default()
            },
            DisplayLevel,
            DisplayLevelTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ))
        // right column
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    text: Text::from_section(
                        get_translation(
                            &locale,
                            &assets,
                            &rock_run_assets,
                            "current_level",
                            convert_to_fluent_args(Some(HashMap::from([(
                                "current_level".to_string(),
                                current_level.id.to_string(),
                            )])))
                            .as_ref(),
                        ),
                        TextStyle {
                            font: rock_run_assets.cute_dino_font.clone(),
                            font_size: 60.0,
                            color: Color::rgb_u8(0xF4, 0x78, 0x04),
                        },
                    ),
                    ..default()
                },
                DisplayLevelText,
            ));
        });
}

fn fade_display_level(
    mut commands: Commands,
    time: Res<Time>,
    mut display_level_timer: Query<&mut DisplayLevelTimer>,
    mut display_level: Query<Entity, With<DisplayLevel>>,
    mut display_level_text: Query<&mut Text, With<DisplayLevelText>>,
) {
    if let Ok(mut display_level_timer) = display_level_timer.get_single_mut() {
        display_level_timer.tick(time.delta());

        if display_level_timer.finished() {
            let mut text = display_level_text.single_mut();
            let transparency = text.sections[0].style.color.a();
            let color = Color::rgb_u8(0xF4, 0x78, 0x04).as_rgba();
            text.sections[0].style.color =
                Color::rgba(color.r(), color.g(), color.b(), transparency - 0.02);

            if transparency < 0.0 {
                commands
                    .entity(display_level.single_mut())
                    .despawn_recursive();
            }
        }
    }
}

fn despawn_display_level(
    mut commands: Commands,
    mut display_level: Query<Entity, With<DisplayLevel>>,
) {
    if let Ok(display_level) = display_level.get_single_mut() {
        commands.entity(display_level).despawn_recursive();
    }
}

fn show_level_background(
    current_level: Res<CurrentLevel>,
    mut tile_query: Query<&mut TileVisible>,
    map_query: Query<(&Level, &TilesetLayerToStorageEntity), With<Level>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
) {
    let mut tiles = get_tiles(map_query, current_level, tile_storage_query);

    tiles.iter_mut().for_each(|tile| {
        let mut tile_visible = tile_query.get_mut(*tile).unwrap();
        tile_visible.0 = true;
    });
}

fn hide_level_background(
    current_level: Res<CurrentLevel>,
    mut tile_query: Query<&mut TileVisible>,
    map_query: Query<(&Level, &TilesetLayerToStorageEntity), With<Level>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
) {
    let mut tiles = get_tiles(map_query, current_level, tile_storage_query);

    tiles.iter_mut().for_each(|tile| {
        let mut tile_visible = tile_query.get_mut(*tile).unwrap();
        tile_visible.0 = false;
    });
}

fn get_tiles(
    map_query: Query<(&Level, &TilesetLayerToStorageEntity), With<Level>>,
    current_level: Res<CurrentLevel>,
    tile_storage_query: Query<(Entity, &TileStorage), ()>,
) -> Vec<Entity> {
    map_query
        .iter()
        .find(|(level, _)| level.id == current_level.id)
        .unwrap()
        .1
        .get_entities()
        .iter()
        .filter_map(|layer_entity| tile_storage_query.get(**layer_entity).ok())
        .flat_map(|(_, layer_tile_storage)| layer_tile_storage.iter().flatten().copied())
        .collect::<Vec<_>>()
}

fn check_exit(
    mut next_state: ResMut<NextState<AppState>>,
    mut sensor_collision_start: EventReader<PositionSensorCollisionStart>,
    mut sensor_collision_stop: EventReader<PositionSensorCollisionStop>,
    input: Query<
        &leafwing_input_manager::action_state::ActionState<player::PlayerMovement>,
        With<player::Player>,
    >,
    mut exit_collision: Local<bool>,
) {
    let input_state = match input.get_single() {
        Ok(state) => state,
        Err(_) => return,
    };

    for collision_event in sensor_collision_start.read() {
        if !collision_event.sensor_name.contains("exit") {
            return;
        }
        *exit_collision = true;
    }

    for collision_event in sensor_collision_stop.read() {
        if !collision_event.sensor_name.contains("exit") {
            return;
        }
        *exit_collision = false;
    }

    if *exit_collision && input_state.just_pressed(&player::PlayerMovement::Climb) {
        debug!("next level");
        *exit_collision = false;
        next_state.set(AppState::FinishLevel);
    }
}
