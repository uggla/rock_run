use std::f32::consts::PI;

use bevy::{
    asset::AssetPath,
    audio::PlaybackMode,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use raqote::{DrawOptions, DrawTarget, Gradient, GradientStop, PathBuilder, Point, Source, Spread};
use serde::{Deserialize, Serialize};

use crate::{
    assets::RockRunAssets,
    coregame::state::AppState,
    events::{SelectionChanged, StoryMessages},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TextSyllableState {
    #[default]
    Hidden,
    Visible,
}

#[derive(Clone, Debug, Default)]
pub struct SyllableStyle {
    pub font_size: f32,
    pub color: Color,
}

pub struct StoryPlugin {
    pub font: AssetPath<'static>,
    pub radius: f32,
    pub box_position: Vec3,
    pub box_size: Vec2,
    pub box_filling: Source<'static>,
    pub style_a: SyllableStyle,
    pub style_b: SyllableStyle,
    pub style_selected: SyllableStyle,
    pub text: String,
}

#[derive(Component)]
struct TextSyllableBox;

#[derive(Component)]
struct RootText;

#[derive(Component)]
struct TextSyllable;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSelection {
    pub selection_items: Vec<String>,
    selected_item: usize,
}

impl UserSelection {
    pub fn new(selections: Vec<String>) -> Self {
        Self {
            selection_items: selections,
            selected_item: 0,
        }
    }

    pub fn get_selected_item(&self) -> usize {
        self.selected_item
    }
}

pub enum SelectionDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Default)]
struct TextStyle {
    color: TextColor,
    font: TextFont,
}

#[derive(Resource)]
pub struct TextSyllableValues {
    font: AssetPath<'static>,
    radius: f32,
    box_position: Vec3,
    box_size: Vec2,
    box_filling: Source<'static>,
    def_style_a: SyllableStyle,
    def_style_b: SyllableStyle,
    def_style_selected: SyllableStyle,
    pub text: String,
    style_a: TextStyle,
    style_b: TextStyle,
    style_selected: TextStyle,
}

impl Default for StoryPlugin {
    fn default() -> Self {
        let box_size = Vec2::new(600.0, 250.0);

        Self {
            font: "fonts/FiraSans-Bold.ttf".into(),
            radius: 20.0,
            box_position: Vec3::new(0.0, 0.0, 0.0),
            box_size,
            box_filling: Source::new_linear_gradient(
                Gradient {
                    stops: vec![
                        GradientStop {
                            position: 0.0,
                            color: raqote::Color::new(0xff, 0x9e, 0x36, 0x13),
                        },
                        GradientStop {
                            position: 0.08,
                            // raqote color is ARGB
                            color: raqote::Color::new(0xff, 0xfe, 0xdf, 0x9c),
                        },
                        GradientStop {
                            position: 0.92,
                            color: raqote::Color::new(0xff, 0xfe, 0xdf, 0x9c),
                        },
                        GradientStop {
                            position: 1.0,
                            color: raqote::Color::new(0xff, 0x9e, 0x36, 0x13),
                        },
                    ],
                },
                Point::new(0., 0.),
                Point::new(0., box_size.y),
                Spread::Pad,
            ),
            // Define the color and fill style
            // box_filling: Source::Solid(SolidSource {
            //     r: 255,
            //     g: 0,
            //     b: 0,
            //     a: 255,
            // }),
            style_a: SyllableStyle {
                font_size: 42.0,
                color: Color::srgb_u8(0xF4, 0x78, 0x04),
            },
            style_b: SyllableStyle {
                font_size: 42.0,
                color: Color::srgb_u8(0x54, 0x2E, 0x0A),
            },
            style_selected: SyllableStyle {
                font_size: 42.0,
                color: Color::srgb_u8(0xD3, 0xCD, 0x39),
            },
            text: "Hel-lo I am Rose, help me re-turn home.".into(),
        }
    }
}

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        let text_params = TextSyllableValues {
            font: self.font.clone(),
            radius: self.radius,
            box_position: self.box_position,
            box_size: self.box_size,
            box_filling: self.box_filling.clone(),
            def_style_a: self.style_a.clone(),
            def_style_b: self.style_b.clone(),
            def_style_selected: self.style_selected.clone(),
            text: self.text.clone(),
            style_a: TextStyle::default(),
            style_b: TextStyle::default(),
            style_selected: TextStyle::default(),
        };
        app.init_state::<TextSyllableState>()
            .insert_resource(text_params)
            .add_systems(OnEnter(AppState::Loading), setup)
            .add_systems(Update, (toggle_visibility, display_or_hide_messages))
            .add_systems(
                Update,
                manage_selection.run_if(in_state(AppState::GameMessage)),
            )
            .add_event::<SelectionChanged>();
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<Image>>,
    mut params: ResMut<TextSyllableValues>,
) {
    let font = asset_server.load(params.font.clone());

    let dt = draw_rounded_rectangle(&params);

    let image = Image::new(
        Extent3d {
            width: params.box_size.x as u32,
            height: params.box_size.y as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        argb_to_rgba(dt.get_data()),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = assets.add(image);

    // Define styles
    params.style_a = TextStyle {
        font: TextFont {
            font: font.clone(),
            font_size: params.def_style_a.font_size,
            ..default()
        },
        color: TextColor(params.def_style_a.color),
    };
    params.style_b = TextStyle {
        font: TextFont {
            font: font.clone(),
            font_size: params.def_style_b.font_size,
            ..default()
        },
        color: TextColor(params.def_style_b.color),
    };
    params.style_selected = TextStyle {
        font: TextFont {
            font: font.clone(),
            font_size: params.def_style_selected.font_size,
            ..default()
        },
        color: TextColor(params.def_style_selected.color),
    };

    let screen_node = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            TextSyllableBox,
        ))
        .id();

    let ui_node = commands
        .spawn((
            // TODO: Add margin based on os cfg.
            // Or based on GPU detection
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(params.box_position.x + WINDOW_WIDTH / 2.0 - params.box_size.x / 2.0),
                // 0 + 720 / 2 - 250 /2 = 360-125        = 235
                top: Val::Px(
                    params.box_position.y + WINDOW_HEIGHT / 2.0
                        - params.box_size.y / 2.0
                        - 35.0 / 2.0,
                ),
                width: Val::Px(params.box_size.x),
                height: Val::Px(params.box_size.y),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ImageNode::new(image_handle),
        ))
        .id();

    let root_text = commands
        .spawn((
            Text::new(""),
            TextLayout {
                justify: JustifyText::Left,
                ..default()
            },
            RootText,
        ))
        .id();

    commands.entity(screen_node).add_child(ui_node);
    commands.entity(ui_node).add_child(root_text);

    display_text_sections(commands, params, root_text);
}

fn display_text_sections(
    mut commands: Commands,
    params: ResMut<TextSyllableValues>,
    root_text: Entity,
) {
    for textspan in build_text_sections(
        &params.text,
        params.style_a.clone(),
        params.style_b.clone(),
        params.style_selected.clone(),
    ) {
        let text_child = commands
            .spawn((textspan.0, textspan.1.color, textspan.1.font, TextSyllable))
            .id();
        commands.entity(root_text).add_child(text_child);
    }
}

fn draw_rounded_rectangle(params: &ResMut<TextSyllableValues>) -> DrawTarget {
    let mut dt = DrawTarget::new(params.box_size.x as i32, params.box_size.y as i32);

    let path = shape_rounded_rectangle(params.radius, params.box_size);

    dt.fill(&path, &params.box_filling, &DrawOptions::new());
    dt
}

fn shape_rounded_rectangle(radius: f32, box_size: Vec2) -> raqote::Path {
    let mut pb = PathBuilder::new();
    //
    // Top right corner
    pb.move_to(radius, 0.0);
    pb.line_to(box_size.x - radius, 0.);
    pb.arc(
        box_size.x - radius,
        radius,
        radius,
        3.0 * PI / 2.0,
        PI / 2.0,
    );
    pb.line_to(box_size.x, box_size.y - radius);
    pb.arc(
        box_size.x - radius,
        box_size.y - radius,
        radius,
        0.0,
        PI / 2.0,
    );
    pb.line_to(radius, box_size.y);
    pb.arc(radius, box_size.y - radius, radius, PI / 2.0, PI / 2.0);
    pb.line_to(0.0, radius);
    pb.arc(radius, radius, radius, PI, PI / 2.0);
    pb.close();
    pb.finish()
}

fn argb_to_rgba(argb_slice: &[u32]) -> Vec<u8> {
    let mut rgba_vec = Vec::with_capacity(argb_slice.len() * 4);

    for &argb in argb_slice {
        let a = ((argb >> 24) & 0xff) as u8; // Extract alpha component
        let r = ((argb >> 16) & 0xff) as u8; // Extract red component
        let g = ((argb >> 8) & 0xff) as u8; // Extract green component
        let b = (argb & 0xff) as u8; // Extract blue component

        // Reorder to RGBA and push to the vector
        rgba_vec.extend_from_slice(&[r, g, b, a]);
    }

    rgba_vec
}

fn build_text_sections_according_to_syllables(
    text: &str,
    style_a: TextStyle,
    style_b: TextStyle,
) -> Vec<(TextSpan, TextStyle)> {
    let mut toggle_style = true;
    let syllables = text.replace("\\-", "####");
    let syllables = syllables.split_whitespace().collect::<Vec<_>>();
    let syllables = syllables
        .iter()
        .map(|s| {
            s.split('-')
                .map(|s| s.replace("####", "-"))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    let text_sections: Vec<(TextSpan, TextStyle)> = syllables
        .iter()
        .map(|s| {
            s.iter()
                .map(|o| {
                    if toggle_style {
                        toggle_style = false;
                        (TextSpan::new(o.to_string()), style_a.clone())
                    } else {
                        toggle_style = true;
                        (TextSpan::new(o.to_string()), style_b.clone())
                    }
                })
                .collect::<Vec<(TextSpan, TextStyle)>>()
        })
        .enumerate()
        .flat_map(|(i, mut s)| {
            if i < syllables.len() - 1 {
                s.extend(vec![(TextSpan::new(" ".to_string()), style_a.clone())]);
                s
            } else {
                s
            }
        })
        .collect();

    text_sections
}

pub fn decompose_selection_msg(text: &str) -> Option<(String, UserSelection, String)> {
    match text.split_once("\\(") {
        Some((ltext, selection)) => {
            let (selection, rtext) = match selection.split_once("\\)") {
                Some((selection, rtext)) => (selection, rtext),
                None => panic!("selection has no closing tag"),
            };

            let selection = format!("{{{}}}", selection);
            let selection: UserSelection = serde_json::from_str(&selection).unwrap();
            Some((ltext.to_string(), selection, rtext.to_string()))
        }
        None => None,
    }
}

pub fn compose_selection_msg(ltext: &str, selection: UserSelection, rtext: &str) -> String {
    let selection = serde_json::to_string(&selection).unwrap();
    let selection = selection.replace('{', "\\(").replace('}', "\\)");
    format!("{}{}{}", ltext, selection, rtext)
}

fn build_text_sections(
    text: &str,
    style_a: TextStyle,
    style_b: TextStyle,
    style_selected: TextStyle,
) -> Vec<(TextSpan, TextStyle)> {
    let text_sections = match decompose_selection_msg(text) {
        Some((ltext, selection, rtext)) => {
            let selection = selection
                .selection_items
                .iter()
                .enumerate()
                .map(|item| {
                    if item.0 == selection.selected_item {
                        (TextSpan::new(item.1.to_string()), style_selected.clone())
                    } else {
                        (TextSpan::new(item.1.to_string()), style_a.clone())
                    }
                })
                .collect::<Vec<(TextSpan, TextStyle)>>();

            let ltext = build_text_sections_according_to_syllables(
                &ltext,
                style_a.clone(),
                style_b.clone(),
            );
            let rtext = build_text_sections_according_to_syllables(
                &rtext,
                style_a.clone(),
                style_b.clone(),
            );

            ltext
                .into_iter()
                .chain(selection)
                .chain(rtext)
                .collect::<Vec<(TextSpan, TextStyle)>>()
        }
        None => build_text_sections_according_to_syllables(text, style_a, style_b),
    };
    text_sections
}

fn toggle_visibility(
    mut commands: Commands,
    mut text_syllable_box: Query<&mut Visibility, With<TextSyllableBox>>,
    root_text: Query<Entity, With<RootText>>,
    text_syllable: Query<Entity, With<TextSyllable>>,
    params: ResMut<TextSyllableValues>,
    visible_state: Res<State<TextSyllableState>>,
) {
    if visible_state.get() == &TextSyllableState::Visible {
        if let Ok(mut visibility) = text_syllable_box.get_single_mut() {
            *visibility = Visibility::Visible;

            if let Ok(root_text_entity) = root_text.get_single() {
                for text_child in text_syllable.iter() {
                    commands.entity(text_child).despawn();
                }

                display_text_sections(commands, params, root_text_entity);
            }
        }
    } else if let Ok(mut visibility) = text_syllable_box.get_single_mut() {
        *visibility = Visibility::Hidden;
    }
}

fn display_or_hide_messages(
    mut msg_event: EventReader<StoryMessages>,
    mut next_state: ResMut<NextState<TextSyllableState>>,
) {
    for ev in msg_event.read() {
        match ev {
            StoryMessages::Display(_msgs) => {
                next_state.set(TextSyllableState::Visible);
            }
            StoryMessages::Hide => {
                next_state.set(TextSyllableState::Hidden);
            }
            StoryMessages::Next => {}
        }
    }
}

fn manage_selection(
    mut commands: Commands,
    mut params: ResMut<TextSyllableValues>,
    mut selection_event: EventReader<SelectionChanged>,
    rock_run_assets: Res<RockRunAssets>,
) {
    for ev in selection_event.read() {
        match ev.movement {
            SelectionDirection::Up => {
                let (ltext, mut selection, rtext) = match decompose_selection_msg(&params.text) {
                    Some((ltext, selection, rtext)) => (ltext, selection, rtext),
                    None => return,
                };

                let sel_item = match selection.selection_items.get_mut(selection.selected_item) {
                    Some(item) => item,
                    None => return,
                };

                if let Ok(mut selection_number) = sel_item.parse::<usize>() {
                    selection_number += 1;
                    if selection_number > 9 {
                        *sel_item = "0".to_string();
                    } else {
                        *sel_item = selection_number.to_string();
                    }
                }

                if selection
                    .selection_items
                    .iter()
                    .all(|item| item.contains('\n'))
                {
                    if selection.selected_item == 0 {
                        selection.selected_item = selection.selection_items.len();
                    }
                    selection.selected_item -= 1;
                }

                params.text = compose_selection_msg(&ltext, selection, &rtext);

                commands.spawn((
                    AudioPlayer::new(rock_run_assets.story_plus_sound.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                ));

                debug!("{}", params.text);
            }
            SelectionDirection::Down => {
                let (ltext, mut selection, rtext) = match decompose_selection_msg(&params.text) {
                    Some((ltext, selection, rtext)) => (ltext, selection, rtext),
                    None => return,
                };

                let sel_item = match selection.selection_items.get_mut(selection.selected_item) {
                    Some(item) => item,
                    None => return,
                };

                if let Ok(mut selection_number) = sel_item.parse::<usize>() {
                    if selection_number == 0 {
                        *sel_item = "9".to_string();
                    } else {
                        selection_number -= 1;
                        *sel_item = selection_number.to_string();
                    }
                }

                if selection
                    .selection_items
                    .iter()
                    .all(|item| item.contains('\n'))
                {
                    selection.selected_item += 1;
                    if selection.selected_item == selection.selection_items.len() {
                        selection.selected_item = 0;
                    }
                }

                params.text = compose_selection_msg(&ltext, selection, &rtext);

                commands.spawn((
                    AudioPlayer::new(rock_run_assets.story_minus_sound.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                ));

                debug!("{}", params.text);
            }
            SelectionDirection::Left => {
                if let Some((ltext, mut selection, rtext)) = decompose_selection_msg(&params.text) {
                    if selection
                        .selection_items
                        .iter()
                        .all(|item| item.contains('\n'))
                    {
                        return;
                    }

                    if selection.selected_item == 0 {
                        selection.selected_item = selection.selection_items.len();
                    }
                    selection.selected_item -= 1;
                    params.text = compose_selection_msg(&ltext, selection, &rtext);
                }

                commands.spawn((
                    AudioPlayer::new(rock_run_assets.story_change_sound.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                ));

                debug!("{}", params.text);
            }
            SelectionDirection::Right => {
                if let Some((ltext, mut selection, rtext)) = decompose_selection_msg(&params.text) {
                    if selection
                        .selection_items
                        .iter()
                        .all(|item| item.contains('\n'))
                    {
                        return;
                    }

                    selection.selected_item += 1;
                    if selection.selected_item == selection.selection_items.len() {
                        selection.selected_item = 0;
                    }
                    params.text = compose_selection_msg(&ltext, selection, &rtext);
                }

                commands.spawn((
                    AudioPlayer::new(rock_run_assets.story_change_sound.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                ));

                debug!("{}", params.text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::palettes::css::{BLUE, GREEN, RED};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build_text_section_according_to_syllables() {
        let result = build_text_sections_according_to_syllables(
            "Hel-lo I am Rose, help me re-turn home.",
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(BLUE.into()),
            },
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(GREEN.into()),
            },
        );
        assert_eq!(result.len(), 17); // space counts as a section
        assert_eq!(result[0].1.color.0, BLUE.into());
        assert_eq!(result[0].0 .0, "Hel".to_string());
        assert_eq!(result[1].1.color.0, GREEN.into());
        assert_eq!(result[1].0 .0, "lo".to_string());
        assert_eq!(result[2].1.color.0, BLUE.into()); // first space
        assert_eq!(result[2].0 .0, " ".to_string());
        assert_eq!(result[3].1.color.0, BLUE.into());
        assert_eq!(result[3].0 .0, "I".to_string());
        assert_eq!(result[4].1.color.0, BLUE.into()); // second space
        assert_eq!(result[4].0 .0, " ".to_string());
        assert_eq!(result[5].1.color.0, GREEN.into());
        assert_eq!(result[5].0 .0, "am".to_string());
    }

    #[test]
    fn test_build_text_section_01() {
        let result = build_text_sections(
            r###"Ceci est un test: \("selection_items":["sel1","sel2"],"selected_item":0\)."###,
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(BLUE.into()),
            },
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(GREEN.into()),
            },
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(RED.into()),
            },
        );

        dbg!(&result);
        assert_eq!(result.len(), 10); // space counts as a section
        assert_eq!(result[0].1.color.0, BLUE.into());
        assert_eq!(result[0].0 .0, "Ceci".to_string());
        assert_eq!(result[1].1.color.0, BLUE.into());
        assert_eq!(result[1].0 .0, " ".to_string());
        assert_eq!(result[2].1.color.0, GREEN.into());
        assert_eq!(result[2].0 .0, "est".to_string());
        assert_eq!(result[3].1.color.0, BLUE.into());
        assert_eq!(result[3].0 .0, " ".to_string());
        assert_eq!(result[4].1.color.0, BLUE.into());
        assert_eq!(result[4].0 .0, "un".to_string());
        assert_eq!(result[5].1.color.0, BLUE.into());
        assert_eq!(result[5].0 .0, " ".to_string());
        assert_eq!(result[6].1.color.0, GREEN.into());
        assert_eq!(result[6].0 .0, "test:".to_string());
        assert_eq!(result[7].1.color.0, RED.into());
        assert_eq!(result[7].0 .0, "sel1".to_string());
        assert_eq!(result[8].1.color.0, BLUE.into());
        assert_eq!(result[8].0 .0, "sel2".to_string());
        assert_eq!(result[9].1.color.0, BLUE.into());
        assert_eq!(result[9].0 .0, ".".to_string());
    }

    #[test]
    fn test_build_text_section_02() {
        let result = build_text_sections(
            r###"Ceci est un test: \("selection_items":["sel1","sel2"],"selected_item":1\)."###,
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(BLUE.into()),
            },
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(GREEN.into()),
            },
            TextStyle {
                font: TextFont {
                    font: Handle::default(),
                    font_size: 42.0,
                    ..default()
                },
                color: TextColor(RED.into()),
            },
        );

        dbg!(&result);
        assert_eq!(result.len(), 10); // space counts as a section
        assert_eq!(result[0].1.color.0, BLUE.into());
        assert_eq!(result[0].0 .0, "Ceci".to_string());
        assert_eq!(result[1].1.color.0, BLUE.into());
        assert_eq!(result[1].0 .0, " ".to_string());
        assert_eq!(result[2].1.color.0, GREEN.into());
        assert_eq!(result[2].0 .0, "est".to_string());
        assert_eq!(result[3].1.color.0, BLUE.into());
        assert_eq!(result[3].0 .0, " ".to_string());
        assert_eq!(result[4].1.color.0, BLUE.into());
        assert_eq!(result[4].0 .0, "un".to_string());
        assert_eq!(result[5].1.color.0, BLUE.into());
        assert_eq!(result[5].0 .0, " ".to_string());
        assert_eq!(result[6].1.color.0, GREEN.into());
        assert_eq!(result[6].0 .0, "test:".to_string());
        assert_eq!(result[7].1.color.0, BLUE.into());
        assert_eq!(result[7].0 .0, "sel1".to_string());
        assert_eq!(result[8].1.color.0, RED.into());
        assert_eq!(result[8].0 .0, "sel2".to_string());
        assert_eq!(result[9].1.color.0, BLUE.into());
        assert_eq!(result[9].0 .0, ".".to_string());
    }
}
