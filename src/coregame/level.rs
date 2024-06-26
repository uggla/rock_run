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

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background)
            .add_systems(OnEnter(AppState::GameCreate), show_level_background)
            .add_systems(OnEnter(AppState::StartMenu), hide_level_background)
            .insert_resource(CurrentLevel { id: 1 })
            .add_event::<Restart>();
    }
}

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                "SHFXF\nXOFOO",
                WINDOW_WIDTH as usize,
                WINDOW_HEIGHT as usize,
            ),
        },
    ));
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
