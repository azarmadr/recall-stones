use crate::components::{Collection, Collection::*, Idx};
use crate::Mode::*;
use crate::{Board, BoardAssets, BoardOptions};
use bevy::prelude::*;
use bevy::text::Text2dSize;

pub fn spawn_cards(
    mut commands: Commands,
    board: Res<Board>,
    board_options: Res<BoardOptions>,
    board_assets: Res<BoardAssets>,
    children: Query<(Entity, &Collection, &Idx)>,
) {
    let size = board.card_size - board_options.card_padding;
    for (entity, col, id) in children.iter() {
        commands.entity(entity).remove::<Collection>();
        if let Some(&val) = board.get_card_val(id) {
            let color = match col {
                Spades | Clubs => Color::BLACK,
                Hearts | Diamonds => Color::RED,
                _ => board_assets.card_color(val, board.deck.max()),
            };
            let value = match col {
                Spades | Clubs | Hearts | Diamonds => char::from_digit(
                    match board_options.mode {
                        SameColor | Zebra => val % (board.deck.max() / 2),
                        TwoDecks | CheckeredDeck => val % (board.deck.max() / 4),
                        _ => val,
                    } as u32,
                    14,
                )
                .unwrap()
                .to_string(),
                _ => val.to_string(),
            };
            commands.entity(entity).insert_bundle(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value,
                        style: TextStyle {
                            color,
                            font: board_assets.col_map.get(col).unwrap().clone().typed(),
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
            });
        }
    }
}
