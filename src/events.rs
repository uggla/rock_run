use bevy::{prelude::*, utils::HashMap};

use crate::elements::story::SelectionDirection;

pub type MessageArgs = Option<HashMap<String, String>>;
pub type Message = String;

#[derive(Event)]
pub enum StoryMessages {
    Display(Vec<(Message, MessageArgs)>),
    Hide,
    Next,
}

#[derive(Event)]
pub struct NoMoreStoryMessages {
    pub latest: Message,
}

#[derive(Event)]
pub struct Hit;

#[derive(Event)]
pub struct StartGame;

#[derive(Event)]
pub struct Restart;

#[derive(Event)]
pub struct NextLevel;

#[derive(Event)]
pub enum LifeEvent {
    Win,
    Lost,
}

#[derive(Event)]
pub struct TriceratopsCollision {
    pub id: Entity,
}

#[derive(Event, Debug)]
pub struct PositionSensorCollisionStart {
    pub sensor_name: String,
    pub spawn_pos: Vec2,
    pub exit_pos: Vec2,
}

#[derive(Event, Debug)]
pub struct PositionSensorCollisionStop {
    pub sensor_name: String,
}

#[derive(Event)]
pub struct LadderCollisionStart;

#[derive(Event)]
pub struct LadderCollisionStop;

#[derive(Event)]
pub struct MovingPlatformCollision {
    pub entity: Entity,
}

#[derive(Event)]
pub struct MovingPlatformDescending {
    pub movement: Vec2,
}

#[derive(Event)]
pub struct SelectionChanged {
    pub movement: SelectionDirection,
}

#[allow(dead_code)]
#[derive(Event, Debug)]
pub enum EnigmaResult {
    Correct(String),
    Incorrect(String),
}

#[derive(Event)]
pub struct ExtraLifeCollision {
    pub entity: Entity,
}

#[derive(Event)]
pub struct NutCollision {
    pub entity: Entity,
}
