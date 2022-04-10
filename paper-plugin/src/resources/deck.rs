use crate::components::Idx;
use rand::seq::{index::sample, SliceRandom};
use std::iter::repeat;
use std::ops::{Deref, DerefMut};

/// Base tile map
#[derive(Debug, Clone)]
pub struct Deck {
    count: u16,
    max: u16,
    couplets: u8,
    duel: bool,
    map: Vec<u16>,
}

impl Deck {
    /// Randomize couplets till max count and initialize them in the Deck
    pub fn init((count, max, couplets): (u16, u16, u8), duel: bool) -> Self {
        let cycles = (count as f32 / max as f32).ceil();
        let mut rng = rand::thread_rng();
        let mut map: Vec<u16> = repeat(cycles)
            .flat_map(|_| {
                sample(
                    &mut rng,
                    max.into(),
                    (count as f32 / cycles).ceil() as usize,
                )
            })
            .flat_map(|x| repeat(x as u16).take(if duel { 1 } else { couplets.into() }))
            .take(2 * count as usize)
            .collect();
        if duel {
            map = std::iter::repeat(map)
                .take(couplets.into())
                .flat_map(|mut x| {
                    x.shuffle(&mut rng);
                    x
                })
                .collect();
        } else {
            map.shuffle(&mut rng);
        }
        Self {
            count,
            max,
            couplets,
            duel,
            map,
        }
    }

    #[cfg(feature = "debug")]
    pub fn console_output(&self) -> String {
        let mut buffer = format!(
            "Deck {{count: {}, max: {}, couplets: {}, duel: {}}}:\n",
            self.count, self.max, self.couplets, self.duel
        );
        let char_width = self.max.to_string().len() + 1;
        let line: String = (0..=(self.width())).into_iter().map(|_| '-').collect();
        buffer = format!("{}{}\n|", buffer, line);
        let mut count = 0;
        for card in self.iter() {
            if count == self.width() {
                count = 0;
                buffer = format!("{}|\n|", buffer);
            }
            count += 1;
            buffer = format!("{}{:char_width$}", buffer, card);
        }
        format!("{}|\n{}", buffer, line)
    }

    #[inline]
    #[must_use]
    pub fn matching_cards(&self, ids: Vec<Idx>) -> Vec<Idx> {
        let mut map = std::collections::HashMap::new();
        for Idx(e) in ids {
            match self.map.get(e as usize) {
                Some(c) => map.entry(c).or_insert(vec![]).push(Idx(e)),
                None => (),
            }
        }
        map.into_values()
            .find(|x| x.len() == self.couplets as usize)
            .unwrap_or_default()
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
        let len = self.map.len() / if self.duel { self.couplets.into() } else { 1 };
        (len as f32).sqrt().round() as u16 * if self.duel { self.couplets as u16 } else { 1 }
    }

    // Getter for `height`
    // needs additional params
    #[inline]
    #[must_use]
    pub fn height(&self) -> u16 {
        let len = self.map.len(); // if self.duel { self.couplets.into() } else { 1 };
        (len as f32 / self.width() as f32).ceil() as u16
    }
}

impl Deref for Deck {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Deck {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
