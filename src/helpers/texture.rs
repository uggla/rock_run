use std::ops::RangeInclusive;

use bevy::{ecs::system::Local, image::TextureAtlas, log::trace};

#[derive(Debug, Default, Eq, PartialEq)]
pub enum IndexDirection {
    #[default]
    Up,
    Down,
}

pub fn cycle_texture(texture: &mut TextureAtlas, texture_index_range: RangeInclusive<usize>) {
    if !texture_index_range.contains(&texture.index) {
        texture.index = *texture_index_range.start();
    }
    texture.index = if texture.index == *texture_index_range.end() {
        *texture_index_range.start()
    } else {
        texture.index + 1
    };
}

pub fn swing_texture(
    texture: &mut TextureAtlas,
    texture_index_range: RangeInclusive<usize>,
    index_direction: &mut Local<IndexDirection>,
) {
    if !texture_index_range.contains(&texture.index) {
        texture.index = *texture_index_range.start();
    }

    if texture.index == *texture_index_range.end() && **index_direction == IndexDirection::Up {
        **index_direction = IndexDirection::Down;
    }

    if texture.index == *texture_index_range.start() && **index_direction == IndexDirection::Down {
        **index_direction = IndexDirection::Up;
    }

    trace!("tdirection: {:?}", index_direction);
    trace!("tindex: {}", texture.index);
    texture.index = if **index_direction == IndexDirection::Up {
        texture.index + 1
    } else {
        texture.index - 1
    };
}
