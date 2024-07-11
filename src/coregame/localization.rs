use crate::{
    elements::story::TextSyllableValues,
    events::{Message, MessageArgs, NoMoreStoryMessages, StoryMessages},
};
use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use bevy_fluent::{BundleAsset, Locale};
use fluent::{FluentArgs, FluentValue};
use unic_langid::langid;

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Locale::new(langid!("fr-FR")))
            .insert_resource(LocaleHandles::default())
            .add_systems(Startup, load_localization)
            .add_systems(Update, (check_localization_loaded, localize_story_messages));
    }
}

#[derive(Resource, Default)]
pub struct LocaleHandles {
    handles: Vec<Handle<BundleAsset>>,
}

fn load_localization(asset_server: Res<AssetServer>, mut locale_handles: ResMut<LocaleHandles>) {
    info!("load_localisation");
    let locale_files = ["locales/en-US/main.ftl.ron", "locales/fr-FR/main.ftl.ron"];

    for file in locale_files {
        locale_handles.handles.push(asset_server.load(file));
    }
}

fn check_localization_loaded(
    asset_server: Res<AssetServer>,
    locale_handles: Res<LocaleHandles>,
    mut already_run: Local<bool>,
) {
    if *already_run {
        return;
    }
    info!("check_localization_loadded");
    let loading_status = locale_handles
        .handles
        .iter()
        .map(|handle| asset_server.get_load_state(handle))
        .collect::<Vec<_>>();

    if loading_status
        .iter()
        .all(|status| matches!(status, Some(LoadState::Loaded)))
    {
        info!("All localisation loaded");
        *already_run = true;
    }
}

pub fn get_translation(
    locale: &Locale,
    assets: &Res<Assets<BundleAsset>>,
    locale_handles: &Res<LocaleHandles>,
    message: &str,
    args: Option<&FluentArgs>,
) -> String {
    let handle = if locale.requested == langid!("fr-FR") {
        debug!("Lang: fr-FR");
        &locale_handles.handles[1]
    } else {
        debug!("Lang: en-US");
        &locale_handles.handles[0]
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
    locale_handles: Res<LocaleHandles>,
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
                    &locale_handles,
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
