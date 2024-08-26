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
    #[asset(path = "sprites/trex.png")]
    pub trex: Handle<Image>,
    #[asset(path = "sprites/squirel.png")]
    pub squirel: Handle<Image>,
    #[asset(path = "sprites/nut.png")]
    pub nut: Handle<Image>,

    #[asset(path = "sprites/vine1.png")]
    pub vine1: Handle<Image>,
    #[asset(path = "sprites/vine2.png")]
    pub vine2: Handle<Image>,
    #[asset(path = "sprites/vine2_end.png")]
    pub vine2_end: Handle<Image>,
    #[asset(path = "sprites/vine_left.png")]
    pub vine_left: Handle<Image>,
    #[asset(path = "sprites/vine_left_end.png")]
    pub vine_left_end: Handle<Image>,
    #[asset(path = "sprites/vine_right.png")]
    pub vine_right: Handle<Image>,
    #[asset(path = "sprites/vine_right_end.png")]
    pub vine_right_end: Handle<Image>,

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
    #[asset(path = "sounds/pterodactyl.ogg")]
    pub pterodactyl_sound: Handle<AudioSource>,
    #[asset(path = "sounds/trex_rush.ogg")]
    pub trex_rush_sound: Handle<AudioSource>,
    #[asset(path = "sounds/trex_bite.ogg")]
    pub trex_bite_sound: Handle<AudioSource>,
    #[asset(path = "sounds/bat.ogg")]
    pub bat_sound: Handle<AudioSource>,
    #[asset(path = "sounds/story_change.ogg")]
    pub story_change_sound: Handle<AudioSource>,
    #[asset(path = "sounds/story_plus.ogg")]
    pub story_plus_sound: Handle<AudioSource>,
    #[asset(path = "sounds/story_minus.ogg")]
    pub story_minus_sound: Handle<AudioSource>,
    #[asset(path = "sounds/story_valid.ogg")]
    pub story_valid_sound: Handle<AudioSource>,
    #[asset(path = "sounds/story_wrong.ogg")]
    pub story_wrong_sound: Handle<AudioSource>,
    #[asset(path = "sounds/get_something.ogg")]
    pub get_something_sound: Handle<AudioSource>,
    #[asset(path = "sounds/pause_in.ogg")]
    pub pause_in_sound: Handle<AudioSource>,
    #[asset(path = "sounds/pause_out.ogg")]
    pub pause_out_sound: Handle<AudioSource>,

    // Music
    #[asset(path = "musics/theme_01.ogg")]
    pub music_level01: Handle<AudioSource>,
    #[asset(path = "musics/theme_03.ogg")]
    pub music_level02: Handle<AudioSource>,
}
