use crate::components::Idx;
use crate::deck::Deck;
use crate::Bounds2;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Board {
    pub deck: Deck,
    pub bounds: Bounds2,
    pub card_size: f32,
    pub hidden_cards: HashMap<Idx, Entity>,
    pub opened_count: HashMap<Idx, u16>,
    pub entity: Entity,
    pub turns: u32,
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
    #[inline]
    #[must_use]
    pub fn flip_card(&self, id: &Idx) -> Option<&Entity> {
        self.hidden_cards.get(id)
    }
    #[inline]
    #[must_use]
    pub fn is_revealed(&self, id: &Idx) -> bool {
        !self.hidden_cards.contains_key(id)
    }
    #[inline]
    #[must_use]
    pub fn opened_count(&self, id: &Idx) -> u16 {
        match self.opened_count.get(id) {
            Some(v) => *v,
            None => 1,
        }
    }
    #[inline]
    #[must_use]
    pub fn get_card_val(&self, id: &Idx) -> Option<&u16> {
        let Idx(i) = *id;
        self.deck.get(i as usize)
    }
    /// reveal all the matching cards
    #[inline]
    pub fn reveal_matching_cards(&mut self, ids: Vec<Idx>) {
        for e in ids.iter() {
            let count = self.opened_count.entry(*e).or_insert(0);
            *count += 1;
        }
        if self.deck.matching_cards(&ids) {
            for id in ids.iter() {
                //probably return to add cloaks
                self.hidden_cards.remove(id);
            }
        }
    }
}
