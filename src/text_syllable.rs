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

const Z_VALUE: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct SyllableStyle {
    pub font_size: f32,
    pub color: Color,
}

pub struct TextSyllablePlugin {
    pub font: AssetPath<'static>,
    pub radius: f32,
    pub box_position: Vec2,
    pub box_size: Vec2,
    pub box_filling: Source<'static>,
    pub style_a: SyllableStyle,
    pub style_b: SyllableStyle,
    pub text: String,
}

#[derive(Component)]
struct TextSyllableBox;

#[derive(Component)]
struct TextSyllable;

#[derive(Resource)]
pub struct TextSyllableValues {
    font: AssetPath<'static>,
    radius: f32,
    box_position: Vec2,
    box_size: Vec2,
    box_filling: Source<'static>,
    def_style_a: SyllableStyle,
    def_style_b: SyllableStyle,
    pub text: String,
    style_a: TextStyle,
    style_b: TextStyle,
}

impl Default for TextSyllablePlugin {
    fn default() -> Self {
        let box_size = Vec2::new(500.0, 200.0);

        Self {
            font: "fonts/FiraSans-Bold.ttf".into(),
            radius: 20.0,
            box_position: Vec2::new(0.0, 0.0),
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
            text: "Hel-lo I am Rose, help me re-turn home.".into(),
        }
    }
}

impl Plugin for TextSyllablePlugin {
    fn build(&self, app: &mut App) {
        let text_params = TextSyllableValues {
            font: self.font.clone(),
            radius: self.radius,
            box_position: self.box_position,
            box_size: self.box_size,
            box_filling: self.box_filling.clone(),
            def_style_a: self.style_a.clone(),
            def_style_b: self.style_b.clone(),
            text: self.text.clone(),
            style_a: TextStyle::default(),
            style_b: TextStyle::default(),
        };
        app.insert_resource(text_params);
        app.add_systems(Startup, setup);
        app.add_systems(Update, toggle_visibility);
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
                transform: Transform::from_translation(params.box_position.extend(Z_VALUE)),
                ..default()
            },
            TextSyllableBox,
        ))
        .with_children(|builder| {
            builder.spawn((
                Text2dBundle {
                    text: Text {
                        sections: build_text_sections_according_to_syllables(
                            &params.text,
                            params.style_a.clone(),
                            params.style_b.clone(),
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

fn toggle_visibility(
    camera: Query<&Transform, (With<Camera>, Without<TextSyllableBox>)>,
    mut text_syllable_box: Query<(&mut Transform, &mut Visibility), With<TextSyllableBox>>,
    mut text_syllable: Query<&mut Text, With<TextSyllable>>,
    params: Res<TextSyllableValues>,
) {
    if let Ok(camera_pos) = camera.get_single() {
        if let Ok((mut transform, mut visibility)) = text_syllable_box.get_single_mut() {
            *visibility = Visibility::Visible;
            transform.translation =
                Vec3::new(camera_pos.translation.x, camera_pos.translation.y, Z_VALUE);

            if let Ok(mut text) = text_syllable.get_single_mut() {
                text.sections = build_text_sections_according_to_syllables(
                    params.text.as_str(),
                    params.style_a.clone(),
                    params.style_b.clone(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build_text_section() {
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
}
