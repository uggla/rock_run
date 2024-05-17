use bevy::prelude::*;

use crate::{
    coregame::{camera::CameraSet, state::AppState},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};

pub const LIFE_SCALE_FACTOR: f32 = 2.0;

#[derive(Resource, Default)]
pub struct Life {
    entities: Vec<Entity>,
}

// TODO: remove dead code
#[allow(dead_code)]
#[derive(Event)]
pub enum LifeEvent {
    Win,
    Lost,
}

#[derive(Component)]
pub struct LifeUI;

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameCreate), setup_life)
            .add_systems(OnEnter(AppState::StartMenu), despawn_life)
            .add_systems(Update, life_management)
            .insert_resource(Life::default())
            .add_systems(
                Update,
                show_life
                    .after(CameraSet)
                    .run_if(in_state(AppState::GameRunning)),
            )
            .add_event::<LifeEvent>();
    }
}

fn setup_life(mut commands: Commands, asset_server: Res<AssetServer>, mut life: ResMut<Life>) {
    let texture = asset_server.load("life.png");

    let parent = spawn_life_entity(&mut commands, &life, &texture);
    commands.entity(parent).insert(LifeUI);
    life.entities.push(parent);

    for _ in 0..2 {
        let child = spawn_life_entity(&mut commands, &life, &texture);
        commands.entity(parent).add_child(child);
        life.entities.push(child);
    }
}

fn spawn_life_entity(
    commands: &mut Commands,
    life: &ResMut<Life>,
    texture: &Handle<Image>,
) -> Entity {
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
    child
}

fn show_life(
    mut life_query: Query<&mut Transform, With<LifeUI>>,
    camera_query: Query<&mut Transform, (With<Camera2d>, Without<LifeUI>)>,
) {
    let mut life_ui = life_query.single_mut();
    let camera = camera_query.single();

    life_ui.translation = camera.translation
        + Vec3::new(
            -WINDOW_WIDTH / 2.0 + 20.0,
            WINDOW_HEIGHT / 2.0 - 20.0,
            100.0,
        );
    life_ui.scale = Vec3::splat(LIFE_SCALE_FACTOR);
}

fn life_management(
    mut commands: Commands,
    mut life_ui: Query<&Handle<Image>, With<LifeUI>>,
    mut life: ResMut<Life>,
    mut life_event: EventReader<LifeEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in life_event.read() {
        match ev {
            LifeEvent::Win => {
                let texture = life_ui.single_mut();
                let child = spawn_life_entity(&mut commands, &life, texture);
                commands.entity(life.entities[0]).add_child(child);
                life.entities.push(child);
            }
            LifeEvent::Lost => match life.entities.pop() {
                Some(entity) => {
                    commands.entity(entity).despawn_recursive();
                    debug!("life left: {}", life.entities.len());
                    if life.entities.is_empty() {
                        next_state.set(AppState::GameOver);
                    }
                }
                None => {
                    unreachable!("No life left to despawn");
                }
            },
        }
    }
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
