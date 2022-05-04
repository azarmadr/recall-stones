use {
    rand::{
        distributions::WeightedIndex,
        prelude::*,
        seq::{index::sample, SliceRandom},
    },
    serde::{Deserialize, Serialize},
    std::{
        fmt::{Debug, Display, Formatter},
        ops::{Deref, DerefMut},
    },
};

/// Game Modes
/// Variants
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MatchRules {
    /// Pairs need only to be of same rank -- 2 == 2
    AnyColor,
    /// Pairs need to be of same rank and color -- 2red == 2red
    SameColor,
    /// Pairs need to be of same rank but color should be of opposite -- 2red == 2black
    Zebra,
    /// Pairs need to be of same rank and suite -- 2redHearts == 2redHearts
    TwoDecks,
    /// Pairs need to be of same rank and suite, cards have different backs for easy differentiation
    CheckeredDeck,
}
use MatchRules::*;
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mode {
    pub rule: MatchRules,
    pub combo: bool,
    pub full_plate: bool,
}
impl Default for Mode {
    fn default() -> Self{Self{rule:Zebra,combo:true,full_plate:true}}
}
/// Deck
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable,Default))]
#[derive(Debug, Clone)]
pub struct Deck {
    mode: Mode,
    map: Vec<u16>,
    player: (u8, u8),
    outcome: Option<u8>,
    pub opened: Vec<usize>,
}
impl Deck {
    /// Randomize couplets till max count and initialize them in the Deck
    pub fn init((count, max, _couplets): (u8, u8, u8), mode: Mode, players: u8) -> Self {
        let mut rng = rand::thread_rng();
        let mut cards = vec![2; max.into()];
        let mut dist = WeightedIndex::new(&vec![1; max.into()]).unwrap();
        for _ in 0..(2 * max - count) {
            let idx = dist.sample(&mut rng);
            cards[idx] -= 1;
            if cards[idx] == 0 {
                dist.update_weights(&[(idx, &0)]).unwrap();
            }
        }
        let map_func = |(card, count): (usize, &usize)| -> Vec<u16> {
            {
                if *count != 1 || mode.rule == AnyColor {
                    sample(&mut rng, 4, count * 2)
                        .iter()
                        .map(|x| (x * 14 + card) as u16)
                        .collect()
                } else {
                    sample(&mut rng, 4, 1)
                        .iter()
                        .flat_map(|x| match mode.rule {
                            Zebra => [x, (x + 1) % 4],
                            SameColor => [x, (x + 2) % 4],
                            TwoDecks => [x,x],
                            CheckeredDeck => [x,x+56],
                            AnyColor => unreachable!(),
                        })
                        .map(|x| (x * 14 + card) as u16)
                        .collect()
                }
            }
        };
        let mut map: Vec<u16> = cards.iter().enumerate().flat_map(map_func).collect();
        if mode.full_plate {
            map.shuffle(&mut rng);
        } else {
            map = map.iter().step_by(2).chain(map.iter().skip(1).step_by(2)).map(|&x|x).collect();
            map[0..count as usize].shuffle(&mut rng);
            map[count as usize..2*count as usize].shuffle(&mut rng);
        }
        Self {
            mode,
            map,
            player: (0, players),
            outcome: None,
            opened: vec![],
        }
    }
    #[inline]
    fn match_found(&self) -> bool {
        let l = self[self.opened[0]];
        let r = self[self.opened[1]];
        let eq = l % 14 == r % 14;
        match self.mode.rule {
            AnyColor => eq,
            Zebra => eq && (l / 14 ^ r / 14 != 2),
            SameColor => eq && l / 14 == r / 14,
            TwoDecks => l == r,
            CheckeredDeck => l % 56 == r % 56,
        }
    }

    pub fn next_player(&self) -> u8 {
        self.player.0
    }

    pub fn is_revealed(&self, mv: usize) -> bool {
        self[mv] > 127
    }
    pub fn is_available_move(&self, mv: usize) -> bool {
        !self.is_revealed(mv) && !(self.opened.contains(&mv) && self.opened.len() == 1)
    }

    pub fn play(&mut self, mv: usize) {
        assert!(self.outcome.is_none());
        assert!(
            self.is_available_move(mv),
            "{:?} is not available on {:?}",
            mv,
            self
        );
        if self.opened.len() == 2 {
            self.opened.clear();
        }

        println!(
            "played: {}, card: {}, player: {:?}",
            mv, self[mv], self.player
        );
        self.opened.push(mv);

        if self.opened.len() == 2 {
            if self.match_found() {
                let id = self.opened[0];
                self[id] += 128 * self.player.0 as u16 + 128;
                let id = self.opened[1];
                self[id] += 128 * self.player.0 as u16 + 128;
                if self.iter().all(|&x| x > 127) {
                    self.outcome = Some(self.next_player());
                }
                if !self.mode.combo {
                    self.player.0 = (self.player.0 + 1) % self.player.1;
                }
            } else {
                self.player.0 = (self.player.0 + 1) % self.player.1;
            }
        }
    }

    pub fn outcome(&self) -> Option<u8> {
        self.outcome
    }

    /*
    fn can_lose_after_move() -> bool {
        false
    }
    fn all_possible_moves() -> Self::AllMovesIterator {
        (0..56).map(|x| Coord(x)).into_internal()
    }
    */
    pub fn available_moves(&self) -> impl Iterator<Item = usize> + '_ {
        //let v:Vec<usize> =self.iter().enumerate().map(|(i,_)|i).collect(); v.iter().filter(|i|self.is_available_move(i)).collect();
        self.iter().enumerate().filter_map(|(i, _)| {
            if self.is_available_move(i) {
                Some(i)
            } else {
                None
            }
        })
        //v
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
impl Display for Deck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " mode: {:?}}}:", self.mode)?;
        let char_width = 3;
        let width = (self.len() as f32).sqrt().round() as u8;
        let line: String = (0..=(width)).into_iter().map(|_| '-').collect();
        write!(f, "{}\n|", line)?;
        let mut count = 0;
        for card in self.iter() {
            if count == width {
                count = 0;
                write!(f, "|\n|")?;
            }
            count += 1;
            write!(f, "{:char_width$}", card)?;
        }
        write!(f, "|\n{}", line)?;
        Ok(())
    }
}
impl Mode {
    pub fn desc(&self) -> String {
        format!("Rule: {}\nCombo: {}\nAccess:{}",match self.rule {
            AnyColor => "Pairs need only to be of same rank",
            SameColor => "Pairs need to be of same rank and color",
            Zebra => "Pairs need to be of same rank but color should be of opposite",
            TwoDecks => "Pairs need to be of same rank and suite",
            CheckeredDeck => "Pairs need to be of same rank and suite,\ncards have different backs for easy differentiation",
        },if self.combo {"Allowed"}else{"One Flip per turn"},if self.full_plate {"Full Plate"}else{"Half Plate"})
    }
    pub fn example(&self) -> &str {
        match self.rule {
            AnyColor => "2 == 2",
            SameColor => "2red == 2red",
            Zebra => "2red == 2black",
            TwoDecks => "2redHearts == 2redHearts",
            CheckeredDeck => "2redHearts == 2redHearts",
        }
    }
}