use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use std::collections::HashMap;

use crate::components::Collection;

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
    pub board_material: SpriteMaterial,
    pub card_material: SpriteMaterial,
    pub covered_card_material: SpriteMaterial,
    pub counter_font: Handle<Font>,
    pub card_color: Vec<Color>,
    pub material: SpriteMaterial,
    pub col_map: HashMap<Collection, HandleUntyped>,
}

impl BoardAssets {
    /// Default card value color set
    pub fn default_colors() -> Vec<Color> {
        vec![
            Color::WHITE,
            Color::GREEN,
            Color::YELLOW,
            Color::ORANGE,
            Color::PURPLE,
        ]
    }

    /// Safely retrieves the color matching a value
    pub fn card_color(&self, val: u16, max: u16) -> Color {
        let value = (val * self.card_color.len() as u16 / max).saturating_sub(1) as usize;
        match self.card_color.get(value) {
            Some(c) => *c,
            None => match self.card_color.last() {
                None => Color::WHITE,
                Some(c) => *c,
            },
        }
    }

    pub fn count_color(&self, val: u16) -> Color {
        match val {
            1 => Color::GREEN,
            2 => Color::WHITE,
            3 => Color::YELLOW,
            4 => Color::ORANGE,
            _ => Color::RED,
        }
    }
}
