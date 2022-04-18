use crate::components::*;
use crate::events::*;
use crate::tween::*;
use crate::{Board, BoardAssets};
use bevy::prelude::*;
use rand::seq::IteratorRandom;
use std::time::Duration;

const ROT_TIME: Duration = Duration::from_millis(81);
pub fn score(
    mut commands: Commands,
    mut board: ResMut<Board>,
    opened: Query<(Entity, &Idx, &Parent), With<Open>>,
    closed: Query<(Entity, &Parent), With<Close>>,
    mut score: Query<&mut Text, With<ScoreBoard>>,
) {
    let deck_len = board.deck.couplets() as usize;
    let mut text = score.single_mut();
    match opened.iter().count() {
        x if x == deck_len => {
            board.reveal_matching_cards(opened.iter().map(|x| *x.1).collect());
            for (entity, id, parent) in opened.iter() {
                commands.entity(entity).remove::<Open>();
                if board.is_revealed(id) {
                    commands.entity(entity).insert(Revealed);
                } else {
                    commands.entity(entity).insert(Close);
                    commands.entity(parent.0).insert(Animator::new(shake_seq(ROT_TIME)));
                }
            }
            board.inc_player_turn();
            let rem_cards = match board.hidden_cards.len() {
                0 => board.deck.count(),
                _ => board.hidden_cards.len() as u16 / 2,
            };
            //text.sections[0].value = format!("turns: {}      ", board.turns);
            text.sections[1].value = format!("Luck: {}      ", rem_cards);
            text.sections[2].value = format!("Perfect Memory: {}      ", rem_cards * 2 - 1);
        }
        1 => {
            for (entity, &parent) in closed.iter() {
                if opened.get(entity).is_err() {
                    commands.entity(parent.0).insert(Animator::new(rot_seq(ROT_TIME)));
                    commands
                        .entity(entity)
                        .insert(Animator::new(vis_seq(ROT_TIME,false)));
                }
                commands.entity(entity).remove::<Close>();
            }
        }
        _ => (),
    }
}
pub fn reveal_cards(
    mut commands: Commands,
    board: Res<Board>,
    board_assets: Res<BoardAssets>,
    revealed: Query<(Entity, &Idx), With<Revealed>>,
) {
    for (entity, id) in revealed.iter() {
        let count = board.opened_count(id);
        commands
            .entity(entity)
            .remove::<Revealed>()
            .insert(Name::new("Revealed"))
            .with_children(|parent| {
                parent.spawn_bundle(Text2dBundle {
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
                });
            });
    }
}
pub fn open_card(
    mut commands: Commands,
    board: Res<Board>,
    mut flip_card_evr: EventReader<CardFlipEvent>,
    mut animate_evr: EventReader<TweenCompleted>,
    parent: Query<&Parent>,
) {
    for CardFlipEvent(id) in flip_card_evr.iter() {
        if let Some(entity) = board.flip_card(&id) {
            commands
                .entity(parent.get(*entity).unwrap().0)
                .insert(Animator::new(rot_seq(ROT_TIME)));
            commands.entity(*entity).insert(Animator::new(
                vis_seq(ROT_TIME,true).with_completed_event(true, id.0 as u64),
            ));
        }
    }
    for event in animate_evr.iter() {
        if let Some(entity) = board.flip_card(&Idx(event.user_data as u16)) {
            commands.entity(*entity).insert(Open);
        }
    }
}
pub fn deck_complete(
    mut cmd: Commands,
    board: ResMut<Board>,
    mut event: EventWriter<DeckCompletedEvent>,
    mut score: Query<(Entity, &mut Text, Option<&Animator<Transform>>), With<ScoreBoard>>,
    mut animate_evr: EventReader<TweenCompleted>,
) {
    if board.hidden_cards.is_empty() {
        if let (entity, mut text, None) = score.single_mut(){
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
                    ROT_TIME * 27,
                    TextColorLens {
                        start: Color::WHITE,
                        end: Color::GREEN,
                        section: 0,
                    },
                )
                .with_completed_event(true, std::u64::MAX),
            ));
        }
    }
    for anim in animate_evr.iter() {
        if anim.user_data == std::u64::MAX {
            event.send(DeckCompletedEvent);
        }
    }
}
#[derive(Component)]
pub struct AiTimer(Timer);
pub fn ai(
    mut cmd: Commands,
    mut event: EventWriter<CardFlipEvent>,
    avail: Query<&Idx, (Without<Open>, Without<Revealed>)>,
    time: Res<Time>,
    mut query: Query<&mut AiTimer>,
) {
    if query.is_empty() {
        cmd.spawn().insert(AiTimer(Timer::from_seconds(0.5, false)));
    } else {
        let mut timer = query.single_mut();
        if timer.0.tick(time.delta()).just_finished() {
            let mut rng = rand::thread_rng();
            if !avail.is_empty() {
                event.send(CardFlipEvent(*avail.iter().choose(&mut rng).unwrap()));
            }
            timer.0.reset();
        }
    }
}
