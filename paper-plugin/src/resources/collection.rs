use crate::resources::BoardAssets;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Collection specifying corresponing assets
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Collection {
    Eng,
    Tel,
    Clubs,
    Diamonds,
    Spades,
    Hearts,
    Dice,
}
use Collection::*;
impl Collection {
    pub fn spawn(&self, val: u16, assets: &Res<BoardAssets>, max: u8, size: f32) -> TextBundle {
        let color = match self {
            Spades | Clubs => Color::BLACK,
            Hearts | Diamonds => Color::RED,
            _ => assets.card_color(val, max),
        };
        let value = match self {
            Spades | Clubs | Hearts | Diamonds => {
                char::from_digit(val as u32 % 14, 14).unwrap().to_string()
            }
            _ => val.to_string(),
        };
        TextBundle {
            style: Style {
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value,
                    style: TextStyle {
                        color,
                        font: assets.col_map.get(self).unwrap().clone().typed(),
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        }
    }
}
