//use crate::components::Idx;
use rand::seq::{index::sample, SliceRandom};
use std::iter::repeat;
use std::ops::{Deref, DerefMut};

use super::Mode;

/// Base tile map
#[derive(Debug, Clone)]
pub struct Deck {
    count: u8,
    max: u8,
    couplets: u8,
    mode: Mode,
    map: Vec<u8>,
}
impl Deck {
    /// Randomize couplets till max count and initialize them in the Deck
    pub fn init((count, max, couplets): (u8, u8, u8), mode: Mode) -> Self {
        let sample_max = if mode == Mode::Zebra { max / 2 } else { max };
        let cycles = (count as f32 / sample_max as f32).ceil();
        println!(
            "sample_max:{};cycles:{},count/cycles:{}",
            sample_max,
            cycles,
            (count as f32 / cycles).ceil() as usize
        );
        let mut rng = rand::thread_rng();
        let mut map: Vec<u8> = repeat(cycles)
            .flat_map(|_| {
                sample(
                    &mut rng,
                    sample_max.into(),
                    (count as f32 / cycles).ceil() as usize,
                )
            })
            .flat_map(|x| {
                repeat(x as u8).take(match mode {
                    Mode::HalfPlate | Mode::Zebra => 1,
                    _ => couplets.into(),
                })
            })
            .take(
                count as usize
                    * match mode {
                        Mode::HalfPlate | Mode::Zebra => 1,
                        _ => 2,
                    },
            )
            .collect();
        match mode {
            Mode::HalfPlate => {
                map = std::iter::repeat(map)
                    .take(couplets.into())
                    .flat_map(|mut x| {
                        x.shuffle(&mut rng);
                        x
                    })
                    .collect();
            }
            Mode::Zebra => {
                map = map.iter().flat_map(|&x| [x, x + max / 2]).collect();
                map.shuffle(&mut rng);
            }
            _ => {
                map.shuffle(&mut rng);
            }
        }
        Self {
            count,
            max,
            couplets,
            mode,
            map,
        }
    }
    #[cfg(feature = "debug")]
    pub fn console_output(&self) -> String {
        let mut buffer = format!(
            "Deck {{count: {}, max: {}, couplets: {}, mode: {:?}}}:\n",
            self.count, self.max, self.couplets, self.mode
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
    pub fn get_val(&self, id: u8) -> Option<&u8> {
        self.get(id as usize)
    }
    #[inline]
    #[must_use]
    pub fn matching_cards(&self, ids: &Vec<u8>) -> bool {
        if self.couplets > 2 || self.mode != Mode::Zebra {
            let f_card = self.get_val(*ids.first().unwrap()).unwrap();
            ids.iter().all(|&x| self.get_val(x).unwrap() == f_card)
        } else {
            if let (Some(&l), Some(&r)) = match &ids[..] {
                &[first, second, ..] => (self.get_val(first), self.get_val(second)),
                _ => unreachable!(),
            } {
                if l == r + self.max / 2 || r == l + self.max / 2 {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
    /*
    // Getter for `max`
    #[inline]
    #[must_use]
    pub fn max(&self) -> u8 {
        self.max
    }
    */
    // Getter for `count`
    #[inline]
    #[must_use]
    pub fn count(&self) -> u8 {
        self.count
    }
    // Getter for `couplets`
    #[inline]
    #[must_use]
    pub fn couplets(&self) -> u8 {
        self.couplets
    }
    // Getter for `width`
    // needs additional params
    #[inline]
    #[must_use]
    pub fn width(&self) -> u8 {
        let duel = self.mode == Mode::HalfPlate;
        let len = self.map.len() / if duel { self.couplets.into() } else { 1 };
        (len as f32).sqrt().round() as u8 * if duel { self.couplets as u8 } else { 1 }
    }
    // Getter for `height`
    // needs additional params
    #[inline]
    #[must_use]
    pub fn height(&self) -> u8 {
        let len = self.map.len(); // if self.duel { self.couplets.into() } else { 1 };
        (len as f32 / self.width() as f32).ceil() as u8
    }
}
impl Deref for Deck {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for Deck {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
