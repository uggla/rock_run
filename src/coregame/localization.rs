use crate::{
    assets::RockRunAssets,
    coregame::state::AppState,
    elements::story::TextSyllableValues,
    events::{Message, MessageArgs, NoMoreStoryMessages, StoryMessages},
};
use bevy::{prelude::*, utils::HashMap};
use bevy_fluent::{BundleAsset, Locale};
use fluent::{FluentArgs, FluentValue};
use unic_langid::langid;

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Locale::new(langid!("en-US")))
            .add_systems(
                Update,
                localize_story_messages.run_if(not(in_state(AppState::Loading))),
            );
    }
}

pub fn get_translation(
    locale: &Locale,
    assets: &Res<Assets<BundleAsset>>,
    rock_run_assets: &Res<RockRunAssets>,
    message: &str,
    args: Option<&FluentArgs>,
) -> String {
    let handle = if locale.requested == langid!("fr-FR") {
        debug!("Lang: fr-FR");
        &rock_run_assets.french_locale
    } else {
        debug!("Lang: en-US");
        &rock_run_assets.english_locale
    };

    let bundle = match assets.get(handle) {
        Some(bundle) => bundle,
        None => panic!("Localization bundle cannot be found or not loaded."),
    };
    let msg = bundle
        .get_message(message)
        .expect("Message translation doesn't exist.");
    let mut errors = vec![];
    let pattern = msg.value().expect("Message has no value.");
    let value = bundle.format_pattern(pattern, args, &mut errors);
    value.to_string().replace(['\u{2068}', '\u{2069}'], "")
}

#[allow(clippy::too_many_arguments)]
fn localize_story_messages(
    mut msg_event_reader: EventReader<StoryMessages>,
    mut msg_event_writer: EventWriter<NoMoreStoryMessages>,
    assets: Res<Assets<BundleAsset>>,
    locale: Res<Locale>,
    rock_run_assets: Res<RockRunAssets>,
    mut params: ResMut<TextSyllableValues>,
    mut messages: Local<Vec<(Message, MessageArgs)>>,
    mut latest_message: Local<Message>,
) {
    for ev in msg_event_reader.read() {
        match ev {
            StoryMessages::Hide => {
                break;
            }
            StoryMessages::Display(ev) => {
                messages.clone_from(ev);
                messages.reverse();
            }
            StoryMessages::Next => {}
        }

        match messages.pop() {
            Some((msg, args)) => {
                latest_message.clone_from(&msg);
                let translation_args = convert_to_fluent_args(args);

                let value = get_translation(
                    &locale,
                    &assets,
                    &rock_run_assets,
                    &msg,
                    translation_args.as_ref(),
                );
                params.text = value;
            }
            None => {
                // No more messages, so close the msg box
                debug!("no more message");
                msg_event_writer.send(NoMoreStoryMessages {
                    latest: latest_message.to_string(),
                });
            }
        }
    }
}

pub fn convert_to_fluent_args(
    args: Option<HashMap<String, String>>,
) -> Option<FluentArgs<'static>> {
    match args {
        Some(args) => {
            let mut fluent_args = FluentArgs::new();
            for (key, value) in args {
                fluent_args.set(key, FluentValue::from(value));
            }
            Some(fluent_args)
        }
        None => None,
    }
}
