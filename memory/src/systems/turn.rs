use bevy_tweening::{Animator, EaseFunction, Tween, TweeningType};

use crate::tween::BeTween;

use super::ROT_TIME;

use {
    crate::{components::*, Deck},
    bevy::prelude::*,
    rand::seq::IteratorRandom,
    // bevy::log::prelude::info,
};

#[derive(Deref, DerefMut)]
pub struct AiTimer(Timer);
impl Default for AiTimer {
    fn default() -> Self {
        AiTimer(Timer::from_seconds(0.5, false))
    }
}
/// Whether the ai or human, get the index of the move and add `Open` Component to that entity
pub fn turn(
    mut players: Query<&mut Player>,
    time: Res<Time>,
    mut timer: Local<AiTimer>,
    mut deck: ResMut<Deck>,
    mut cards: Query<(&mut Idx, &Interaction, ChangeTrackers<Interaction>)>,
) {
    let mut player = players
        .iter_mut()
        .find(|pl| deck.player() == pl.deref().0)
        .unwrap();
    let mut rng = rand::thread_rng();

    if let Some(mut id) = if player.is_bot() && timer.tick(time.delta()).just_finished() {
        timer.reset();
        cards
            .iter_mut()
            .filter(|(id, _, _)| deck.is_available_move(id.0))
            .choose(&mut rng)
            .map(|x| x.0)
    } else if player.is_bot() {
        None
    } else {
        cards
            .iter_mut()
            .find(|(id, &flip, tracker)| {
                tracker.is_changed() && flip == Interaction::Clicked && deck.is_available_move(id.0)
            })
            .map(|x| x.0)
    } {
        deck.play(id.0);
        if deck.opened.len() == 2 {
            player.inc_turn();
        }
        id.1 += 1;
    };
}
pub fn score_board(
    players: Query<(Entity, &Player, &Parent)>,
    deck: Res<Deck>,
    // children: Query<&Children>,
    mut text: Query<&mut Text>,
    mut cmd: Commands,
    mut turn_change: Local<Option<u8>>,
) {
    if turn_change.map_or(deck.outcome().is_none(), |x| x != deck.player()) {
        for (entity, player, parent) in players.iter() {
            let is_player = player.id() == deck.player();
            cmd.entity(**parent).insert(Animator::new(Tween::new(
                EaseFunction::QuadraticIn,
                TweeningType::Once,
                ROT_TIME * 2,
                BeTween::with_lerp(move |c: &mut UiColor, _, r| {
                    let end = if is_player {
                        Color::GREEN
                    } else {
                        Color::WHITE
                    };
                    let start: Vec4 = c.0.into();
                    *c = UiColor(start.lerp(end.into(), r).into());
                }),
            )));
            let mut text = text.get_mut(entity).unwrap();
            text.sections[0].value = format!(
                "{} {}\nScore: {}\nOpened: {}\nTurns: {}\n",
                if player.is_bot() { "Bot" } else { "Human" },
                player.deref().0,
                deck.scores[player.deref().0 as usize],
                deck.iter()
                    .filter(|&&c| c / 128 == 1 + player.deref().0 as u16)
                    .count(),
                player.deref().1
            );
        }
        *turn_change = Some(deck.player());
    }
    if deck.outcome().is_some() {
        *turn_change = None
    }
}
