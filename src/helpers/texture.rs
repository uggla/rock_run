use std::ops::RangeInclusive;

use bevy::{ecs::system::Local, log::trace, sprite::TextureAtlas};

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
    direction: &mut Local<IndexDirection>,
) {
    if !texture_index_range.contains(&texture.index) {
        texture.index = *texture_index_range.start();
    }

    if texture.index == *texture_index_range.end() && **direction == IndexDirection::Up {
        **direction = IndexDirection::Down;
    }

    if texture.index == *texture_index_range.start() && **direction == IndexDirection::Down {
        **direction = IndexDirection::Up;
    }

    trace!("tdirection: {:?}", direction);
    trace!("tindex: {}", texture.index);
    texture.index = if **direction == IndexDirection::Up {
        texture.index + 1
    } else {
        texture.index - 1
    };
}
