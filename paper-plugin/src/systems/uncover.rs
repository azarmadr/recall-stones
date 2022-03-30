use crate::components::{Idx, Open, Revealed, Score};
use crate::events::{CardFlipEvent, DeckCompletedEvent};
use crate::{Board, BoardAssets};
use bevy::log;
use bevy::prelude::*;
use bevy::render::view::Visibility;

pub fn flip_cards(
    mut commands: Commands,
    mut board: ResMut<Board>,
    board_assets: Res<BoardAssets>,
    children: Query<(Entity, &Idx), With<Open>>,
    mut score: Query<&mut Text, With<Score>>,
    mut visibility: Query<&mut Visibility>,
    mut deck_complete_ewr: EventWriter<DeckCompletedEvent>,
) {
    let deck_len = board.deck.couplets() as usize;
    let mut text = score.single_mut();
    for (entity, _) in children.iter() {
        if let Ok(mut visibility) = visibility.get_mut(entity) {
            visibility.is_visible = true;
        }
    }
    match children.iter().count() {
        x if x == deck_len => {
            board.reveal_matching_cards(children.iter().map(|x| *x.1).collect());
            for (entity, id) in children.iter() {
                commands.entity(entity).remove::<Open>();
                if board.is_revealed(id) {
                    commands
                        .entity(entity)
                        .insert(Revealed)
                        .with_children(|parent| {
                            render_revealed(parent, board.opened_count(id), &board_assets)
                        });
                }
            }
            board.score += 1;
            let rem_cards = match board.hidden_cards.len() {
                0 => board.deck.count(),
                _ => board.hidden_cards.len() as u16 / 2,
            };
            text.sections[0].value = format!(
                "turns: {}\nLuck: {}\nPerfect Memory: {}",
                board.score,
                rem_cards,
                rem_cards * 2 - 1
            );
        }
        1 => {
            for &entity in board.hidden_cards.values() {
                if let Ok(mut visibility) = visibility.get_mut(entity) {
                    if let Err(..) = children.get(entity) {
                        visibility.is_visible = false;
                    }
                }
            }
        }
        _ => (),
    }
    if !board.completed && board.hidden_cards.is_empty() {
        log::info!("Deck Completed");
        board.completed = true;
        deck_complete_ewr.send(DeckCompletedEvent);
    }
}
pub fn render_revealed(parent: &mut ChildBuilder, count: u16, board_assets: &BoardAssets) {
    parent
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                count.to_string(),
                TextStyle {
                    color: board_assets.count_color(count),
                    font: board_assets.counter_font.clone(),
                    font_size: 27.,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Left,
                    vertical: VerticalAlign::Top,
                },
            ),
            transform: Transform::from_xyz(10., 0., 1.),
            ..Default::default()
        })
        .insert(Name::new("Taps"));
}
pub fn trigger_event_handler(
    mut commands: Commands,
    board: Res<Board>,
    mut flip_card_evr: EventReader<CardFlipEvent>,
) {
    for trigger_event in flip_card_evr.iter() {
        if let Some(entity) = board.flip_card(&trigger_event.0) {
            commands.entity(*entity).insert(Open);
        }
    }
}
