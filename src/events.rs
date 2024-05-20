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
