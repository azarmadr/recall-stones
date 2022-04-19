use crate::components::*;
use crate::events::*;
use crate::tween::*;
use crate::{Board, BoardAssets};
use bevy::prelude::*;
use std::time::Duration;

const ROT_TIME: Duration = Duration::from_millis(81);
pub fn score(
    mut cmd: Commands,
    mut board: ResMut<Board>,
    mut opened: Query<(Entity, &mut Idx, &Parent), With<Open>>,
    closed: Query<(Entity, &Parent), With<Close>>,
    mut score: Query<&mut Text, With<ScoreBoard>>,
    player: Query<(Entity, &Player), With<Turn>>,
) {
    let deck_len = board.deck.couplets() as usize;
    let mut text = score.single_mut();
    match opened.iter().count() {
        x if x == deck_len => {
            board.reveal_matching_cards(opened.iter().map(|x| **x.1).collect());
            for (entity, mut id, parent) in opened.iter_mut() {
                **id += 1;
                let (e, pl) = player.single();
                cmd.entity(entity).remove::<Open>();
                if board.is_revealed(&id) {
                    if !board.hidden_cards.is_empty() {
                    cmd.entity(entity).insert(Revealed);}
                    board.players[pl.deref() as usize].opened.push(id.0);
                } else {
                    cmd.entity(e).remove::<Turn>();
                    cmd
                        .entity(
                            board.players
                                [(pl.deref() as usize + 1) % board.players.len()]
                            .entity,
                        )
                        .insert(Turn);
                    cmd.entity(entity).insert(Close);
                    cmd
                        .entity(parent.0)
                        .insert(Animator::new(shake_seq(ROT_TIME)));
                }
            }
            //board.inc_player_turn();
            let rem_cards = match board.hidden_cards.len() {
                0 => board.deck.count(),
                _ => board.hidden_cards.len() as u8 / 2,
            };
            //text.sections[0].value = format!("turns: {}      ", board.turns);
            text.sections[1].value = format!("Luck: {}      ", rem_cards);
            text.sections[2].value = format!("Perfect Memory: {}      ", rem_cards * 2 - 1);
        }
        1 => {
            for (entity, &parent) in closed.iter() {
                if opened.get(entity).is_err() {
                    cmd
                        .entity(parent.0)
                        .insert(Animator::new(rot_seq(ROT_TIME)));
                    cmd
                        .entity(entity)
                        .insert(Animator::new(vis_seq(ROT_TIME, false)));
                }
                cmd.entity(entity).remove::<Close>();
            }
        }
        _ => (),
    }
}
pub fn reveal_cards(
    mut cmd: Commands,
    board_assets: Res<BoardAssets>,
    revealed: Query<(Entity, &Parent, &Idx), With<Revealed>>,
) {
    for (entity, &parent, id) in revealed.iter() {
        let count = id.1;
        cmd
            .entity(*parent)
            .insert(Animator::new(vis_seq(12 * ROT_TIME, false)));
        cmd
            .entity(entity)
            .remove::<Revealed>()
            .insert(Name::new("Revealed"))
            .insert(Animator::new(vis_seq(9 * ROT_TIME, false)))
            .with_children(|parent| {
                parent
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(
                            count.to_string(),
                            TextStyle {
                                color: board_assets.count_color(count),
                                font: board_assets.score_font.clone(),
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
                    .insert(Animator::new(vis_seq(8 * ROT_TIME, false)));
            });
    }
}
pub fn open_card(
    mut cmd: Commands,
    board: Res<Board>,
    mut flip_card_evr: EventReader<CardFlipEvent>,
    mut animate_evr: EventReader<TweenCompleted>,
    parent: Query<&Parent>,
) {
    for CardFlipEvent(id) in flip_card_evr.iter() {
        if let Some(entity) = board.flip_card(*id) {
            cmd.entity(parent.get(*entity).unwrap().0)
                .insert(Animator::new(rot_seq(ROT_TIME)));
            cmd.entity(*entity).insert(Animator::new(
                vis_seq(ROT_TIME, true).with_completed_event(true, *id as u64),
            ));
        }
    }
    for event in animate_evr.iter() {
        if let Some(entity) = board.flip_card(event.user_data as u8) {
            cmd.entity(*entity).insert(Open);
        }
    }
}
pub fn deck_complete(
    mut cmd: Commands,
    board: ResMut<Board>,
    mut event: EventWriter<DeckCompletedEvent>,
    mut score: Query<(Entity, &mut Text, Option<&Animator<Transform>>), With<ScoreBoard>>,
    cards: Query<(Entity, &Parent), With<Idx>>,
    mut animate_evr: EventReader<TweenCompleted>,
) {
    if board.hidden_cards.is_empty() {
        if let (entity, mut text, None) = score.single_mut() {
            text.sections[3].value = format!("Board Completed\n");
            cmd.entity(entity)
                .insert(Animator::new(Tween::new(
                    EaseFunction::ElasticInOut,
                    TweeningType::PingPong,
                    ROT_TIME * 9,
                    TransformScaleLens {
                        start: Vec3::splat(0.91),
                        end: Vec3::ONE,
                    },
                )))
                .insert(Animator::new(
                    Tween::new(
                        EaseFunction::ElasticInOut,
                        TweeningType::Once,
                        ROT_TIME * 81,
                        TextColorLens {
                            start: Color::WHITE,
                            end: Color::GREEN,
                            section: 0,
                        },
                    )
                    .with_completed_event(true, std::u64::MAX),
                ));
            let mut cycle = (15..27).cycle();
            for (entity, parent) in cards.iter() {
                [entity, parent.0]
                    .iter()
                    //.chain(**children.iter())
                    .for_each(|&e| {
                        cmd.entity(e).insert(Animator::new(vis_seq(
                            cycle.next().unwrap() * ROT_TIME,
                            true,
                        )));
                    });
            }
        }
    }
    for anim in animate_evr.iter() {
        if anim.user_data == std::u64::MAX {
            event.send(DeckCompletedEvent);
        }
    }
}
