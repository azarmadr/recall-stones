use crate::events::CardFlipEvent;
use crate::Board;
use bevy::input::{mouse::MouseButtonInput, ElementState};
use bevy::log;
use bevy::prelude::*;

pub fn input_handling(
    windows: Res<Windows>,
    board: Res<Board>,
    mut button_evr: EventReader<MouseButtonInput>,
    mut flip_card_ewr: EventWriter<CardFlipEvent>,
) {
    let window = windows.get_primary().unwrap();

    for event in button_evr.iter() {
        if let ElementState::Pressed = event.state {
            let position = window.cursor_position();
            if let Some(pos) = position {
                log::trace!("Mouse button pressed: {:?} at {}", event.button, pos);
                let tile_coordinates =
                    board.mouse_position(pos - Vec2::new(window.width(), window.height()) / 2.);
                if let Some(coordinates) = tile_coordinates {
                    if event.button == MouseButton::Left {
                        flip_card_ewr.send(CardFlipEvent(coordinates));
                    }
                }
            }
        }
    }
}
