use crate::{
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    events::{EnigmaResult, NoMoreStoryMessages},
    helpers::texture::cycle_texture,
    text_syllable::{decompose_selection_msg, TextSyllableValues},
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
pub struct Gate {
    associated_story: String,
}

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
                associated_story: "story03-01".to_string(),
                kind: EnigmaKind::Numbers(HashMap::from([
                    ("n1".to_string(), rng.gen_range(0..=50).to_string()),
                    ("n2".to_string(), rng.gen_range(0..50).to_string()),
                ])),
            }],
        };
        app.insert_resource(enigmas)
            .add_systems(OnEnter(AppState::GameCreate), spawn_enigma_materials)
            .add_systems(OnEnter(AppState::NextLevel), spawn_enigma_materials)
            .add_systems(
                OnEnter(AppState::StartMenu),
                (despawn_warrior, despawn_gate),
            )
            .add_systems(
                OnEnter(AppState::FinishLevel),
                (despawn_warrior, despawn_gate),
            )
            .add_systems(
                Update,
                (move_warrior, move_gate).run_if(in_state(AppState::GameRunning)),
            )
            .add_systems(Update, check_enigma)
            .add_event::<EnigmaResult>();
    }
}

fn spawn_enigma_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
) {
    info!("spawn_enigma_materials");

    if current_level.id == 2 {
        let level = levels
            .iter()
            .find(|level| level.id == current_level.id)
            .unwrap();

        let texture = asset_server.load("warrior.png");
        let layout = TextureAtlasLayout::from_grid(
            Vec2::new(WARRIOR_WIDTH, WARRIOR_HEIGHT),
            6,
            1,
            None,
            None,
        );
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
            Gate {
                associated_story: "story03-01".to_string(),
            },
        ));
    }
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
    mut gate_query: Query<(Entity, &Gate), With<Gate>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Transform, &mut Sprite)>,
    mut enigna_result: EventReader<EnigmaResult>,
    mut iteration: Local<usize>,
    mut gate: Local<Option<Entity>>,
) {
    let mut anim = |gate_entity| -> bool {
        let (mut anim_timer, mut transform, mut _sprite) =
            animation_query.get_mut(gate_entity).unwrap();
        anim_timer.tick(time.delta());
        if anim_timer.just_finished() {
            if *iteration < 23 {
                transform.translation.y += GATE_HEIGHT / 20.0;
                *iteration += 1;
                return false;
            } else {
                return true;
            }
        }
        false
    };

    for ev in enigna_result.read() {
        if let EnigmaResult::Correct(enigma) = ev {
            for (gate_entity, current_gate) in gate_query.iter_mut() {
                if current_gate.associated_story == *enigma {
                    debug!(
                        "Opening gate {:?} associted to \"{:?}\"",
                        gate_entity, current_gate.associated_story
                    );
                    *gate = Some(gate_entity);
                }
            }
        }
    }

    if gate.is_some() && anim(gate.unwrap()) {
        *gate = None;
        *iteration = 0;
    }
}

#[allow(clippy::single_match)]
fn check_enigma(
    mut no_more_msg_event: EventReader<NoMoreStoryMessages>,
    enigmas: ResMut<Enigmas>,
    params: ResMut<TextSyllableValues>,
    mut enigna_result: EventWriter<EnigmaResult>,
) {
    for ev in no_more_msg_event.read() {
        debug!("No more story messages: {:?}", ev.latest);
        match ev.latest.as_ref() {
            "story03-01" => {
                let story = "story03-01";
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == story)
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Qcm(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                let n1 = numbers.get("n1").unwrap().parse::<usize>().unwrap();
                let n2 = numbers.get("n2").unwrap().parse::<usize>().unwrap();

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 + n2 == user_answer {
                    debug!("Correct answer: {} + {} = {}", n1, n2, user_answer);
                    enigna_result.send(EnigmaResult::Correct(story.to_string()));
                } else {
                    debug!("Incorrect answer: {} + {} = {}", n1, n2, user_answer);
                    enigna_result.send(EnigmaResult::Incorrect(story.to_string()));
                }
            }
            _ => {}
        }
    }
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
