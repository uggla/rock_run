use bevy::{prelude::*, utils::HashMap};

pub type MessageArgs = Option<HashMap<String, String>>;
pub type Message = String;

#[derive(Event)]
pub enum StoryMessages {
    Display(Vec<(Message, MessageArgs)>),
    Hide,
    Next,
}

#[derive(Event)]
pub struct NoMoreStoryMessages;

#[derive(Event)]
pub struct Hit;

#[derive(Event)]
pub struct Restart;

// TODO: remove dead code
#[allow(dead_code)]
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
pub struct PositionSensorCollision {
    pub sensor_name: String,
    pub spawn_pos: Vec2,
    pub exit_pos: Vec2,
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
