use {
    crate::{components::*, Deck},
    bevy::prelude::*,
    rand::seq::IteratorRandom,
    bevy::log::prelude::info,
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
        cards.iter_mut().find(|(id, &flip, tracker)| {
            tracker.is_changed() && flip == Interaction::Clicked && deck.is_available_move(id.0)
        }).map(|x| x.0)
    } {
        deck.play(id.0);
        if deck.opened.len() == 2 {
            player.inc_turn();
        }
        id.1 += 1;
    };
}
pub fn score_board(
    mut players: Query<(&Player, &mut Text, &Parent)>,
    mut color: Query<&mut UiColor>,
    deck: Res<Deck>,
) {
    for (player, mut text, parent) in players.iter_mut() {
        let mut color = color.get_mut(parent.get()).unwrap();
        if deck.player() == player.deref().0 {
            color.0 = Color::GREEN;
        } else {
            color.0 = Color::WHITE;
        }
        text.sections[0].value = format!(
            "{}\nScore: {}\nOpened: {}\nTurns: {}\n{player:?}",
            if player.is_bot() { "Bot" } else { "Human" },
            deck.scores[player.deref().0 as usize],
            deck.iter()
                .filter(|&&c| c / 128 == 1 + player.deref().0 as u16)
                .count(),
                player.deref().1
        );
    }
}
