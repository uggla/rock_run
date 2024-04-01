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

use raqote::{
    DrawOptions, DrawTarget, Gradient, GradientStop, PathBuilder, Point, SolidSource, Source,
    Spread,
};

pub struct TextSyllablePlugin {
    pub(crate) font: AssetPath<'static>,
    pub(crate) radius: f32,
    pub(crate) box_position: Vec2,
    pub(crate) box_size: Vec2,
}

#[derive(Resource)]
struct TextSyllable {
    font: AssetPath<'static>,
    radius: f32,
    box_position: Vec2,
    box_size: Vec2,
}

impl Default for TextSyllablePlugin {
    fn default() -> Self {
        Self {
            font: "fonts/FiraSans-Bold.ttf".into(),
            radius: 20.0,
            box_position: Vec2::new(-300.0, -237.0),
            box_size: Vec2::new(500.0, 200.0),
        }
    }
}

impl Plugin for TextSyllablePlugin {
    fn build(&self, app: &mut App) {
        let text_params = TextSyllable {
            font: self.font.clone(),
            radius: self.radius,
            box_position: self.box_position,
            box_size: self.box_size,
        };
        app.insert_resource(text_params);
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<Image>>,
    params: Res<TextSyllable>,
) {
    let font = asset_server.load(params.font.clone());
    let radius = 20.0;
    let box_position = Vec2::new(-300.0, -237.0);
    let box_size = Vec2::new(500.0, 200.0);

    let mut dt = DrawTarget::new(box_size.x as i32, box_size.y as i32);

    let path = shape_rounded_rectangle(radius, box_size);

    // Define the color and fill style
    let red = Source::Solid(SolidSource {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    });

    let gradient = Source::new_linear_gradient(
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
    );

    // Draw the path
    dt.fill(&path, &gradient, &DrawOptions::new());

    let image = Image::new(
        Extent3d {
            width: box_size.x as u32,
            height: box_size.y as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        argb_to_rgba(dt.get_data()),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = assets.add(image);

    // Demonstrate text wrapping
    let slightly_smaller_text_style_blue = TextStyle {
        font: font.clone(),
        font_size: 42.0,
        color: Color::BLUE,
    };
    let slightly_smaller_text_style_green = TextStyle {
        font: font.clone(),
        font_size: 42.0,
        color: Color::DARK_GREEN,
    };
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                // color: Color::rgb(0.25, 0.25, 0.75),
                // custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                ..default()
            },
            // visibility: Visibility::Hidden,
            texture: image_handle,
            transform: Transform::from_translation(box_position.extend(10.0)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections: build_text_sections_according_to_syllables(
                        "Hel-lo I am Rose, help me re-turn home.",
                        slightly_smaller_text_style_blue,
                        slightly_smaller_text_style_green,
                    ),
                    justify: JustifyText::Left,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: Vec2::new(box_size.x, box_size.y),
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            });
        });
}

fn shape_rounded_rectangle(radius: f32, box_size: Vec2) -> raqote::Path {
    let mut pb = PathBuilder::new();
    pb.move_to(radius, 0.0);

    pb.line_to(box_size.x - radius, 0.);
    // Top right corner
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
    let path = pb.finish();
    path
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
