use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;

/// Material of a `Sprite` with a texture and color
#[derive(Debug, Clone)]
pub struct SpriteMaterial {
    pub color: Color,
    pub texture: Handle<Image>,
}

impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: DEFAULT_IMAGE_HANDLE.typed(),
        }
    }
}

/// Assets for the board. Must be used as a resource.
///
/// Use the loader for partial setup
#[derive(Debug, Clone)]
pub struct BoardAssets {
    /// Label
    pub label: String,
    ///
    pub board_material: SpriteMaterial,
    ///
    pub card_material: SpriteMaterial,
    ///
    pub covered_card_material: SpriteMaterial,
    ///
    pub counter_font: Handle<Font>,
    ///
    pub counter_colors: Vec<Color>,
    ///
    pub flag_material: SpriteMaterial,
    ///
    pub material: SpriteMaterial,
}

impl BoardAssets {
    /// Default bomb counter color set
    pub fn default_colors() -> Vec<Color> {
        vec![
            Color::WHITE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::PURPLE,
        ]
    }

    /// Safely retrieves the color matching a bomb counter
    pub fn bomb_counter_color(&self, counter: u8) -> Color {
        let counter = counter.saturating_sub(1) as usize;
        match self.counter_colors.get(counter) {
            Some(c) => *c,
            None => match self.counter_colors.last() {
                None => Color::WHITE,
                Some(c) => *c,
            },
        }
    }
}
