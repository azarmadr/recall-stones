use crate::{components::*, tween::*, MemoryGAssts, Deck, DeckCompletedEvent};
use {bevy::prelude::*, std::time::Duration};

const ROT_TIME: Duration = Duration::from_millis(81);
pub fn uncover(
    mut cmd: Commands,
    mut opened: Local<Vec<usize>>,
    deck: Res<Deck>,
    assets: Res<MemoryGAssts>,
    cards: Query<(Entity, &Idx)>,
    children: Query<&Children>,
) {
    let mut tween = |show, entity| {
        cmd.entity(entity).insert(Animator::new(rot_seq(ROT_TIME)));
        for &child in &**children.get(entity).unwrap() {
            cmd.entity(child)
                .insert(Animator::new(vis_seq(ROT_TIME, show)));
        }
    };
    let find_card = |i| cards.iter().find(|(_, &id)| id.0 == i).unwrap();
    let new_turn = deck.opened.len() == 1;
    let deck_complete = deck.outcome().is_some();

    let text = |id: &Idx| TextBundle {
        text: Text::with_section(
            id.1.to_string(),
            TextStyle {
                color: assets.count_color(id.1),
                font: assets.score_font.clone(),
                font_size: 22.,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Left,
                vertical: VerticalAlign::Top,
            },
        ),
        style: Style {
            position: Rect {
                left: Val::Px(20.),
                ..default()
            },
            size: Size {
                width: Val::Px(27.),
                height: Val::Px(27.),
            },
            ..default()
        },
        ..default()
    };
    if if deck_complete {
        !opened.is_empty()
    } else {
        deck.opened.len() != opened.len()
    } {
        if new_turn {
            opened.drain(..).for_each(|c| {
                if !deck.is_revealed(c) {
                    tween(false, find_card(c).0);
                }
            });
        }
        deck.opened.last().and_then(|&v| {
            opened.push(v);
            tween(true, find_card(v).0);
            Some(())
        });
        if !new_turn {
            cards
                .iter()
                .filter_map(|x| {
                    if deck.opened.contains(&x.1 .0) {
                        Some(x)
                    } else {
                        None
                    }
                })
                .for_each(|(entity, id)| {
                    if deck.is_available_move(id.0) {
                        println!("SSS");
                        cmd.entity(entity)
                            .insert(Animator::new(shake_seq(ROT_TIME)));
                    } else if deck_complete {
                        opened.clear();
                        children.get(entity).ok().and_then(|children| {
                            children.iter().for_each(|&child| {
                                cmd.entity(child).with_children(|parent| {
                                    parent.spawn_bundle(text(id));
                                });
                            });
                            Some(())
                        });
                    } else {
                        cmd.entity(entity)
                            .insert(Animator::new(vis_seq(12 * ROT_TIME, false)));
                        for &child in &**children.get(entity).unwrap() {
                            cmd.entity(child)
                                .insert(Animator::new(vis_seq(9 * ROT_TIME, false)))
                                .with_children(|parent| {
                                    parent
                                        .spawn_bundle(text(id))
                                        .insert(Animator::new(vis_seq(8 * ROT_TIME, false)));
                                });
                        }
                    }
                });
        }
    }
}
pub fn deck_complete(
    mut cmd: Commands,
    mut event: EventWriter<DeckCompletedEvent>,
    mut score: Query<(Entity, &mut Text, Option<&Animator<Transform>>), With<ScoreBoard>>,
    cards: Query<Entity, With<Idx>>,
    mut animate_evr: EventReader<TweenCompleted>,
    deck: Res<Deck>,
    children: Query<&Children>,
) {
    if deck.outcome().is_some() {
        if let (entity, mut text, None) = score.single_mut() {
            text.sections[3].value = format!("Deck Completed\n");
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
            let mut tween = |e| {
                cmd.entity(e).insert(Animator::new(vis_seq(
                    cycle.next().unwrap() * ROT_TIME,
                    true,
                )));
            };
            for entity in cards.iter() {
                tween(entity);
                for &child in &**children.get(entity).unwrap() {
                    tween(child);
                    children.get(child).ok().and_then(|children| {
                        children.iter().for_each(|&child| tween(child));
                        Some(())
                    });
                }
            }
        }
    }
    for anim in animate_evr.iter() {
        if anim.user_data == std::u64::MAX {
            event.send(DeckCompletedEvent);
        }
    }
}
