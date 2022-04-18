use crate::components::Idx;
use crate::player::*;
use crate::deck::Deck;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Board {
    pub deck: Deck,
    pub card_size: f32,
    pub board_position: Vec3,
    pub hidden_cards: HashMap<Idx, Entity>,
    pub opened_count: HashMap<Idx, u16>,
    pub entity: Entity,
    pub current_player: u8,
    pub player_panels: Vec<Panel>,
}
impl Board {
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
    /// Translates a mouse position to board coordinates
    pub fn mouse_position(&self, position: Vec2) -> Option<Idx> {
        if !self.in_bounds(position) {
            return None;
        }
        let coordinates = (position - self.board_position.xy()) / self.card_size;
        Some(Idx(
            coordinates.x as u16 + self.deck.width() * coordinates.y as u16
        ))
    }
    #[inline]
    pub fn inc_player_turn(&mut self) {
        let player = self.current_player;
        self.player_panels[player as usize].turns += 1;
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
