use crate::components::Idx;
use crate::resources::card::Card;
use rand::seq::{index::sample, SliceRandom};
use std::ops::{Deref, DerefMut};

/// Base tile map
#[derive(Debug, Clone)]
pub struct Deck {
    count: u16,
    max: u16,
    couplets: u8,
    map: Vec<Card>,
}

impl Deck {
    /// Randomize couplets till max count and initialize them in the Deck
    pub fn init((count, max, couplets): (u16, u16, u8)) -> Self {
        let mut rng = rand::thread_rng();
        let mut map: Vec<Card> = sample(&mut rng, max.into(), count.into())
            .iter()
            .flat_map(|x| std::iter::repeat(Card(x as u16)).take(couplets.into()))
            .collect();
        map.shuffle(&mut rng);
        Self {
            count,
            max,
            couplets,
            map,
        }
    }

    pub fn matching_cards(&self, ids: Vec<Idx>) -> Vec<Idx> {
        let mut map = std::collections::HashMap::new();
        for Idx(e) in ids {
            match self.map.get(e as usize) {
                Some(c) => map.entry(c.val()).or_insert(vec![]).push(Idx(e)),
                None => (),
            }
        }
        map.into_values()
            .find(|x| x.len() == self.couplets as usize)
            .unwrap_or_default()
    }

    #[cfg(feature = "debug")]
    pub fn console_output(&self) -> String {
        let mut buffer = format!("Deck of {} cards from 0 to {}:\n", self.count, self.max);
        let char_width = self.max.to_string().len() + 1;
        println!("{:>3}", char_width);
        let line: String = (0..=(self.width())).into_iter().map(|_| '-').collect();
        buffer = format!("{}{}\n|", buffer, line);
        let mut count = 0;
        for card in self.iter() {
            if count == self.width() {
                count = 0;
                buffer = format!("{}|\n|", buffer);
            }
            count += 1;
            buffer = format!("{}{:char_width$}", buffer, card.console_output());
        }
        format!("{}|\n{}", buffer, line)
    }

    // Getter for `max`
    #[inline]
    #[must_use]
    pub fn max(&self) -> u16 {
        self.max
    }

    // Getter for `count`
    #[inline]
    #[must_use]
    pub fn count(&self) -> u16 {
        self.count
    }

    // Getter for `open`
    #[inline]
    #[must_use]
    pub fn couplets(&self) -> u8 {
        self.couplets
    }

    // Getter for `width`
    // needs additional params
    #[inline]
    #[must_use]
    pub fn width(&self) -> u16 {
        (self.map.len() as f32).sqrt().round() as u16
    }

    // Getter for `height`
    // needs additional params
    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        (self.map.len() as f32 / self.width() as f32).ceil() as u16
    }
}

impl Deref for Deck {
    type Target = Vec<Card>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Deck {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
