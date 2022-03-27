use std::collections::HashMap;
use bevy::text::Text2dSize;
use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use serde::{Deserialize, Serialize};

/// Collection specifying corresponing assets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Collection {
    Eng, Tel
}

use Collection::*;

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
    pub col_map: HashMap<Collection, HandleUntyped>
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
    pub fn card_color(&self, value: u16) -> Color {
        let value = value.saturating_sub(1) as usize;
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

impl Collection {
    pub fn translate(&self, val: u16, max: u16,size: f32, board_assets: &BoardAssets) -> Text2dBundle {
        // We retrieve the text and the correct color
        let color = board_assets.card_color(val * board_assets.card_color.len() as u16 / max);
        // We generate a text bundle
        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: {
                        match self {
                            Eng => val.to_string(),
                            Tel => val.to_string().chars().fold(String::new(),|a,x| format!("{}{}",a,char::from_u32(0xc66+x.to_digit(10).unwrap()).unwrap()))
                        }

                    },
                    style: TextStyle {
                        color,
                        font: board_assets.col_map.get(self).unwrap().clone().typed(),
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            text_2d_size: Text2dSize {
                size: Size {
                    width: size,
                    height: size,
                },
            },
            visibility: Visibility { is_visible: false },
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        }
    }
}
