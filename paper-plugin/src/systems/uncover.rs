use crate::components::{Idx, Open, Revealed, Score};
use crate::events::{CardFlipEvent, DeckCompletedEvent};
use crate::{Board, BoardAssets};
use bevy::log;
use bevy::text::Text2dSize;
use bevy::prelude::*;
use bevy::render::view::Visibility;

pub fn flip_cards(
    mut commands: Commands,
    mut board: ResMut<Board>,
    children: Query<(Entity, &Idx), With<Open>>,
    mut score: Query<&mut Text, With<Score>>,
    mut visibility: Query<&mut Visibility>,
    mut deck_complete_ewr: EventWriter<DeckCompletedEvent>,
) {
    if children.iter().count() == 1 {
        for &entity in board.hidden_cards.values() {
            if let Ok(mut visible) = visibility.get_mut(entity) {
                visible.is_visible = false;
            }
        }
    }
    let mut open_cards = vec![];
    let mut text = score.single_mut();
    for (entity, id) in children.iter() {
        open_cards.push(*id);
        if let Ok(mut visible) = visibility.get_mut(entity) {
            visible.is_visible = true;
        }
    }
    if open_cards.len() > 1 {
        board.reveal_matching_cards(open_cards);
        for (entity, _) in children.iter() {
            commands.entity(entity).remove::<Open>();
        }
        board.score += 1;
        let rem_cards = board.hidden_cards.len() / 2;
        if rem_cards > 0 {
            text.sections[0].value = format!(
                "turns: {}\nLuck: {}\nPerfect Memory: {}",
                board.score,
                rem_cards,
                rem_cards * 2 - 1
            );
        }
    }
    if !board.completed && board.hidden_cards.len() == 0 {
        log::info!("Deck Completed");
        board.completed = true;
        text.sections[0].value = format!(
            "turns: {}\nLuck: {}\nPerfect Memory: {}",
            board.score,
            board.deck.count(),
            board.deck.count() * 2 - 1
        );
        deck_complete_ewr.send(DeckCompletedEvent);
    }
}
pub fn render_revealed(
    mut commands: Commands,
    board: ResMut<Board>,
    board_assets: Res<BoardAssets>,
    query: Query<(Entity, &Idx), (Without<Open>, Without<Revealed>)>,
) {
    for (entity, id) in query.iter() {
        if board.is_revealed(&id) {
            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(
                            board.opened_count(id).to_string(),
                            TextStyle {
                                color: board_assets.flag_material.color,
                                font: board_assets.counter_font.clone(),
                                font_size: 20.,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Left,
                                vertical: VerticalAlign::Top,
                            },
                        ),
                        transform: Transform::from_xyz(0., 0., 1.),
                        text_2d_size: Text2dSize {
                            size: Size::<f32>::new(20.0,20.)
                        },
                        ..Default::default()
                    })
                /*
                    .insert_bundle(SpriteBundle {
                        texture: board_assets.flag_material.texture.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(board.card_size)),
                            color: board_assets.flag_material.color,
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(0., 0., 1.),
                        ..Default::default()
                    })
                    */
                    .insert(Name::new("Flag"));
            })
            .insert(Revealed);
        }
    }
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
