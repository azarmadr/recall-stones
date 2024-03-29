use crate::{components::*, tween::*, Deck, MemoryGAssts, MemoryGOpts};
use {bevy::prelude::*, std::time::Duration};

pub(crate) const ROT_TIME: Duration = Duration::from_millis(81);
pub fn card_flip(
    mut cards: Query<&mut BackgroundColor, With<Idx>>,
    vis: Query<(&Parent, &Visibility), With<Animator<Visibility>>>,
    assets: Res<MemoryGAssts>,
) {
    vis.iter().for_each(|(p, v)| {
        if let Ok(c) = cards.get_mut(p.get()).as_mut() {
            assets.flip_card_color(c, v.is_visible);
        }
    });
}
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
        text: Text::from_section(
            id.1.to_string(),
            TextStyle {
                color: assets.count_color(id.1),
                font: assets.score_font.clone(),
                font_size: 22.,
            },
        )
        .with_alignment(TextAlignment {
            horizontal: HorizontalAlign::Left,
            vertical: VerticalAlign::Top,
        }),
        style: Style {
            position: UiRect {
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
        deck.opened.last().map(|&v| {
            opened.push(v);
            tween(true, find_card(v).0);
            Some(())
        });
        if !new_turn {
            cards
                .iter()
                .filter(|x| deck.opened.contains(&x.1 .0))
                .for_each(|(entity, id)| {
                    if deck.is_available_move(id.0) {
                        // println!("SSS");
                        cmd.entity(entity)
                            .insert(Animator::new(shake_seq(ROT_TIME)));
                    } else if deck_complete {
                        opened.clear();
                        if let Ok(children) = children.get(entity) {
                            children.iter().for_each(|&child| {
                                cmd.entity(child).with_children(|parent| {
                                    parent.spawn(text(id));
                                });
                            });
                        };
                    } else {
                        cmd.entity(entity)
                            .insert(Animator::new(vis_seq(12 * ROT_TIME, false)));
                        for &child in &**children.get(entity).unwrap() {
                            cmd.entity(child)
                                .insert(Animator::new(vis_seq(9 * ROT_TIME, false)))
                                .with_children(|parent| {
                                    parent
                                        .spawn(text(id))
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
    cards: Query<Entity, With<Idx>>,
    deck: Res<Deck>,
    children: Query<&Children>,
    mut opts: ResMut<MemoryGOpts>,
) {
    opts.outcome = deck.outcome();
    if opts.outcome.is_some() {
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
                if let Ok(children) = children.get(child) {
                    children.iter().for_each(|&child| tween(child));
                };
            }
        }
        cmd.remove_resource::<Deck>();
    }
}
