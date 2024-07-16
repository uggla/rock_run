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
    #[asset(path = "sprites/girl.png")]
    pub player: Handle<Image>,
    #[asset(path = "sprites/life.png")]
    pub life: Handle<Image>,
    #[asset(path = "sprites/story_qm.png")]
    pub story_qm: Handle<Image>,
    #[asset(path = "sprites/rock_ball.png")]
    pub rock_ball: Handle<Image>,
    #[asset(path = "sprites/moving_platform.png")]
    pub moving_platform: Handle<Image>,
    #[asset(path = "sprites/warrior.png")]
    pub warrior: Handle<Image>,
    #[asset(path = "sprites/gate.png")]
    pub gate: Handle<Image>,
    #[asset(path = "sprites/triceratops.png")]
    pub triceratops: Handle<Image>,
    #[asset(path = "sprites/pterodactyl.png")]
    pub pterodactyl: Handle<Image>,
    #[asset(path = "sprites/rock_small.png")]
    pub rock_small: Handle<Image>,
    #[asset(path = "sprites/bat.png")]
    pub bat: Handle<Image>,
    // #[asset(path = "sprites/tyrannosaurus.png")]
    // pub tyrannosaurus: Handle<Image>,

    // Images
    #[asset(path = "images/menu.jpg")]
    pub menu: Handle<Image>,
    #[asset(path = "images/menu2.jpg")]
    pub menu2: Handle<Image>,
    #[asset(path = "images/en.png")]
    pub en_flag: Handle<Image>,
    #[asset(path = "images/fr.png")]
    pub fr_flag: Handle<Image>,
    #[asset(path = "images/victory.jpg")]
    pub victory: Handle<Image>,
    #[asset(path = "images/gameover.jpg")]
    pub gameover: Handle<Image>,

    // Sounds
    #[asset(path = "sounds/jump.ogg")]
    pub jump_sound: Handle<AudioSource>,
    #[asset(path = "sounds/hit.ogg")]
    pub hit_sound: Handle<AudioSource>,
    #[asset(path = "sounds/loose.ogg")]
    pub loose_sound: Handle<AudioSource>,

    // Music
    #[asset(path = "musics/theme_01.ogg")]
    pub music_level01: Handle<AudioSource>,
    #[asset(path = "musics/theme_03.ogg")]
    pub music_level02: Handle<AudioSource>,
}
