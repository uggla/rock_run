use crate::coregame::state::AppState;
use bevy::{prelude::*, utils::HashMap};
use rand::{thread_rng, Rng};

#[derive(Resource, Debug)]
pub struct Enigmas {
    pub enigmas: Vec<Enigma>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Enigma {
    pub associated_story: String,
    pub kind: EnigmaKind,
}

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
        app.insert_resource(enigmas).add_systems(
            Update,
            (check_enigma).run_if(in_state(AppState::GameRunning)),
        );
    }
}

fn check_enigma() {}
