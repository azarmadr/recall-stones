use crate::resources::{BoardAssets, Mode, Mode::*};
use bevy::prelude::*;
use bevy::text::Text2dSize;
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
    pub fn spawn_card(
        &self,
        val: u8,
        assets: &Res<BoardAssets>,
        max: u8,
        size: f32,
        mode: Mode,
    ) -> Text2dBundle {
        let color = match self {
            Spades | Clubs => Color::BLACK,
            Hearts | Diamonds => Color::RED,
            _ => assets.card_color(val, max),
        };
        let value = match self {
            Spades | Clubs | Hearts | Diamonds => char::from_digit(
                match mode {
                    SameColor | Zebra => val % (max / 2),
                    TwoDecks | CheckeredDeck => val % (max / 4),
                    _ => val,
                } as u32,
                14,
            )
            .unwrap()
            .to_string(),
            _ => val.to_string(),
        };
        Text2dBundle {
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
