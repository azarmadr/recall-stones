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

const CARD_MASK:u8 = (1<<7) -1;
const OWN_MASK:u16 = 3<<7;

/// Game Modes
/// Variants
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mode {
    pub rule: MatchRules,
    pub combo: bool,
    pub full_plate: bool,
    pub duel: bool,
}
impl Default for Mode {
    fn default() -> Self {
        Self {
            rule: Zebra,
            combo: true,
            full_plate: true,
            duel: true,
        }
    }
}
/// Deck
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable, Default))]
#[derive(Debug, Clone)]
pub struct Deck {
    mode: Mode,
    /// Map of cards, where each entry is 
    /// card with its value in 7 bits,
    /// player who opened it in 2 bits,
    /// number of times opened in 7 bits.
    map: Vec<u16>,
    players: (u8, u8),
    outcome: Option<u8>,
    pub opened: Vec<usize>,
    pub scores: Vec<u16>,
}
impl Deck {
    /// Randomize couplets till max count and initialize them in the Deck
    pub fn init((count, max): (u8, u8), mode: Mode, players: u8) -> Self {
        let mut rng = rand::thread_rng();
        let suites = match mode.rule {
            TwoDecks | CheckeredDeck => 4,
            _ => 2,
        };
        let mut cards = vec![suites as usize; max.into()];
        let mut dist = WeightedIndex::new(&vec![1; max.into()]).unwrap();
        for _ in 0..(suites * max - count) {
            let idx = dist.sample(&mut rng);
            cards[idx] -= 1;
            if cards[idx] == 0 {
                dist.update_weights(&[(idx, &0)]).unwrap();
            }
        }
        let map_func = |(card, count): (usize, &usize)| -> Vec<u16> {
            {
                match mode.rule {
                    AnyColor => sample(&mut rng, 4, count * 2)
                        .iter()
                        .map(|x| (x * 14 + card) as u16)
                        .collect(),
                    SameColor | Zebra => sample(&mut rng, 4, 1)
                        .iter()
                        .flat_map(|x| {
                            (0..count * 2)
                                .map(|i| {
                                    if mode.rule == Zebra {
                                        (i + x) % 4
                                    } else {
                                        i % 2 * 2 + if (i < 2) ^ (x < 2) { 0 } else { 1 }
                                    }
                                })
                                .collect::<Vec<usize>>()
                        })
                        .map(|x| (x * 14 + card) as u16)
                        .collect(),
                    TwoDecks | CheckeredDeck => sample(&mut rng, 4, *count)
                        .iter()
                        .flat_map(|x| [x, if mode.rule == TwoDecks { x } else { x + 4 }])
                        .map(|x| (x * 14 + card) as u16)
                        .collect(),
                }
            }
        };
        let mut map: Vec<u16> = cards.iter().enumerate().flat_map(map_func).collect();
        if mode.full_plate {
            map.shuffle(&mut rng);
        } else {
            map = map
                .iter()
                .step_by(2)
                .chain(map.iter().skip(1).step_by(2))
                .copied()
                .collect();
            map[0..count as usize].shuffle(&mut rng);
            map[count as usize..2 * count as usize].shuffle(&mut rng);
        }
        Self {
            mode,
            map,
            players: (0, players),
            outcome: None,
            opened: vec![],
            scores: vec![0;players as usize],
        }
    }
    #[inline]
    fn match_found(&self) -> bool {
        let l = self.get_card(self.opened[0]);
        let r = self.get_card(self.opened[1]);
        // println!("{l} {r}");
        let eq = l % 14 == r % 14;
        match self.mode.rule {
            AnyColor => eq,
            Zebra => eq && ((l / 14) ^ (r / 14) != 2),
            SameColor => eq && l / 14 % 2 == r / 14 % 2,
            TwoDecks => l == r,
            CheckeredDeck => l % 56 == r % 56,
        }
    }

    pub fn next_player(&self) -> u8 {
        (self.players.0 + 1) % self.players.1
    }

    pub fn is_revealed(&self, mv: usize) -> bool {
        self.get_owner(mv) > 0
    }
    pub fn is_available_move(&self, mv: usize) -> bool {
        !(self.is_revealed(mv) || self.opened.contains(&mv) && self.opened.len() == 1)
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

        // println!("played: {mv}, card: {}, players: {:?}",self[mv],self.players);
        self.opened.push(mv);
        self[mv] += 1<<9;

        if self.opened.len() == 2 {
            if self.match_found() {
                let player = (self.player() as u16 + 1) << 7; 
                let pmv = self.opened[0];
                self[pmv] += player;
                self[mv]  += player;
                let outcome = self.iter().all(|&x| (x & 3 <<7) > 0);
                if outcome || self.get_count(mv) > 1 {
                    let score = self.get_count(pmv) + self.get_count(mv);
                    let player = self.player() as usize;
                    self.scores[player] += score as u16;
                }
                if outcome {
                    let s0 = self.scores[0];
                    let s1 = self.scores[1];
                    self.outcome = Some(if s0 > s1 {
                        0
                    } else {1});
                }
                if !self.mode.combo {
                    self.players.0 = self.next_player();
                }
            } else {
                self.players.0 = self.next_player();
            }
        }
    }

    pub fn player(&self) -> u8 {
        self.players.0
    }

    pub fn outcome(&self) -> Option<u8> {
        self.outcome
    }

    fn _get(&self, idx: usize) -> u16 {
        self[idx]
    }
    pub fn get_owner(&self, idx: usize) -> u8 {
        ((self[idx] & OWN_MASK) >> 7)as u8
    }
    pub fn get_card(&self, idx: usize) -> u8 {
        self[idx] as u8 & CARD_MASK
    }
    pub fn get_count(&self, idx: usize) -> u8 {
        (self[idx] >> 9) as u8
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
