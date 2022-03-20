use crate::components::Idx;
use crate::deck::Deck;
use crate::Bounds2;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Debug)]
pub struct Board {
    pub deck: Deck,
    pub bounds: Bounds2,
    pub card_size: f32,
    pub hidden_cards: HashMap<Idx, Entity>,
    pub entity: Entity,
    pub score: u32,
    pub completed: bool,
}

impl Board {
    /// Translates a mouse position to board coordinates
    pub fn mouse_position(&self, window: &Window, position: Vec2) -> Option<Idx> {
        let window_size = Vec2::new(window.width(), window.height());
        let position = position - window_size / 2.;

        if !self.bounds.in_bounds(position) {
            return None;
        }
        let coordinates = (position - self.bounds.position) / self.card_size;
        Some(Idx(
            coordinates.x as u16 + self.deck.width() * coordinates.y as u16
        ))
    }

    /// Retrieves a covered tile entity
    pub fn flip_card(&self, id: &Idx) -> Option<&Entity> {
        self.hidden_cards.get(&id)
    }

    pub fn is_revealed(&self, id: &Idx) -> bool {
        !self.hidden_cards.contains_key(&id)
    }

    /// reveal all the matching cards
    pub fn reveal_matching_cards(&mut self, ids: Vec<Idx>) {
        for id in self.deck.matching_cards(ids).iter() {
            //probably return to add cloaks
            self.hidden_cards.remove(&id);
        }
    }
}
