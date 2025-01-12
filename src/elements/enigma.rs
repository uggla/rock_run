use std::mem;

use crate::{
    assets::RockRunAssets,
    beasts::squirel::Nuts,
    coregame::{
        colliders::{ColliderName, Story},
        level::{CurrentLevel, Level},
        localization,
        state::AppState,
    },
    elements::{
        moving_platform::MovingPlatformMovement,
        rock::{ROCK_DIAMETER, ROCK_SCALE_FACTOR},
        story::{decompose_selection_msg, TextSyllableValues},
    },
    events::{EnigmaResult, NoMoreStoryMessages},
    helpers::texture::cycle_texture,
    key::{Key, Keys, KEY_HEIGHT, KEY_SCALE_FACTOR, KEY_WIDTH},
};
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    utils::HashMap,
};
use bevy_fluent::{BundleAsset, Locale};
use bevy_rapier2d::{
    dynamics::{Ccd, ExternalImpulse, GravityScale, RigidBody, Velocity},
    prelude::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Group, Sensor},
};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use super::moving_platform::MovingPlatform;

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

#[derive(Debug, PartialEq, Eq, Clone)]
enum Answer {
    Correct,
    Incorrect,
}

#[derive(Component)]
pub struct RockGate {
    associated_story: String,
    impulse: Vec2,
    move_on: Answer,
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
    Mcq(Vec<String>),
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
            .add_systems(
                Update,
                (move_gate, move_rockgate, check_enigma, move_platform)
                    .run_if(not(in_state(AppState::Loading))),
            )
            .add_event::<EnigmaResult>();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_enigma_materials(
    mut commands: Commands,
    locale: Res<Locale>,
    assets: Res<Assets<BundleAsset>>,
    rock_run_assets: Res<RockRunAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    current_level: Res<CurrentLevel>,
    levels: Query<&Level, With<Level>>,
    mut enigmas: ResMut<Enigmas>,
) {
    info!("spawn_enigma_materials");
    let mut rng = thread_rng();

    let mut mcqs = vec![
        ("mammals-question", "mammals", "non-mammals"),
        (
            "fastest-land-animal-question",
            "non-fastest-land-animal",
            "fastest-land-animal",
        ),
        (
            "closest-planet-to-sun-question",
            "non-closest-planet-to-sun",
            "closest-planet-to-sun",
        ),
        (
            "japan-capital-question",
            "non-japan-capital",
            "japan-capital",
        ),
        (
            "who-is-napoleon-question",
            "non-napoleon-answer",
            "napoleon-answer",
        ),
        (
            "largest-ocean-question",
            "non-largest-ocean",
            "largest-ocean",
        ),
        (
            "largest-animal-question",
            "non-largest-animal",
            "largest-animal",
        ),
    ];

    mcqs.shuffle(&mut thread_rng());

    let mut enigmas_builder = Vec::new();

    enigmas_builder.push(Enigma {
        associated_story: "story03-03".to_string(),
        kind: EnigmaKind::Numbers(HashMap::from([
            ("n1".to_string(), rng.gen_range(0..=50).to_string()),
            ("n2".to_string(), rng.gen_range(0..50).to_string()),
        ])),
    });

    enigmas_builder.push(Enigma {
        associated_story: "story04-03".to_string(),
        kind: EnigmaKind::Numbers(HashMap::from([(
            "n1".to_string(),
            rng.gen_range(0..=49).to_string(),
        )])),
    });

    let mcq = mcqs.pop().unwrap();
    enigmas_builder.push(Enigma {
        associated_story: "story05-04".to_string(),
        kind: EnigmaKind::Mcq(vec![
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.0, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.1, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.2, None),
        ]),
    });

    let mcq = mcqs.pop().unwrap();
    enigmas_builder.push(Enigma {
        associated_story: "story06-03".to_string(),
        kind: EnigmaKind::Mcq(vec![
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.0, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.1, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.2, None),
        ]),
    });

    let mcq = mcqs.pop().unwrap();
    enigmas_builder.push(Enigma {
        associated_story: "story07-04".to_string(),
        kind: EnigmaKind::Mcq(vec![
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.0, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.1, None),
            localization::get_translation(&locale, &assets, &rock_run_assets, mcq.2, None),
        ]),
    });

    let mut n1 = rng.gen_range(0..=100);
    let mut n2 = rng.gen_range(0..=100);
    if n2 > n1 {
        mem::swap(&mut n1, &mut n2);
    }

    enigmas_builder.push(Enigma {
        associated_story: "story08-03".to_string(),
        kind: EnigmaKind::Numbers(HashMap::from([
            ("n1".to_string(), n1.to_string()),
            ("n2".to_string(), n2.to_string()),
        ])),
    });

    enigmas_builder.push(Enigma {
        associated_story: "story100-03".to_string(),
        kind: EnigmaKind::Mcq(vec![]),
    });

    enigmas_builder.push(Enigma {
        associated_story: "story101-03".to_string(),
        kind: EnigmaKind::Numbers(HashMap::from([(
            "n1".to_string(),
            (rng.gen_range(0..=49) * 2).to_string(),
        )])),
    });

    *enigmas = Enigmas {
        enigmas: enigmas_builder,
    };

    let level = levels
        .iter()
        .find(|level| level.id == current_level.id)
        .unwrap();

    match current_level.id {
        1 => {
            let texture = rock_run_assets.rock_ball.clone();

            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(ROCK_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(2800.0, 160.0 - ROCK_DIAMETER / 2.0))
                        .extend(20.0),
                    ..default()
                },
                RigidBody::Dynamic,
                GravityScale(20.0),
                Velocity::zero(),
                Collider::ball(ROCK_DIAMETER / 2.0),
                // Group definition so far, might evolves in the future...
                // GROUP_1: rocks (only this one)
                // GROUP_2: beasts (pterodactyls)
                //
                // Note: All colliders not part of a group (without this component) collides by default.
                CollisionGroups::new(Group::GROUP_1, Group::GROUP_1),
                Ccd::enabled(),
                ExternalImpulse::default(),
                RockGate {
                    associated_story: "story04-03".to_string(),
                    impulse: Vec2::new(4096.0 * 120.0, 0.0),
                    move_on: Answer::Correct,
                },
            ));

            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(ROCK_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(7505.0, 208.0 - ROCK_DIAMETER / 2.0))
                        .extend(20.0),
                    ..default()
                },
                RigidBody::Dynamic,
                GravityScale(20.0),
                Velocity::zero(),
                Collider::ball(ROCK_DIAMETER / 2.0),
                Ccd::enabled(),
                ExternalImpulse::default(),
                RockGate {
                    associated_story: "story06-03".to_string(),
                    impulse: Vec2::new(-4096.0 * 120.0, 0.0),
                    move_on: Answer::Incorrect,
                },
            ));

            let texture = rock_run_assets.gate.clone();
            commands.spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(GATE_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(8032.0, 548.0))
                        .extend(3.0),
                    ..default()
                },
                Collider::cuboid(GATE_WIDTH / 2.0, GATE_HEIGHT / 2.0),
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                Gate {
                    associated_story: "story06-03".to_string(),
                },
            ));
        }

        2 => {
            let texture = rock_run_assets.gate.clone();
            commands.spawn((
                Sprite {
                    image: texture,
                    flip_y: true,
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(GATE_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(12144.0, 2135.0))
                        .extend(4.0),
                    ..default()
                },
                Collider::cuboid(GATE_WIDTH / 2.0, GATE_HEIGHT / 2.0),
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                Gate {
                    associated_story: "story08-03".to_string(),
                },
            ));
        }

        3 => {
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
                Sprite {
                    image: texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout,
                        index: 0,
                    }),
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(WARRIOR_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(5575.0, 608.0))
                        .extend(2.0),
                    ..default()
                },
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                Warrior,
            ));

            let texture = rock_run_assets.gate.clone();
            commands.spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    scale: Vec3::splat(GATE_SCALE_FACTOR),
                    translation: level
                        .map
                        .tiled_to_bevy_coord(Vec2::new(5488.0, 615.0))
                        .extend(2.0),
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

fn check_enigma(
    mut commands: Commands,
    rock_run_assets: Res<RockRunAssets>,
    mut no_more_msg_event: EventReader<NoMoreStoryMessages>,
    enigmas: ResMut<Enigmas>,
    params: ResMut<TextSyllableValues>,
    mut enigna_result: EventWriter<EnigmaResult>,
    nuts: Res<Nuts>,
    levels: Query<&Level, With<Level>>,
    current_level: Res<CurrentLevel>,
    collected_keys: Res<Keys>,
    stories_query: Query<(Entity, &ColliderName), With<Story>>,
) {
    for ev in no_more_msg_event.read() {
        debug!("No more story messages: {:?}", ev.latest);

        let level = levels
            .iter()
            .find(|level| level.id == current_level.id)
            .unwrap();

        match ev.latest.as_ref() {
            "story03-03" => {
                let story = "story03-03";
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == story)
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Mcq(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                let n1 = numbers.get("n1").unwrap().parse::<usize>().unwrap();
                let n2 = numbers.get("n2").unwrap().parse::<usize>().unwrap();

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 + n2 == user_answer {
                    debug!("Correct answer: {} + {} = {}", n1, n2, user_answer);
                    correct_answer(
                        &mut enigna_result,
                        story,
                        &mut commands,
                        &rock_run_assets,
                        &stories_query,
                    );
                } else {
                    debug!("Incorrect answer: {} + {} = {}", n1, n2, user_answer);
                    wrong_answer(&mut enigna_result, story, &mut commands, &rock_run_assets);
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
                        EnigmaKind::Mcq(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                let n1 = numbers.get("n1").unwrap().parse::<usize>().unwrap();

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 * 2 == user_answer {
                    debug!("Correct answer: {} * 2 = {}", n1, user_answer);
                    correct_answer(
                        &mut enigna_result,
                        story,
                        &mut commands,
                        &rock_run_assets,
                        &stories_query,
                    );
                } else {
                    debug!("Incorrect answer: {} * 2 = {}", n1, user_answer);
                    wrong_answer(&mut enigna_result, story, &mut commands, &rock_run_assets);
                }
            }
            "story05-04" => {
                check_mcq(
                    "story05-04",
                    &enigmas,
                    &params,
                    &mut enigna_result,
                    &mut commands,
                    &rock_run_assets,
                    level,
                    &stories_query,
                    spawn_story_100,
                );
            }
            "story06-03" => {
                check_mcq(
                    "story06-03",
                    &enigmas,
                    &params,
                    &mut enigna_result,
                    &mut commands,
                    &rock_run_assets,
                    level,
                    &stories_query,
                    |_level, _assets, _commands| {},
                );
            }
            "story07-04" => {
                check_mcq(
                    "story07-04",
                    &enigmas,
                    &params,
                    &mut enigna_result,
                    &mut commands,
                    &rock_run_assets,
                    level,
                    &stories_query,
                    spawn_key,
                );
            }
            "story08-03" => {
                let story = "story08-03";
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == story)
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Mcq(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                let n1 = numbers.get("n1").unwrap().parse::<usize>().unwrap();
                let n2 = numbers.get("n2").unwrap().parse::<usize>().unwrap();

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 - n2 == user_answer && collected_keys.numbers == 1 {
                    debug!(
                        "Correct answer: {} - {} = {} | Collected keys: {}",
                        n1, n2, user_answer, collected_keys.numbers
                    );
                    correct_answer(
                        &mut enigna_result,
                        story,
                        &mut commands,
                        &rock_run_assets,
                        &stories_query,
                    );
                    spawn_story_101(level, &rock_run_assets, &mut commands);
                } else {
                    debug!(
                        "Incorrect answer: {} - {} = {} | Collected keys: {}",
                        n1, n2, user_answer, collected_keys.numbers
                    );
                    wrong_answer(&mut enigna_result, story, &mut commands, &rock_run_assets);
                }
            }
            "story100-03" => {
                let story = "story100-03";
                if nuts.len() == 11 {
                    debug!("Correct answer: {}", nuts.len());
                    correct_answer(
                        &mut enigna_result,
                        story,
                        &mut commands,
                        &rock_run_assets,
                        &stories_query,
                    );
                } else {
                    debug!("Incorrect answer: {}", nuts.len());
                    wrong_answer(&mut enigna_result, story, &mut commands, &rock_run_assets);
                }
            }
            "story101-03" => {
                let story = "story101-03";
                let numbers = enigmas
                    .enigmas
                    .iter()
                    .filter(|e| e.associated_story == story)
                    .map(|e| match e.kind.clone() {
                        EnigmaKind::Numbers(n) => n,
                        EnigmaKind::Mcq(_) => unreachable!(),
                    })
                    .last()
                    .unwrap();

                let n1 = numbers.get("n1").unwrap().parse::<usize>().unwrap();

                let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
                let user_answer = selection.selection_items.join("").parse::<usize>().unwrap();

                if n1 / 2 == user_answer {
                    debug!(
                        "Correct answer: {} / 2 = {} | Collected keys: {}",
                        n1, user_answer, collected_keys.numbers
                    );
                    correct_answer(
                        &mut enigna_result,
                        story,
                        &mut commands,
                        &rock_run_assets,
                        &stories_query,
                    );
                } else {
                    debug!(
                        "Incorrect answer: {} - / 2 = {} | Collected keys: {}",
                        n1, user_answer, collected_keys.numbers
                    );
                    wrong_answer(&mut enigna_result, story, &mut commands, &rock_run_assets);
                }
            }
            _ => {}
        }
    }
}

fn spawn_story_100(level: &Level, _rock_run_assets: &Res<RockRunAssets>, commands: &mut Commands) {
    let Vec2 { x, y } = level.map.tiled_to_bevy_coord(Vec2::new(7250.0, 608.0));
    commands
        .spawn((
            Collider::cuboid(1.0, 1.0),
            Story,
            Transform::from_xyz(x, y, 0.0),
        ))
        .insert(Sensor)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(ColliderName("story100".to_string()));
}

fn spawn_story_101(level: &Level, _rock_run_assets: &Res<RockRunAssets>, commands: &mut Commands) {
    let Vec2 { x, y } = level.map.tiled_to_bevy_coord(Vec2::new(12496.0, 2064.0));
    commands
        .spawn((
            Collider::cuboid(1.0, 1.0),
            Story,
            Transform::from_xyz(x, y, 0.0),
        ))
        .insert(Sensor)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::KINEMATIC_STATIC)
        .insert(ColliderName("story101".to_string()));
}

fn spawn_key(level: &Level, rock_run_assets: &Res<RockRunAssets>, commands: &mut Commands) {
    let start_pos = level.map.tiled_to_bevy_coord(Vec2::new(5984.0, 2064.0));
    commands.spawn((
        Sprite {
            image: rock_run_assets.key.clone(),
            ..default()
        },
        Transform {
            scale: Vec3::splat(KEY_SCALE_FACTOR),
            translation: start_pos.extend(10.0),
            ..default()
        },
        Collider::cuboid(KEY_WIDTH / 2.0, KEY_HEIGHT / 2.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::KINEMATIC_STATIC,
        Key,
        ColliderName("key01".to_string()),
    ));
}

#[allow(clippy::too_many_arguments)]
fn check_mcq<F>(
    story: &str,
    enigmas: &ResMut<Enigmas>,
    params: &ResMut<TextSyllableValues>,
    enigna_result: &mut EventWriter<EnigmaResult>,
    commands: &mut Commands,
    rock_run_assets: &Res<RockRunAssets>,
    level: &Level,
    stories_query: &Query<(Entity, &ColliderName), With<Story>>,
    correct_fn: F,
) where
    F: Fn(&Level, &Res<RockRunAssets>, &mut Commands),
{
    let mcq_values = enigmas
        .enigmas
        .iter()
        .filter(|e| e.associated_story == story)
        .map(|e| match e.kind.clone() {
            EnigmaKind::Numbers(_) => unreachable!(),
            EnigmaKind::Mcq(values) => values,
        })
        .last()
        .unwrap();

    let (_ltext, selection, _rtext) = decompose_selection_msg(&params.text).unwrap();
    let user_answer_nb = selection.get_selected_item();
    let user_answer = selection
        .selection_items
        .get(user_answer_nb)
        .unwrap()
        .trim();

    if mcq_values[2].contains(user_answer) {
        debug!("Correct answer: {}", user_answer);
        correct_fn(level, rock_run_assets, commands);
        correct_answer(
            enigna_result,
            story,
            commands,
            rock_run_assets,
            stories_query,
        );
    } else {
        debug!("Incorrect answer: {}", user_answer);
        wrong_answer(enigna_result, story, commands, rock_run_assets);
    }
}

fn wrong_answer(
    enigna_result: &mut EventWriter<EnigmaResult>,
    story: &str,
    commands: &mut Commands,
    rock_run_assets: &Res<RockRunAssets>,
) {
    enigna_result.send(EnigmaResult::Incorrect(story.to_string()));
    commands.spawn((
        AudioPlayer::new(rock_run_assets.story_wrong_sound.clone()),
        PlaybackSettings {
            volume: Volume::new(7.0),
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));
}

fn correct_answer(
    enigna_result: &mut EventWriter<EnigmaResult>,
    story: &str,
    commands: &mut Commands,
    rock_run_assets: &Res<RockRunAssets>,
    stories_query: &Query<(Entity, &ColliderName), With<Story>>,
) {
    enigna_result.send(EnigmaResult::Correct(story.to_string()));
    commands.spawn((
        AudioPlayer::new(rock_run_assets.story_valid_sound.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));

    for (entity, collider_name) in stories_query.iter() {
        if story.contains(&collider_name.0) {
            debug!("Despawn story: {}", collider_name.0);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn move_warrior(
    time: Res<Time>,
    mut warrior_query: Query<Entity, With<Warrior>>,
    mut animation_query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for warrior_entity in warrior_query.iter_mut() {
        let mut anim = || {
            let (mut anim_timer, mut sprite) = animation_query.get_mut(warrior_entity).unwrap();
            anim_timer.tick(time.delta());
            if anim_timer.just_finished() {
                if let Some(texture) = &mut sprite.texture_atlas {
                    cycle_texture(texture, 0..=5);
                }
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
                        "Opening gate {:?} associated to {:?}",
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
    mut rockgate_query: Query<(Entity, &RockGate), With<RockGate>>,
    mut enigna_result: EventReader<EnigmaResult>,
    mut ext_impulses: Query<&mut ExternalImpulse, With<RockGate>>,
) {
    for ev in enigna_result.read() {
        match ev {
            EnigmaResult::Correct(enigma) => {
                do_movement(
                    &mut rockgate_query,
                    enigma,
                    &mut ext_impulses,
                    Answer::Correct,
                );
            }
            EnigmaResult::Incorrect(enigma) => {
                do_movement(
                    &mut rockgate_query,
                    enigma,
                    &mut ext_impulses,
                    Answer::Incorrect,
                );
            }
        }
    }

    fn do_movement(
        rockgate_query: &mut Query<(Entity, &RockGate), With<RockGate>>,
        enigma: &String,
        ext_impulses: &mut Query<&mut ExternalImpulse, With<RockGate>>,
        act_on: Answer,
    ) {
        for (rockgate_entity, current_rockgate) in rockgate_query.iter_mut() {
            if current_rockgate.associated_story == *enigma && current_rockgate.move_on == act_on {
                debug!(
                    "Moving rockgate {:?} associated to {:?}",
                    rockgate_entity, current_rockgate.associated_story
                );

                let mut ext_impulse = ext_impulses.get_mut(rockgate_entity).unwrap();
                ext_impulse.impulse = current_rockgate.impulse;
            }
        }
    }
}

fn move_platform(
    mut moving_platform_query: Query<&mut MovingPlatform>,
    mut enigna_result: EventReader<EnigmaResult>,
) {
    for ev in enigna_result.read() {
        if let EnigmaResult::Correct(enigma) = ev {
            if enigma == "story05-04" {
                for mut mvp in moving_platform_query.iter_mut() {
                    if let MovingPlatformMovement::UpDown(ref mut data) = mvp.movement {
                        if data.speed == 0.0 {
                            data.speed = 2.0;
                        }
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
