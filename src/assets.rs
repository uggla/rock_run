use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_fluent::BundleAsset;

use crate::helpers;

#[derive(AssetCollection, Resource)]
pub struct RockRunAssets {
    // Localization files
    #[asset(path = "locales/en-US/main.ftl.ron")]
    pub english_locale: Handle<BundleAsset>,
    #[asset(path = "locales/fr-FR/main.ftl.ron")]
    pub french_locale: Handle<BundleAsset>,

    // Levels
    #[asset(path = "level01.tmx")]
    pub level01: Handle<helpers::tiled::TiledMap>,
    #[asset(path = "level02.tmx")]
    pub level02: Handle<helpers::tiled::TiledMap>,

    // Fonts
    #[asset(path = "fonts/Cute_Dino.ttf")]
    pub cute_dino_font: Handle<Font>,

    // Sprites
    #[asset(path = "sprites/life.png")]
    pub life: Handle<Image>,
}
