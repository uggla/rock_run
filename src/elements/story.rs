use std::f32::consts::PI;

use bevy::{
    asset::AssetPath,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    text::{BreakLineOn, Text2dBounds},
};

use raqote::{DrawOptions, DrawTarget, Gradient, GradientStop, PathBuilder, Point, Source, Spread};
use serde::{Deserialize, Serialize};

use crate::{
    coregame::state::AppState,
    events::{SelectionChanged, StoryMessages},
};

const Z_VALUE: f32 = 15.0;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TextSyllableState {
    #[default]
    Hidden,
    Visible,
}

#[derive(Clone, Debug)]
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
struct TextSyllable;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSelection {
    pub selection_items: Vec<String>,
    selected_item: usize,
}

pub enum SelectionDirection {
    Up,
    Down,
    Left,
    Right,
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
        let box_size = Vec2::new(500.0, 200.0);

        Self {
            font: "fonts/FiraSans-Bold.ttf".into(),
            radius: 20.0,
            box_position: Vec3::new(0.0, 0.0, Z_VALUE),
            box_size,
            box_filling: Source::new_linear_gradient(
                Gradient {
                    stops: vec![
                        GradientStop {
                            position: 0.0,
                            color: raqote::Color::new(0xff, 0xff, 0xff, 0xff),
                        },
                        // GradientStop {
                        //     position: 0.9999,
                        //     color: Color::new(0xff, 0x0, 0x0, 0x0),
                        // },
                        GradientStop {
                            position: 1.0,
                            color: raqote::Color::new(0xff, 0xa4, 0xa4, 0xa4),
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
                color: Color::BLUE,
            },
            style_b: SyllableStyle {
                font_size: 42.0,
                color: Color::DARK_GREEN,
            },
            style_selected: SyllableStyle {
                font_size: 42.0,
                color: Color::RED,
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
            .add_systems(Startup, setup)
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
        font: font.clone(),
        font_size: params.def_style_a.font_size,
        color: params.def_style_a.color,
    };
    params.style_b = TextStyle {
        font: font.clone(),
        font_size: params.def_style_b.font_size,
        color: params.def_style_b.color,
    };
    params.style_selected = TextStyle {
        font: font.clone(),
        font_size: params.def_style_selected.font_size,
        color: params.def_style_selected.color,
    };
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    // color: Color::rgb(0.25, 0.25, 0.75),
                    // custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                    ..default()
                },
                visibility: Visibility::Hidden,
                texture: image_handle,
                transform: Transform::from_translation(params.box_position),
                ..default()
            },
            TextSyllableBox,
        ))
        .with_children(|builder| {
            builder.spawn((
                Text2dBundle {
                    text: Text {
                        sections: build_text_sections(
                            &params.text,
                            params.style_a.clone(),
                            params.style_b.clone(),
                            params.style_selected.clone(),
                        ),
                        justify: JustifyText::Left,
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    text_2d_bounds: Text2dBounds {
                        // Wrap text in the rectangle
                        size: Vec2::new(params.box_size.x, params.box_size.y),
                    },
                    // ensure the text is drawn on top of the box
                    transform: Transform::from_translation(Vec3::Z),
                    ..default()
                },
                TextSyllable,
            ));
        });
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
) -> Vec<TextSection> {
    let mut toggle_style = true;
    let syllables = text.split_whitespace().collect::<Vec<_>>();
    let syllables = syllables
        .iter()
        .map(|s| s.split('-').collect::<Vec<&str>>())
        .collect::<Vec<Vec<&str>>>();

    let text_sections: Vec<TextSection> = syllables
        .iter()
        .map(|s| {
            s.iter()
                .map(|o| {
                    if toggle_style {
                        toggle_style = false;
                        TextSection::new(o.to_string(), style_a.clone())
                    } else {
                        toggle_style = true;
                        TextSection::new(o.to_string(), style_b.clone())
                    }
                })
                .collect::<Vec<TextSection>>()
        })
        .enumerate()
        .flat_map(|(i, mut s)| {
            if i < syllables.len() - 1 {
                s.extend(vec![TextSection::new(" ".to_string(), style_a.clone())]);
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

fn compose_selection_msg(ltext: &str, selection: UserSelection, rtext: &str) -> String {
    let selection = serde_json::to_string(&selection).unwrap();
    let selection = selection.replace('{', "\\(").replace('}', "\\)");
    format!("{}{}{}", ltext, selection, rtext)
}

fn build_text_sections(
    text: &str,
    style_a: TextStyle,
    style_b: TextStyle,
    style_selected: TextStyle,
) -> Vec<TextSection> {
    let text_sections = match decompose_selection_msg(text) {
        Some((ltext, selection, rtext)) => {
            let selection = selection
                .selection_items
                .iter()
                .enumerate()
                .map(|item| {
                    if item.0 == selection.selected_item {
                        TextSection::new(item.1.to_string(), style_selected.clone())
                    } else {
                        TextSection::new(item.1.to_string(), style_a.clone())
                    }
                })
                .collect::<Vec<TextSection>>();

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
                .collect::<Vec<TextSection>>()
        }
        None => build_text_sections_according_to_syllables(text, style_a, style_b),
    };
    text_sections
}

fn toggle_visibility(
    camera: Query<&Transform, (With<Camera>, Without<TextSyllableBox>)>,
    mut text_syllable_box: Query<(&mut Transform, &mut Visibility), With<TextSyllableBox>>,
    mut text_syllable: Query<&mut Text, With<TextSyllable>>,
    params: Res<TextSyllableValues>,
    visible_state: Res<State<TextSyllableState>>,
) {
    if visible_state.get() == &TextSyllableState::Visible {
        if let Ok(camera_pos) = camera.get_single() {
            if let Ok((mut transform, mut visibility)) = text_syllable_box.get_single_mut() {
                *visibility = Visibility::Visible;
                transform.translation =
                    Vec3::new(camera_pos.translation.x, camera_pos.translation.y, 0.0)
                        + params.box_position;

                if let Ok(mut text) = text_syllable.get_single_mut() {
                    text.sections = build_text_sections(
                        params.text.as_str(),
                        params.style_a.clone(),
                        params.style_b.clone(),
                        params.style_selected.clone(),
                    )
                }
            }
        }
    } else if let Ok((_, mut visibility)) = text_syllable_box.get_single_mut() {
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
    mut params: ResMut<TextSyllableValues>,
    mut selection_event: EventReader<SelectionChanged>,
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

                params.text = compose_selection_msg(&ltext, selection, &rtext);
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

                params.text = compose_selection_msg(&ltext, selection, &rtext);
                debug!("{}", params.text);
            }
            SelectionDirection::Left => {
                if let Some((ltext, mut selection, rtext)) = decompose_selection_msg(&params.text) {
                    if selection.selected_item == 0 {
                        selection.selected_item = selection.selection_items.len();
                    }
                    selection.selected_item -= 1;
                    params.text = compose_selection_msg(&ltext, selection, &rtext);
                }
                debug!("{}", params.text);
            }
            SelectionDirection::Right => {
                if let Some((ltext, mut selection, rtext)) = decompose_selection_msg(&params.text) {
                    selection.selected_item += 1;
                    if selection.selected_item == selection.selection_items.len() {
                        selection.selected_item = 0;
                    }
                    params.text = compose_selection_msg(&ltext, selection, &rtext);
                }
                debug!("{}", params.text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build_text_section_according_to_syllables() {
        let result = build_text_sections_according_to_syllables(
            "Hel-lo I am Rose, help me re-turn home.",
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::BLUE,
            },
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::GREEN,
            },
        );
        assert_eq!(result.len(), 17); // space counts as a section
        assert_eq!(result[0].style.color, Color::BLUE);
        assert_eq!(result[1].style.color, Color::GREEN);
        assert_eq!(result[2].style.color, Color::BLUE); // first space
        assert_eq!(result[3].style.color, Color::BLUE);
        assert_eq!(result[4].style.color, Color::BLUE); // second space
        assert_eq!(result[5].style.color, Color::GREEN);
    }

    #[test]
    fn test_build_text_section_01() {
        let result = build_text_sections(
            r###"Ceci est un test: \("selection_items":["sel1","sel2"],"selected_item":0\)."###,
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::BLUE,
            },
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::GREEN,
            },
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::RED,
            },
        );

        dbg!(&result);
        assert_eq!(result.len(), 10); // space counts as a section
        assert_eq!(result[0].style.color, Color::BLUE);
        assert_eq!(result[1].style.color, Color::BLUE);
        assert_eq!(result[2].style.color, Color::GREEN);
        assert_eq!(result[3].style.color, Color::BLUE);
        assert_eq!(result[4].style.color, Color::BLUE);
        assert_eq!(result[5].style.color, Color::BLUE);
        assert_eq!(result[6].style.color, Color::GREEN);
        assert_eq!(result[7].style.color, Color::RED);
        assert_eq!(result[8].style.color, Color::BLUE);
        assert_eq!(result[9].style.color, Color::BLUE);
    }

    #[test]
    fn test_build_text_section_02() {
        let result = build_text_sections(
            r###"Ceci est un test: \("selection_items":["sel1","sel2"],"selected_item":1\)."###,
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::BLUE,
            },
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::GREEN,
            },
            TextStyle {
                font: Handle::default(),
                font_size: 42.0,
                color: Color::RED,
            },
        );

        dbg!(&result);
        assert_eq!(result.len(), 10); // space counts as a section
        assert_eq!(result[0].style.color, Color::BLUE);
        assert_eq!(result[1].style.color, Color::BLUE);
        assert_eq!(result[2].style.color, Color::GREEN);
        assert_eq!(result[3].style.color, Color::BLUE);
        assert_eq!(result[4].style.color, Color::BLUE);
        assert_eq!(result[5].style.color, Color::BLUE);
        assert_eq!(result[6].style.color, Color::GREEN);
        assert_eq!(result[7].style.color, Color::BLUE);
        assert_eq!(result[8].style.color, Color::RED);
        assert_eq!(result[9].style.color, Color::BLUE);
    }
}
