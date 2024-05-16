use bevy::prelude::*;

use crate::coregame::state::AppState;

pub const LIFE_SCALE_FACTOR: f32 = 2.0;

#[derive(Component)]
pub struct Life;

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_life)
            .add_systems(OnEnter(AppState::StartMenu), despawn_life);
        // .add_systems(Update, show_life.run_if(in_state(AppState::GameRunning)));
    }
}

pub fn setup_life(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("life.png");

    commands.spawn((
        SpriteSheetBundle {
            texture,
            sprite: Sprite { ..default() },
            transform: Transform {
                scale: Vec3::splat(LIFE_SCALE_FACTOR),
                translation: Vec3::new(0.0, 0.0, 100.0),
                ..default()
            },
            ..default()
        },
        Life,
    ));
}

fn despawn_life(mut commands: Commands, life: Query<Entity, With<Life>>) {
    if let Ok(life) = life.get_single() {
        commands.entity(life).despawn_recursive();
    }
}
