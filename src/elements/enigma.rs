use crate::{
    assets::RockRunAssets,
    coregame::{
        level::{CurrentLevel, Level},
        state::AppState,
    },
    elements::{
        rock::{ROCK_DIAMETER, ROCK_SCALE_FACTOR},
        story::{decompose_selection_msg, TextSyllableValues},
    },
    events::{EnigmaResult, NoMoreStoryMessages},
    helpers::texture::cycle_texture,
};
use bevy::{audio::PlaybackMode, prelude::*, utils::HashMap};
use bevy_rapier2d::{
    dynamics::{Ccd, ExternalImpulse, GravityScale, RigidBody, Velocity},
    geometry::{ActiveCollisionTypes, Collider},
};
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

#[derive(Component)]
pub struct RockGate {
    associated_story: String,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource, Debug, Default)]
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
        app.insert_resource(Enigmas::default())
            .add_systems(OnEnter(AppState::GameCreate), spawn_enigma_materials)
            .add_systems(OnEnter(AppState::NextLevel), spawn_enigma_materials)
            .add_systems(
                OnEnter(AppState::StartMenu),
                (despawn_warrior, despawn_gate, despawn_rockgate),
            )
            .add_systems(
                OnEnter(AppState::FinishLevel),
                (despawn_warrior, despawn_gate, despawn_rockgate),
            )
            .add_systems(Update, move_warrior.run_if(in_state(AppState::GameRunning)))
            // .add_systems(
            //     Update,
            //     (move_gate, move_rockgate).run_if(
            //         in_state(AppState::GameRunning).or_else(in_state(AppState::GameMessage)),
            //     ),
            // )
            .add_systems(
                Update,
                (move_gate, move_rockgate, check_enigma).run_if(not(in_state(AppState::Loading))),
            )
            .add_event::<EnigmaResult>();
    }
}

fn spawn_enigma_materials(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
    mut enigmas: ResMut<Enigmas>,
) {
    info!("spawn_enigma_materials");
    let mut rng = thread_rng();

    *enigmas = Enigmas {
        enigmas: vec![
            Enigma {
                associated_story: "story03-03".to_string(),
                kind: EnigmaKind::Numbers(HashMap::from([
                    ("n1".to_string(), rng.gen_range(0..=50).to_string()),
                    ("n2".to_string(), rng.gen_range(0..50).to_string()),
                ])),
            },
            Enigma {
                associated_story: "story04-03".to_string(),
                kind: EnigmaKind::Numbers(HashMap::from([(
                    "n1".to_string(),
                    rng.gen_range(0..=49).to_string(),
                )])),
            },
        ],
    };

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    match current_level.id {
        1 => {
            let texture = rock_run_assets.rock_ball.clone();

            commands.spawn((
                SpriteBundle {
                    texture,
                    sprite: Sprite { ..default() },
                    transform: Transform {
                        scale: Vec3::splat(ROCK_SCALE_FACTOR),
                        translation: level
                            .map
                            .tiled_to_bevy_coord(Vec2::new(2800.0, 160.0 - ROCK_DIAMETER / 2.0))
                            .extend(20.0),
                        ..default()
                    },
                    ..default()
                },
                RigidBody::Dynamic,
                GravityScale(20.0),
                Velocity::zero(),
                Collider::ball(ROCK_DIAMETER / 2.0),
                ActiveCollisionTypes::DYNAMIC_KINEMATIC | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                Ccd::enabled(),
                ExternalImpulse::default(),
                RockGate {
                    associated_story: "story04-03".to_string(),
                },
            ));
        }

        2 => {
            let texture = rock_run_assets.warrior.clone();
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(WARRIOR_WIDTH as u32, WARRIOR_HEIGHT as u32),
                6,
                1,
                None,
                None,
            );
            let texture_atlas_layout = texture_atlases.add(layout);

            commands.spawn((
                SpriteBundle {
                    texture,
                    sprite: Sprite { ..default() },
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
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                Warrior,
            ));

            let texture = rock_run_assets.gate.clone();
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
                    associated_story: "story03-03".to_string(),
                },
            ));
        }
        _ => (),
    }
}

#[allow(clippy::single_match)]
fn check_enigma(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut no_more_msg_event: EventReader<NoMoreStoryMessages>,
    enigmas: ResMut<Enigmas>,
    params: ResMut<TextSyllableValues>,
    mut enigna_result: EventWriter<EnigmaResult>,
) {
    for ev in no_more_msg_event.read() {
        debug!("No more story messages: {:?}", ev.latest);
        match ev.latest.as_ref() {
            "story03-03" => {
                let story = "story03-03";
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
            "story04-03" => {
                let story = "story04-03";
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

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 * 2 == user_answer {
                    debug!("Correct answer: {} * 2 = {}", n1, user_answer);
                    enigna_result.send(EnigmaResult::Correct(story.to_string()));
                } else {
                    debug!("Correct answer: {} * 2 = {}", n1, user_answer);
                    enigna_result.send(EnigmaResult::Incorrect(story.to_string()));
                }
            }
            _ => {}
        }

        commands.spawn(AudioBundle {
            source: rock_run_assets.story_valid_sound.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
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
                        "Opening gate {:?} associted to {:?}",
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

fn move_rockgate(
    mut gate_query: Query<(Entity, &RockGate), With<RockGate>>,
    mut enigna_result: EventReader<EnigmaResult>,
    mut ext_impulses: Query<&mut ExternalImpulse, With<RockGate>>,
) {
    for ev in enigna_result.read() {
        if let EnigmaResult::Correct(enigma) = ev {
            debug!("{:?}", enigma);
            for (gate_entity, current_gate) in gate_query.iter_mut() {
                if current_gate.associated_story == *enigma {
                    debug!(
                        "Moving rockgate {:?} associted to {:?}",
                        gate_entity, current_gate.associated_story
                    );

                    for mut ext_impulse in ext_impulses.iter_mut() {
                        ext_impulse.impulse = Vec2::new(4096.0 * 120.0, 0.0);
                    }
                }
            }
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

fn despawn_rockgate(mut commands: Commands, rockgates: Query<Entity, With<RockGate>>) {
    for rockgate in rockgates.iter() {
        commands.entity(rockgate).despawn_recursive();
    }
}
