use crate::components::*;
use crate::events::*;
//use crate::tween::*;
use crate::Board;
use bevy::prelude::*;
//use std::time::Duration;
use rand::seq::IteratorRandom;

pub fn turn(
    //mut cmd: Commands,
    mut player: Query<(Entity, &mut Player), Added<Turn>>,
) {
    if let Ok((_entity, mut player)) = player.get_single_mut() {
        player.inc_turn();
    }
}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AiTimer(Timer);
pub fn ai(
    mut cmd: Commands,
    mut event: EventWriter<CardFlipEvent>,
    player: Query<&Player, With<Turn>>,
    time: Res<Time>,
    board: Res<Board>,
    mut query: Query<&mut AiTimer>,
) {
    if let Ok(Player::Bolts(_)) = player.get_single() {
        if query.is_empty() {
            cmd.spawn().insert(AiTimer(Timer::from_seconds(1.5, false)));
        } else {
            let mut timer = query.single_mut();
            if timer.0.tick(time.delta()).just_finished() {
                let mut rng = rand::thread_rng();
                if !board.hidden_cards.is_empty() {
                    event.send(CardFlipEvent(
                        *board.hidden_cards.keys().choose(&mut rng).unwrap(),
                    ));
                }
                timer.0.reset();
            }
        }
    }
}
