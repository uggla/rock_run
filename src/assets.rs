use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_fluent::BundleAsset;

#[derive(AssetCollection, Resource)]
pub struct RockRunAssets {
    #[asset(path = "locales/en-US/main.ftl.ron")]
    pub english_locale: Handle<BundleAsset>,
    #[asset(path = "locales/fr-FR/main.ftl.ron")]
    pub french_locale: Handle<BundleAsset>,
}
