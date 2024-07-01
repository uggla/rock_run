use crate::{
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::NoMoreStoryMessages,
    helpers::texture::cycle_texture,
    text_syllable::TextSyllableValues,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::geometry::Collider;
use rand::{thread_rng, Rng};

const WARRIOR_SCALE_FACTOR: f32 = 1.0;
const WARRIOR_WIDTH: f32 = 70.0;
const WARRIOR_HEIGHT: f32 = 65.0;

const GATE_SCALE_FACTOR: f32 = 1.0;
const GATE_WIDTH: f32 = 32.0;
const GATE_HEIGHT: f32 = 80.0;

#[derive(Component)]
pub struct Warrior;

#[derive(Component)]
pub struct Gate;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource, Debug)]
pub struct Enigmas {
    pub enigmas: Vec<Enigma>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Enigma {
    pub associated_story: String,
    pub kind: EnigmaKind,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnigmaKind {
    Qcm(Vec<String>),
    Numbers(HashMap<String, String>),
}

pub struct EnigmaPlugin;

impl Plugin for EnigmaPlugin {
    fn build(&self, app: &mut App) {
        let mut rng = thread_rng();
        let enigmas = Enigmas {
            enigmas: vec![Enigma {
                associated_story: "story03".to_string(),
                kind: EnigmaKind::Numbers(HashMap::from([
                    ("n1".to_string(), rng.gen_range(0..=50).to_string()),
                    ("n2".to_string(), rng.gen_range(0..50).to_string()),
                ])),
            }],
        };
        app.insert_resource(enigmas)
            .add_systems(OnEnter(AppState::GameCreate), spawn_warrior)
            .add_systems(
                OnEnter(AppState::StartMenu),
                (despawn_warrior, despawn_gate),
            )
            .add_systems(
                Update,
                (check_enigma, move_warrior, move_gate).run_if(in_state(AppState::GameRunning)),
            );
    }
}

fn spawn_warrior(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
) {
    info!("setup_warrior");

    if current_level.id != 1 {
        return;
    }

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    let texture = asset_server.load("warrior.png");
    let layout =
        TextureAtlasLayout::from_grid(Vec2::new(WARRIOR_WIDTH, WARRIOR_HEIGHT), 6, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);

    commands.spawn((
        SpriteSheetBundle {
            texture,
            sprite: Sprite { ..default() },
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
            transform: Transform {
                scale: Vec3::splat(WARRIOR_SCALE_FACTOR),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(5575.0, 608.0))
                    .extend(2.0),
                ..default()
            },
            ..default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Warrior,
    ));

    let texture = asset_server.load("gate.png");
    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite { ..default() },
            transform: Transform {
                scale: Vec3::splat(GATE_SCALE_FACTOR),
                translation: level
                    .map
                    .tiled_to_bevy_coord(Vec2::new(5488.0, 615.0))
                    .extend(1.0),
                ..default()
            },
            ..default()
        },
        Collider::cuboid(GATE_WIDTH / 2.0, GATE_HEIGHT / 2.0),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Gate,
    ));
}

fn move_warrior(
    time: Res<Time>,
    mut warrior_query: Query<Entity, With<Warrior>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut TextureAtlas, &mut Sprite)>,
) {
    for warrior_entity in warrior_query.iter_mut() {
        let mut anim = || {
            let (mut anim_timer, mut texture, mut _sprite) =
                animation_query.get_mut(warrior_entity).unwrap();
            anim_timer.tick(time.delta());
            if anim_timer.just_finished() {
                cycle_texture(&mut texture, 0..=5);
            }
        };
        anim();
    }
}

fn move_gate(
    time: Res<Time>,
    mut gate_query: Query<Entity, With<Gate>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Transform, &mut Sprite)>,
    input: Query<
        &leafwing_input_manager::action_state::ActionState<crate::player::PlayerMovement>,
        With<crate::player::Player>,
    >,
    mut iteration: Local<usize>,
) {
    if *iteration > 1 && *iteration < 23 {
        for gate_entity in gate_query.iter_mut() {
            let mut anim = || {
                let (mut anim_timer, mut transform, mut _sprite) =
                    animation_query.get_mut(gate_entity).unwrap();
                anim_timer.tick(time.delta());
                if anim_timer.just_finished() {
                    transform.translation.y += GATE_HEIGHT / 20.0;
                    *iteration += 1;
                }
            };
            anim();
        }
    }

    let input_state = match input.get_single() {
        Ok(state) => state,
        Err(_) => return,
    };
    if input_state.just_pressed(&crate::player::PlayerMovement::Crouch) {
        *iteration += 1;
    }
}

fn check_enigma(
    mut no_more_msg_event: EventReader<NoMoreStoryMessages>,
    mut enigmas: ResMut<Enigmas>,
    mut params: ResMut<TextSyllableValues>,
) {
}

fn despawn_warrior(mut commands: Commands, warriors: Query<Entity, With<Warrior>>) {
    for warrior in warriors.iter() {
        commands.entity(warrior).despawn_recursive();
    }
}

fn despawn_gate(mut commands: Commands, gates: Query<Entity, With<Gate>>) {
    for gate in gates.iter() {
        commands.entity(gate).despawn_recursive();
    }
}
