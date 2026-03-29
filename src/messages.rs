use bevy::{platform::collections::HashMap, prelude::*};

use crate::elements::story::SelectionDirection;

pub type MessageArgs = Option<HashMap<String, String>>;
pub type Message = String;

#[derive(Message)]
pub enum StoryMessages {
    Display(Vec<(Message, MessageArgs)>),
    Hide,
    Next,
}

#[derive(Message)]
pub struct NoMoreStoryMessages {
    pub latest: Message,
}

#[derive(Message)]
pub struct Hit;

#[derive(Message)]
pub struct StartGame;

#[derive(Message)]
pub struct Restart;

#[derive(Message)]
pub struct NextLevel;

#[derive(Message)]
pub struct ShakeCamera;

#[derive(Message)]
pub enum LifeEvent {
    Win,
    Lost,
}

#[derive(Message)]
pub struct TriceratopsCollision {
    pub id: Entity,
}

#[derive(Message, Debug)]
pub struct PositionSensorCollisionStart {
    pub sensor_name: String,
    pub spawn_pos: Vec2,
    pub exit_pos: Vec2,
}

#[derive(Message, Debug)]
pub struct PositionSensorCollisionStop {
    pub sensor_name: String,
}

#[derive(Message)]
pub struct LadderCollisionStart;

#[derive(Message)]
pub struct LadderCollisionStop;

#[derive(Message)]
pub struct MovingPlatformCollision {
    pub entity: Entity,
}

#[derive(Message)]
pub struct MovingPlatformDescending {
    pub movement: Vec2,
}

#[derive(Message)]
pub struct SelectionChanged {
    pub movement: SelectionDirection,
}

#[allow(dead_code)]
#[derive(Message, Debug)]
pub enum EnigmaResult {
    Correct(String),
    Incorrect(String),
}

#[derive(Message)]
pub struct ExtraLifeCollision {
    pub entity: Entity,
}

#[derive(Message)]
pub struct NutCollision {
    pub entity: Entity,
}

#[derive(Message)]
pub struct KeyCollision {
    pub entity: Entity,
}

#[derive(Message)]
pub struct SmallRockAboutToRelease;
