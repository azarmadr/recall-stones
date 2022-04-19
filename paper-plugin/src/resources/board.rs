use crate::components::Idx;
use crate::deck::Deck;
use crate::player::*;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Board {
    pub deck: Deck,
    pub card_size: f32,
    pub board_position: Vec3,
    pub hidden_cards: HashMap<u8, Entity>,
    pub entity: Entity,
    pub player_panels: Vec<Panel>,
}
impl Board {
    /// Retrieves a covered tile entity
    #[inline]
    #[must_use]
    pub fn flip_card(&self, id: u8) -> Option<&Entity> {
        self.hidden_cards.get(&id)
    }
    #[inline]
    #[must_use]
    pub fn is_revealed(&self, id: &Idx) -> bool {
        !self.hidden_cards.contains_key(id)
    }
    /*
    #[inline]
    #[must_use]
    pub fn opened_count(&self, id: &Idx) -> u8 {
        match self.opened_count.get(id) {
            Some(v) => *v,
            None => 1,
        }
    }
    */
    #[inline]
    #[must_use]
    pub fn get_card_val(&self, id: &Idx) -> Option<&u8> {
        self.deck.get(**id as usize)
    }
    /// reveal all the matching cards
    #[inline]
    pub fn reveal_matching_cards(&mut self, ids: Vec<u8>) {
        /*
        for e in ids.iter() {
            let count = self.opened_count.entry(*e).or_insert(0);
            *count += 1;
        }
        */
        if self.deck.matching_cards(&ids) {
            for id in ids.iter() {
                //probably return to add cloaks
                self.hidden_cards.remove(id);
            }
        }
    }
    /// Translates a mouse position to board coordinates
    pub fn mouse_position(&self, position: Vec2) -> Option<u8> {
        if !self.in_bounds(position) {
            return None;
        }
        let coordinates = (position - self.board_position.xy()) / self.card_size;
        Some(coordinates.x as u8 + coordinates.y as u8 * self.deck.width() as u8)
    }
    #[inline]
    #[must_use]
    fn in_bounds(&self, position: Vec2) -> bool {
        self.board_position.x <= position.x
            && position.x <= self.board_position.x + self.card_size * self.deck.width() as f32
            && self.board_position.y <= position.y
            && position.y <= self.board_position.y + self.card_size * self.deck.height() as f32
    }
    //pub fn next_player(&mut self) -> Entity { }
}
