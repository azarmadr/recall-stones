use crate::components::*;
use crate::events::*;
use crate::tween::*;
use crate::{Board, BoardAssets};
use bevy::prelude::*;
use std::time::Duration;

const ROT_TIME: Duration = Duration::from_millis(81);
fn rot_seq() -> Sequence<Transform> {
    let start = 0.;
    let end = std::f32::consts::PI / 2.;
    let tween = |start, end| {
        Tween::new(
            EaseFunction::QuadraticIn,
            TweeningType::Once,
            ROT_TIME,
            TransformRotateYLens { start, end },
        )
    };
    tween(start, end).then(tween(end, start))
}
fn vis_seq(show: bool) -> Tween<Visibility> {
    Tween::new(
        EaseFunction::QuadraticIn,
        TweeningType::Once,
        2 * ROT_TIME,
        VisibilityLens(show),
    )
}
fn shake_seq() -> Sequence<Transform> {
    let tween = |x, i| {
        Tween::new(
            EaseFunction::ElasticInOut,
            TweeningType::Once,
            ROT_TIME * i / 3,
            TransformPositionLensByDelta(x),
        )
    };
    Sequence::new((1..4).rev().map(|i| {
        tween(Vec3::X / 3. * i as f32, i)
            .then(tween(Vec3::X / 3. * -2. * i as f32, i))
            .then(tween(Vec3::X / 3. * i as f32, i))
    }))
}
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
                    commands.entity(parent.0).insert(Animator::new(shake_seq()));
                }
            }
            board.turns += 1;
            let rem_cards = match board.hidden_cards.len() {
                0 => board.deck.count(),
                _ => board.hidden_cards.len() as u16 / 2,
            };
            text.sections[0].value = format!("turns: {}      ",board.turns);
            text.sections[1].value = format!( "Luck: {}      ", rem_cards);
            text.sections[2].value = format!( "Perfect Memory: {}      ", rem_cards * 2 - 1);
        }
        1 => {
            for (entity, &parent) in closed.iter() {
                if opened.get(entity).is_err() {
                    commands.entity(parent.0).insert(Animator::new(rot_seq()));
                    commands
                        .entity(entity)
                        .insert(Animator::new(vis_seq(false)));
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
                .insert(Animator::new(rot_seq()));
            commands.entity(*entity).insert(Animator::new(
                vis_seq(true).with_completed_event(true, id.0 as u64),
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
    mut board: ResMut<Board>,
    mut event: EventWriter<DeckCompletedEvent>,
    mut score: Query<(Entity, &mut Text), With<ScoreBoard>>,
    mut animate_evr: EventReader<TweenCompleted>,
) {
    if !board.completed && board.hidden_cards.is_empty() {
        board.completed = true;
        let (entity, mut text) = score.single_mut();
        text.sections[0].value = format!("Board Completed\n");
        text.sections[1].value = format!("{}{}", text.sections[0].value,text.sections[1].value);
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
    for anim in animate_evr.iter() {
        if anim.user_data == std::u64::MAX {
            event.send(DeckCompletedEvent);
        }
    }
}
