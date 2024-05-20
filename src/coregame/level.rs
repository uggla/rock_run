use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileStorage, TileVisible};

use crate::{
    coregame::state::AppState,
    events::Restart,
    helpers::{
        self,
        tiled::{TiledMap, TilesetLayerToStorageEntity},
    },
    WINDOW_HEIGHT, WINDOW_WIDTH,
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
struct Menu;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background)
            .add_systems(
                OnEnter(AppState::GameCreate),
                (
                    toggle_level_background_visibility,
                    toggle_menu_background_visibility,
                ),
            )
            .add_systems(
                OnEnter(AppState::StartMenu),
                (
                    toggle_level_background_visibility,
                    toggle_menu_background_visibility,
                ),
            )
            .insert_resource(CurrentLevel { id: 1 })
            .add_event::<Restart>();
    }
}

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite_handle: Handle<Image> = asset_server.load("menu.jpg");

    commands.spawn((
        SpriteBundle {
            texture: sprite_handle,
            transform: Transform::from_xyz(-(WINDOW_WIDTH / 2.0 - 720.0 / 2.0), 0.0, 0.0),
            ..default()
        },
        Menu,
    ));

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("level01.tmx");
    commands.spawn((
        helpers::tiled::TiledMapBundle {
            tiled_map: map_handle.clone(),
            ..Default::default()
        },
        Level {
            id: 1,
            handle: map_handle.clone(),
            map: Map::new(
                "SHFXX\nXOFOO",
                WINDOW_WIDTH as usize,
                WINDOW_HEIGHT as usize,
            ),
        },
    ));
}

fn toggle_level_background_visibility(
    current_level: Res<CurrentLevel>,
    mut tile_query: Query<&mut TileVisible>,
    map_query: Query<(&Level, &TilesetLayerToStorageEntity), With<Level>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
) {
    for (level, tileset_layer_storage) in map_query.iter() {
        if level.id == current_level.id {
            for layer_entity in tileset_layer_storage.get_entities() {
                if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                    for tile in layer_tile_storage.iter().flatten() {
                        let mut tile_visible = tile_query.get_mut(*tile).unwrap();
                        tile_visible.0 = !tile_visible.0;
                    }
                }
            }
            break;
        }
    }
}

fn toggle_menu_background_visibility(mut menu: Query<&mut Visibility, With<Menu>>) {
    let mut visibility = menu.single_mut();
    if *visibility == Visibility::Visible {
        *visibility = Visibility::Hidden;
    } else {
        *visibility = Visibility::Visible;
    }
}
