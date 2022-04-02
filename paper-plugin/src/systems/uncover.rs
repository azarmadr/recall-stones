use crate::components::{Close, Idx, Open, Revealed, Score};
use crate::events::{CardFlipEvent, DeckCompletedEvent};
use crate::{Board, BoardAssets};
use bevy::log;
use bevy::prelude::*;
use bevy::render::view::Visibility;
use bevy_tweening::{lens::*, *};
use std::time::Duration;

const SHOW_TIME: Duration = Duration::from_millis(27);
const ROT_TIME: Duration = Duration::from_millis(81);

pub fn deck_complete(mut board: ResMut<Board>, mut event: EventWriter<DeckCompletedEvent>) {
    if !board.completed && board.hidden_cards.is_empty() {
        log::info!("Deck Completed");
        board.completed = true;
        event.send(DeckCompletedEvent);
    }
}
pub fn flip_cards(
    mut commands: Commands,
    mut board: ResMut<Board>,
    children: Query<(Entity, &Idx, &Parent), With<Open>>,
    mut score: Query<&mut Text, With<Score>>,
    transform: Query<&Transform>,
) {
    let deck_len = board.deck.couplets() as usize;
    let mut text = score.single_mut();
    match children.iter().count() {
        x if x == deck_len => {
            board.reveal_matching_cards(children.iter().map(|x| *x.1).collect());
            for (entity, id, parent) in children.iter() {
                commands.entity(entity).remove::<Open>();
                if board.is_revealed(id) {
                    commands.entity(entity).insert(Revealed);
                } else {
                    commands.entity(entity).insert(Close);
                    let Transform { translation, .. } = transform.get(parent.0).unwrap();
                    let mut seq = Sequence::from_single(Delay::new(ROT_TIME + SHOW_TIME));
                    for i in (1..4).rev() {
                        seq = seq
                            .then(Tween::new(
                                EaseFunction::ElasticInOut,
                                TweeningType::Once,
                                SHOW_TIME*i,
                                TransformPositionLens {
                                    start: *translation - (Vec3::X * i as f32),
                                    end: *translation,
                                },
                            ))
                            .then(Tween::new(
                                EaseFunction::ElasticInOut,
                                TweeningType::Once,
                                SHOW_TIME*i,
                                TransformPositionLens {
                                    start: Vec3::X * i as f32 + *translation,
                                    end: *translation,
                                },
                            ))
                    }
                    commands.entity(parent.0).insert(Animator::new(seq));
                }
            }
            board.turns += 1;
            let rem_cards = match board.hidden_cards.len() {
                0 => board.deck.count(),
                _ => board.hidden_cards.len() as u16 / 2,
            };
            text.sections[0].value = format!(
                "turns: {}\nLuck: {}\nPerfect Memory: {}",
                board.turns,
                rem_cards,
                rem_cards * 2 - 1
            );
        }
        _ => (),
    }
}
pub fn close_cards(
    mut commands: Commands,
    children: Query<(Entity, &Parent), With<Close>>,
    open: Query<Entity, With<Open>>,
) {
    if let Ok(opened) = open.get_single() {
        for (entity, &parent) in children.iter() {
            if entity != opened {
            let rot_seq = Tween::new(
                EaseFunction::QuadraticIn,
                TweeningType::Once,
                ROT_TIME,
                TransformRotateYLens {
                    start: 0.,
                    end: std::f32::consts::PI / 2.,
                },
            )
            .then(Sequence::from_single(Delay::new(SHOW_TIME)))
            .then(Tween::new(
                EaseFunction::QuadraticOut,
                TweeningType::Once,
                ROT_TIME,
                TransformRotateYLens {
                    end: 0.,
                    start: std::f32::consts::PI / 2.,
                },
            ));
            let vis_seq = Sequence::from_single(Delay::new(ROT_TIME)).then(Tween::new(
                EaseFunction::QuadraticIn,
                TweeningType::Once,
                SHOW_TIME,
                VisibilityLens { show: false },
            ));
            commands.entity(parent.0).insert(Animator::new(rot_seq));
            commands.entity(entity).insert(Animator::new(vis_seq));
    }
            commands.entity(entity).remove::<Close>();
        }
    }
}
pub fn render_revealed(
    mut commands: Commands,
    board: Res<Board>,
    board_assets: Res<BoardAssets>,
    revealed: Query<(Entity, &Idx), With<Revealed>>,
) {
    for (entity, id) in revealed.iter() {
        let count = board.opened_count(id);
        commands.entity(entity).remove::<Revealed>();
        commands
            .entity(entity)
            .insert(Name::new("Revealed"))
            .with_children(|parent| {
                parent.spawn_bundle(Text2dBundle {
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
                });
            });
    }
}
struct VisibilityLens {
    /// boolean to decide whether to show the component. true -> shows.
    show: bool,
}
impl Lens<Visibility> for VisibilityLens {
    fn lerp(&mut self, target: &mut Visibility, ratio: f32) {
        target.is_visible = self.show ^ (ratio < 0.5);
    }
}
pub fn trigger_event_handler(
    mut commands: Commands,
    board: Res<Board>,
    mut flip_card_evr: EventReader<CardFlipEvent>,
    parent: Query<&Parent>,
) {
    for trigger_event in flip_card_evr.iter() {
        if let Some(entity) = board.flip_card(&trigger_event.0) {
            let rot_seq = Tween::new(
                EaseFunction::QuadraticIn,
                TweeningType::Once,
                ROT_TIME,
                TransformRotateYLens {
                    start: 0.,
                    end: std::f32::consts::PI / 2.,
                },
            )
            .then(Sequence::from_single(Delay::new(SHOW_TIME)))
            .then(Tween::new(
                EaseFunction::QuadraticOut,
                TweeningType::Once,
                ROT_TIME,
                TransformRotateYLens {
                    end: 0.,
                    start: std::f32::consts::PI / 2.,
                },
            ));
            let vis_seq = Sequence::from_single(Delay::new(ROT_TIME)).then(Tween::new(
                EaseFunction::QuadraticIn,
                TweeningType::Once,
                SHOW_TIME,
                VisibilityLens { show: true },
            ));

            commands
                .entity(parent.get(*entity).unwrap().0)
                .insert(Animator::new(rot_seq));
            commands
                .entity(*entity)
                .insert(Open)
                .insert(Animator::new(vis_seq));
        }
    }
}
