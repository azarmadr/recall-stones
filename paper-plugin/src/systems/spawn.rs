use crate::components::{Collection, Idx};
use crate::{Board, BoardAssets, BoardOptions};
use bevy::prelude::*;
use bevy::text::Text2dSize;

pub fn spawn_cards(
    mut commands: Commands,
    board: Res<Board>,
    board_options: Option<Res<BoardOptions>>,
    board_assets: Res<BoardAssets>,
    children: Query<(Entity, &Collection, &Idx)>,
    windows: Res<Windows>,
) {
    let size = match board_options {
        None => BoardOptions::default(), // If no options is set we use the default one
        Some(o) => o.clone(),
    }
    .adaptative_card_size(
        windows.get_primary().unwrap(),
        (board.deck.width(), board.deck.height()),
    );
    for (entity, col, id) in children.iter() {
        commands.entity(entity).remove::<Collection>();
        if let Some(&val) = board.get_card_val(id) {
            let color = board_assets.card_color(val, board.deck.max());
            commands.entity(entity).insert_bundle(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: val.to_string(),
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
