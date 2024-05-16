use bevy::prelude::*;

use crate::coregame::{camera::CameraSet, state::AppState};

pub const LIFE_SCALE_FACTOR: f32 = 2.0;

#[derive(Resource, Default)]
pub struct Life {
    entities: Vec<Entity>,
}

#[derive(Component)]
pub struct LifeUI;

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_life)
            .add_systems(OnEnter(AppState::StartMenu), despawn_life)
            .insert_resource(Life::default())
            .add_systems(
                Update,
                show_life
                    .after(CameraSet)
                    .run_if(in_state(AppState::GameRunning)),
            );
    }
}

pub fn setup_life(mut commands: Commands, asset_server: Res<AssetServer>, mut life: ResMut<Life>) {
    let texture = asset_server.load("life.png");

    let parent = commands
        .spawn((
            SpriteBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    scale: Vec3::splat(LIFE_SCALE_FACTOR),
                    translation: Vec3::new(0.0, 0.0, 100.0),
                    ..default()
                },
                ..default()
            },
            LifeUI,
        ))
        .id();

    life.entities.push(parent);

    for _ in 0..2 {
        let x_offset = life.entities.len() as f32 * 20.0;
        let child = commands
            .spawn(SpriteBundle {
                texture: texture.clone(),
                sprite: Sprite { ..default() },
                transform: Transform {
                    translation: Vec3::new(x_offset, 0.0, 0.0),
                    ..default()
                },
                ..default()
            })
            .id();
        commands.entity(parent).add_child(child);
        life.entities.push(child);
    }
}

fn show_life(
    mut life_query: Query<&mut Transform, With<LifeUI>>,
    camera_query: Query<&mut Transform, (With<Camera2d>, Without<LifeUI>)>,
) {
    let mut life_ui = life_query.single_mut();
    let camera = camera_query.single();

    life_ui.translation = camera.translation + Vec3::new(-620.0, 340.0, 100.0);
}

fn despawn_life(
    mut commands: Commands,
    life_ui: Query<Entity, With<LifeUI>>,
    mut life: ResMut<Life>,
) {
    if let Ok(life_ui) = life_ui.get_single() {
        commands.entity(life_ui).despawn_recursive();
        life.entities.clear();
    }
}
