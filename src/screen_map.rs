#![allow(dead_code)] // TODO: remove
use bevy::prelude::*;
use std::ops::Range;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum Transition {
    #[default]
    Smooth,
    Hard,
}

/// A struct that describe a Screen
#[derive(Debug, PartialEq)]
pub struct Screen {
    x_index: usize,
    y_index: usize,
    x_range: Range<f32>,
    y_range: Range<f32>,
    start_screen: bool,
    allowed_screen: bool,
    fixed_screen: bool,
    transition: Transition,
}

impl Screen {
    /// Returns `true` if the point is in the screen
    pub fn contains(&self, point: &Vec2) -> bool {
        self.x_range.contains(&point.x) && self.y_range.contains(&point.y)
    }

    /// Returns the center of the screen as bevy coordinates
    pub fn get_center(&self) -> Vec2 {
        Vec2::new(
            (self.x_range.start + self.x_range.end) / 2.0,
            (self.y_range.start + self.y_range.end) / 2.0 - 1f32,
        )
    }

    /// Returns the indices of the screen
    ///
    /// Note: The origin is at the top left
    pub fn get_indices(&self) -> (usize, usize) {
        (self.x_index, self.y_index)
    }

    pub fn get_ranges(&self) -> (Range<f32>, Range<f32>) {
        (self.x_range.clone(), self.y_range.clone())
    }

    pub fn is_start_screen(&self) -> bool {
        self.start_screen
    }

    pub fn is_allowed_screen(&self) -> bool {
        self.allowed_screen
    }

    pub fn is_fixed_screen(&self) -> bool {
        self.fixed_screen
    }

    pub fn get_transition(&self) -> Transition {
        self.transition
    }
}

/// A struct to manage map of Screen
///
/// This `struct` is created by the [`Map::new()`] function. See its documentation for more.
#[derive(Debug, PartialEq)]
pub struct Map {
    width: usize,
    height: usize,
    screen_width: usize,
    screen_height: usize,
    data: Vec<Vec<Screen>>,
}

impl Map {
    /// Creates a map of Screen
    ///
    /// - 'X' screen can't be seen
    /// - 'O' screen can be seen
    /// - 'S' start screen
    /// - 'F' fixed screen, smooth transition
    /// - 'H' fixed screen, hard transition
    ///
    /// Transitions:
    /// - Smooth transition is the default
    ///
    /// # Examples
    ///
    /// A 3 x 3 screen map with 1280 x 720 screen resolution
    ///
    /// ```markdown
    ///    <-3840->
    ///   i+0|1|2+
    ///   -+-----+ ^
    ///   0|X|X|O| |
    ///   1|S|O|O| 2160
    ///   2|O|X|X| |
    ///   -+-----+ v
    /// ```
    ///
    /// ```rust
    ///  use screen_map::Map;
    ///  use screen_map::Screen;
    ///  use bevy::math::Vec2;
    ///
    ///  let screen_map = "XXO\nSOO\nOXX";
    ///  let screen_width = 1280;
    ///  let screen_height = 720;
    ///
    ///  let map = Map::new(screen_map, screen_width, screen_height);
    ///
    ///  assert_eq!(
    ///      map.get_screen_from_index(1, 1).unwrap().get_center(),
    ///      Vec2::new(0.0, 0.0)
    ///  );
    ///  assert_eq!(
    ///      map.get_screen_from_index(1, 1).unwrap().is_start_screen(),
    ///      false
    ///  );
    /// ```
    pub fn new(screen_map: &str, screen_width: usize, screen_height: usize) -> Self {
        let vert_size = screen_map.split('\n').count() * screen_height;
        let vert_center = vert_size / 2;
        let horiz_size = screen_map.split('\n').next().unwrap().len() * screen_width;
        let horiz_center = horiz_size / 2;

        let data = screen_map
            .split('\n')
            .enumerate()
            .map(|(cell_vert_index, horiz_screen)| {
                horiz_screen
                    .chars()
                    .enumerate()
                    .map(|(cell_horiz_index, screen_cell)| {
                        let x_range = (cell_horiz_index * screen_width) as f32 - horiz_center as f32
                            ..(cell_horiz_index * screen_width + screen_width) as f32
                                - horiz_center as f32;
                        let y_range = (vert_size as f32
                            - (cell_vert_index * screen_height + screen_height) as f32
                            - vert_center as f32)
                            + 1f32
                            ..(vert_size as f32
                                - (cell_vert_index * screen_height) as f32
                                - vert_center as f32)
                                + 1f32;
                        if screen_cell == 'X' {
                            Screen {
                                x_range,
                                y_range,
                                x_index: cell_horiz_index,
                                y_index: cell_vert_index,
                                start_screen: false,
                                allowed_screen: false,
                                fixed_screen: false,
                                transition: Transition::Smooth,
                            }
                        } else if screen_cell == 'O' {
                            Screen {
                                x_range,
                                y_range,
                                x_index: cell_horiz_index,
                                y_index: cell_vert_index,
                                start_screen: false,
                                allowed_screen: true,
                                fixed_screen: false,
                                transition: Transition::Smooth,
                            }
                        } else if screen_cell == 'S' {
                            Screen {
                                x_range,
                                y_range,
                                x_index: cell_horiz_index,
                                y_index: cell_vert_index,
                                start_screen: true,
                                allowed_screen: true,
                                fixed_screen: false,
                                transition: Transition::Smooth,
                            }
                        } else if screen_cell == 'F' {
                            Screen {
                                x_range,
                                y_range,
                                x_index: cell_horiz_index,
                                y_index: cell_vert_index,
                                start_screen: false,
                                allowed_screen: true,
                                fixed_screen: true,
                                transition: Transition::Smooth,
                            }
                        } else if screen_cell == 'H' {
                            Screen {
                                x_range,
                                y_range,
                                x_index: cell_horiz_index,
                                y_index: cell_vert_index,
                                start_screen: false,
                                allowed_screen: true,
                                fixed_screen: true,
                                transition: Transition::Hard,
                            }
                        } else {
                            unreachable!();
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            width: horiz_size,
            height: vert_size,
            screen_width,
            screen_height,
            data,
        }
    }
    pub fn tiled_to_bevy_coord(&self, tiled_coord: Vec2) -> Vec2 {
        Vec2::new(
            tiled_coord.x - (self.width / 2) as f32,
            -(tiled_coord.y - (self.height / 2) as f32) - 1f32,
        )
    }

    pub fn get_screen(&self, point: Vec2) -> Option<&Screen> {
        self.data
            .iter()
            .flatten()
            .find(|screen| screen.x_range.contains(&point.x) && screen.y_range.contains(&point.y))
    }

    pub fn get_above_screen(&self, point: Vec2) -> Option<&Screen> {
        let screen = match self.get_screen(point) {
            Some(screen) => screen,
            None => return None,
        };

        let (x_index, y_index) = screen.get_indices();

        let above_screen_y_index = match y_index {
            0 => return None,
            _ => y_index - 1,
        };

        match self.get_screen_from_index(x_index, above_screen_y_index) {
            Some(above_screen) => Some(above_screen),
            None => None,
        }
    }

    fn get_camera_points_coords(&self, point: Vec2) -> Vec<Vec2> {
        // Clockwise
        /*
          p2 +------------+ p3
             |            |
             |            |
          p1 +------------+ p0

        */
        let p0 = Vec2::from((
            point.x + (self.screen_width / 2 - 1) as f32,
            point.y - (self.screen_height / 2) as f32 + 1f32,
        ));
        let p1 = Vec2::from((
            point.x - (self.screen_width / 2) as f32,
            point.y - (self.screen_height / 2) as f32 + 1f32,
        ));
        let p2 = Vec2::from((
            point.x - (self.screen_width / 2) as f32,
            point.y + (self.screen_height / 2 - 1) as f32 + 1f32,
        ));
        let p3 = Vec2::from((
            point.x + (self.screen_width / 2 - 1) as f32,
            point.y + (self.screen_height / 2 - 1) as f32 + 1f32,
        ));

        vec![p0, p1, p2, p3]
    }

    pub fn move_camera(&self, old_pos: Vec2, new_pos: Vec2) -> Vec2 {
        let mut camera_pos = old_pos;
        let direction = new_pos - old_pos;

        enum Direction {
            Up,
            Down,
            Left,
            Right,
        }

        impl Direction {
            fn get_points(direction: Direction) -> (usize, usize) {
                match direction {
                    Direction::Right => (0, 3),
                    Direction::Down => (0, 1),
                    Direction::Left => (1, 2),
                    Direction::Up => (2, 3),
                }
            }
        }

        let camera_points = self.get_camera_points_coords(Vec2::new(new_pos.x, old_pos.y));

        if direction.x == 0.0 {
            // Do nothing because we're not moving horizontally
        } else if
        // move to right
        direction.x > 0.0
            && self.check_points(
                &camera_points,
                Direction::get_points(Direction::Right),
        )
        // move to left
        || direction.x < 0.0
            && self.check_points(
                &camera_points,
                Direction::get_points(Direction::Left),
            )
        {
            camera_pos.x = new_pos.x;
        } else {
            // Stick the camera horizontally to the center of the screen
            if let Some(screen) = self.get_screen(new_pos) {
                let screen_center = screen.get_center();
                camera_pos.x = screen_center.x;
            }
        }

        let camera_points = self.get_camera_points_coords(Vec2::new(camera_pos.x, new_pos.y));
        if direction.y == 0.0 {
            // Do nothing because we're not moving vertically
        } else if
        // move up
        direction.y > 0.0
            && self.check_points(
                &camera_points,
                Direction::get_points(Direction::Up),
            )
        // move down
        || direction.y < 0.0
            && self.check_points(
                &camera_points,
                Direction::get_points(Direction::Down),
            )
        {
            camera_pos.y = new_pos.y;
        } else {
            // Stick the camera vertically to the center of the screen
            if let Some(screen) = self.get_screen(new_pos) {
                let screen_center = screen.get_center();
                camera_pos.y = screen_center.y;
            }
        }

        camera_pos
    }

    fn check_points(&self, camera_points: &[Vec2], (p1, p2): (usize, usize)) -> bool {
        self.get_screen(camera_points[p1]).is_some()
            && self.get_screen(camera_points[p2]).is_some()
            && self.get_screen(camera_points[p1]).unwrap().allowed_screen
            && self.get_screen(camera_points[p2]).unwrap().allowed_screen
    }

    /// Returns the start screen
    ///
    /// # Examples
    ///
    /// ```rust
    ///  use screen_map::Map;
    ///  use screen_map::Screen;
    ///  use bevy::math::Vec2;
    ///  let screen_map = "XXO\nSOO\nOXX";
    ///  let screen_width = 1280;
    ///  let screen_height = 720;
    ///
    ///  let map = Map::new(screen_map, screen_width, screen_height);
    ///
    ///  assert_eq!(
    ///      map.get_start_screen().get_indices(),
    ///      (
    ///           0,
    ///           1
    ///      )
    ///  );
    /// ```
    pub fn get_start_screen(&self) -> &Screen {
        self.data
            .iter()
            .flatten()
            .find(|screen| screen.start_screen)
            .unwrap()
    }

    /// Returns the screen at index (index_x, index_y) or None if it doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    ///  use screen_map::Map;
    ///  use screen_map::Screen;
    ///  use bevy::math::Vec2;
    ///  let screen_map = "XXO\nSOO\nOXX";
    ///  let screen_width = 1280;
    ///  let screen_height = 720;
    ///
    ///  let map = Map::new(screen_map, screen_width, screen_height);
    ///
    ///  assert_eq!(
    ///      map.get_screen_from_index(1, 1).unwrap().get_ranges(),
    ///      (
    ///           -640.0..640.0,
    ///           -359.0..361.0
    ///      )
    ///  );
    /// ```
    pub fn get_screen_from_index(&self, index_x: usize, index_y: usize) -> Option<&Screen> {
        // Origin is top left
        self.data
            .iter()
            .flatten()
            .find(|screen| screen.x_index == index_x && screen.y_index == index_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_map_parsing() {
        let screen_map = "XOX\nSOO\nXXX";
        let screen_width = 1280;
        let screen_height = 720;

        let output = Map::new(screen_map, screen_width, screen_height);
        assert_eq!(
            output,
            Map {
                width: 3 * screen_width,
                height: 3 * screen_height,
                screen_width,
                screen_height,
                data: vec![
                    vec![
                        Screen {
                            x_range: -1920.0..-640.0,
                            y_range: 361.0..1081.0,
                            x_index: 0,
                            y_index: 0,
                            start_screen: false,
                            allowed_screen: false,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: -640.0..640.0,
                            y_range: 361.0..1081.0,
                            x_index: 1,
                            y_index: 0,
                            start_screen: false,
                            allowed_screen: true,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: 640.0..1920.0,
                            y_range: 361.0..1081.0,
                            x_index: 2,
                            y_index: 0,
                            start_screen: false,
                            allowed_screen: false,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                    ],
                    vec![
                        Screen {
                            x_range: -1920.0..-640.0,
                            y_range: -359.0..361.0,
                            x_index: 0,
                            y_index: 1,
                            start_screen: true,
                            allowed_screen: true,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: -640.0..640.0,
                            y_range: -359.0..361.0,
                            x_index: 1,
                            y_index: 1,
                            start_screen: false,
                            allowed_screen: true,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: 640.0..1920.0,
                            y_range: -359.0..361.0,
                            x_index: 2,
                            y_index: 1,
                            start_screen: false,
                            allowed_screen: true,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                    ],
                    vec![
                        Screen {
                            x_range: -1920.0..-640.0,
                            y_range: -1079.0..-359.0,
                            x_index: 0,
                            y_index: 2,
                            start_screen: false,
                            allowed_screen: false,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: -640.0..640.0,
                            y_range: -1079.0..-359.0,
                            x_index: 1,
                            y_index: 2,
                            start_screen: false,
                            allowed_screen: false,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                        Screen {
                            x_range: 640.0..1920.0,
                            y_range: -1079.0..-359.0,
                            x_index: 2,
                            y_index: 2,
                            start_screen: false,
                            allowed_screen: false,
                            fixed_screen: false,
                            transition: Transition::Smooth,
                        },
                    ],
                ]
            }
        );
    }

    #[test]
    fn test_tiled_to_bevy_coord() {
        let screen_map = "XOX\nSOO\nXXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(0.0, 0.0)),
            Vec2::new(-1920.0, 1079.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(1919.0, 0.0)),
            Vec2::new(-1.0, 1079.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(0.0, 719.0)),
            Vec2::new(-1920.0, 360.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(3839.0, 2159.0)),
            Vec2::new(1919.0, -1080.0)
        );
    }

    #[test]
    fn test_tiled_to_bevy_coord2() {
        let screen_map = "SO";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(0.0, 0.0)),
            Vec2::new(-1280.0, 359.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(1280.0, 360.0)),
            Vec2::new(0.0, -1.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(0.0, 719.0)),
            Vec2::new(-1280.0, -360.0)
        );
        assert_eq!(
            map.tiled_to_bevy_coord(Vec2::new(2559.0, 719.0)),
            Vec2::new(1279.0, -360.0)
        );
    }

    #[test]
    fn test_get_screen() {
        let screen_map = "XOX\nSOO\nXXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(
            map.get_screen(Vec2::new(0.0, 0.0)),
            Some(&Screen {
                x_range: -640.0..640.0,
                y_range: -359.0..361.0,
                x_index: 1,
                y_index: 1,
                start_screen: false,
                allowed_screen: true,
                fixed_screen: false,
                transition: Transition::Smooth,
            })
        )
    }

    #[test]
    fn test_get_window_points_coord() {
        let screen_map = "XOX\nSOO\nXXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(
            map.get_camera_points_coords(Vec2::new(0.0, 0.0)),
            vec![
                Vec2::new(639.0, -359.0),
                Vec2::new(-640.0, -359.0),
                Vec2::new(-640.0, 360.0),
                Vec2::new(639.0, 360.0)
            ]
        )
    }

    #[test]
    fn test_move_camera() {
        let screen_map = "XXO\nSOO\nOXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        // From middle screen
        // Move to right
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            Vec2::new(10.0, 0.0)
        );
        // Move to left
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(-10.0, 0.0)),
            Vec2::new(-10.0, 0.0)
        );
        // Move up is not possible so the camera should not move
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(0.0, 10.0)),
            Vec2::new(0.0, 0.0)
        );
        // Move down is not possible so the camera should not move
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(0.0, -10.0)),
            Vec2::new(0.0, 0.0)
        );
        // Move up and move right
        // Camera should only move on x
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(100.0, 10.0)),
            Vec2::new(100.0, 0.0)
        );
        // Move down and move left
        // Camera should only move on x
        assert_eq!(
            map.move_camera(Vec2::new(0.0, 0.0), Vec2::new(-100.0, -10.0)),
            Vec2::new(-100.0, 0.0)
        );

        // From right middle screen
        // Move to right
        // Move right is not possible so the camera should not move
        assert_eq!(
            map.move_camera(Vec2::new(1280.0, 0.0), Vec2::new(1300.0, 0.0)),
            Vec2::new(1280.0, 0.0)
        );
        // Move to left
        assert_eq!(
            map.move_camera(Vec2::new(1280.0, 0.0), Vec2::new(1260.0, 0.0)),
            Vec2::new(1260.0, 0.0)
        );
        // Move up
        assert_eq!(
            map.move_camera(Vec2::new(1280.0, 0.0), Vec2::new(1280.0, 100.0)),
            Vec2::new(1280.0, 100.0)
        );
        // Move down
        // Move down is not possible so the camera should not move
        assert_eq!(
            map.move_camera(Vec2::new(1280.0, 0.0), Vec2::new(1280.0, -100.0)),
            Vec2::new(1280.0, 0.0)
        );
        // Move up and move right
        // Camera should only move on y and x must be at the screen center
        assert_eq!(
            map.move_camera(Vec2::new(1290.0, 0.0), Vec2::new(1300.0, 150.0)),
            Vec2::new(1280.0, 150.0)
        );
        // Move down and move left
        // Camera should only move on x
        assert_eq!(
            map.move_camera(Vec2::new(1280.0, 0.0), Vec2::new(1200.0, -120.0)),
            Vec2::new(1200.0, 0.0)
        );
        // Move down and move left
        // Camera should be clipped to the screen center
        assert_eq!(
            map.move_camera(Vec2::new(1380.0, 0.0), Vec2::new(1300.0, -120.0)),
            Vec2::new(1300.0, 0.0)
        );
    }

    #[test]
    fn test_get_center() {
        let screen_map = "XXO\nSOO\nOXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);
        dbg!(&map);

        dbg!(map.get_screen(Vec2::new(0.0, 0.0)).unwrap());
        assert_eq!(
            map.get_screen(Vec2::new(0.0, 0.0)).unwrap().get_center(),
            Vec2::new(0.0, 0.0)
        );
        assert_eq!(
            map.get_screen(Vec2::new(-1900.0, 400.0))
                .unwrap()
                .get_center(),
            Vec2::new(-1280.0, 720.0)
        );
    }

    #[test]
    fn test_get_start_screen_and_move_camera() {
        let screen_map = "XXO\nSOO\nOXX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(map.get_start_screen().get_center(), Vec2::new(-1280.0, 0.0));

        // From middle screen
        // Move to right
        assert_eq!(
            map.move_camera(Vec2::new(-1280.0, 0.0), Vec2::new(-1240.0, 0.0)),
            Vec2::new(-1240.0, 0.0)
        );
        // Move to left
        assert_eq!(
            map.move_camera(Vec2::new(-1280.0, 0.0), Vec2::new(-1300.0, 0.0)),
            Vec2::new(-1280.0, 0.0)
        );
        // Move up and left
        // Camera should not move and remain at the screen center
        assert_eq!(
            map.move_camera(Vec2::new(-1290.0, 0.0), Vec2::new(-1300.0, 100.0)),
            Vec2::new(-1280.0, 0.0)
        );
        // Move down and left
        // Camera should move only on y and x must be at the screen center
        assert_eq!(
            map.move_camera(Vec2::new(-1290.0, 0.0), Vec2::new(-1300.0, -250.0)),
            Vec2::new(-1280.0, -250.0)
        );
    }

    #[test]
    fn test_get_screen_from_index() {
        let screen_map = "XXO\nSHO\nOOX";
        let screen_width = 1280;
        let screen_height = 720;

        let map = Map::new(screen_map, screen_width, screen_height);

        assert_eq!(
            map.get_screen_from_index(1, 1),
            Some(&Screen {
                x_range: -640.0..640.0,
                y_range: -359.0..361.0,
                x_index: 1,
                y_index: 1,
                start_screen: false,
                allowed_screen: true,
                fixed_screen: true,
                transition: Transition::Hard,
            })
        );

        assert_eq!(
            map.get_screen_from_index(2, 2),
            Some(&Screen {
                x_range: 640.0..1920.0,
                y_range: -1079.0..-359.0,
                x_index: 2,
                y_index: 2,
                start_screen: false,
                allowed_screen: false,
                fixed_screen: false,
                transition: Transition::Smooth,
            })
        );
        assert_eq!(map.get_screen_from_index(2, 3), None);
    }
}
