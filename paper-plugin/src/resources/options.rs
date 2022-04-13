use crate::components::{Collection, Collection::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::*;

/// Card size options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardSize {
    /// Fixed card size
    Fixed(f32),
    /// Window adaptative card size
    Adaptive { min: f32, max: f32 },
}

/// Board position customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardPosition {
    /// Centered board
    Centered { offset: Vec3 },
    /// Custom position
    Custom(Vec3),
}

/// Game Mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    AnyColor,
    SameColor,
    Zebra,
    TwoDecks,
    OneFlip,
    TwoDecksDuel,
    DoubleDeckerCheckerboard,
    Fancy,
    Spaghetti,
    Pexeso,
}
use Mode::*;

/// Board generation options. Must be used as a resource
// We use serde to allow saving option presets and loading them at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOptions {
    /// Baard world position
    pub position: BoardPosition,
    /// Card world size
    pub card_size: CardSize,
    /// Padding between cards
    pub card_padding: f32,
    /// Various collection from which cards are generated
    pub collections: Vec<Collection>,
    /// Game Mode
    pub mode: Mode,
    pub level: u8,
    pub couplets: u8,
}
impl Default for CardSize {
    fn default() -> Self {
        Self::Adaptive {
            min: 10.0,
            max: 50.0,
        }
    }
}
impl Default for BoardPosition {
    fn default() -> Self {
        Self::Centered {
            offset: Default::default(),
        }
    }
}
impl Default for BoardOptions {
    fn default() -> Self {
        Self {
            level: 0,
            couplets: 2,
            position: Default::default(),
            card_size: Default::default(),
            card_padding: 3.,
            collections: vec![
                Clubs, Spades, Hearts, Diamonds, //Collection::Tel, Collection::Eng
            ],
            //mode: AnyColor,
            mode: Zebra,
        }
    }
}
impl BoardOptions {
    /// Computes a card size that matches the window according to the card map size
    pub fn adaptative_card_size(&self, window: (f32, f32), (width, height): (u16, u16)) -> f32 {
        match self.card_size {
            CardSize::Fixed(v) => v,
            CardSize::Adaptive { min, max } => {
                let max_width = window.0 / width as f32;
                let max_heigth = window.1 / height as f32;
                max_width.min(max_heigth).clamp(min, max)
            }
        }
    }
    pub fn col_is_suites(&self) -> bool {
        self.collections.contains(&Clubs)
            || self.collections.contains(&Spades)
            || self.collections.contains(&Hearts)
            || self.collections.contains(&Diamonds)
    }
    pub fn deck_params(&self) -> (u16, u16, u8) {
        let (deck_size, suite_size, ct_jump, mx_jump): (u16, u16, u16, u16) = match self.mode {
            AnyColor => (3, 4, 5, 2),                  //pairs 28,      uniq 14
            SameColor | Zebra => (3, 8, 5, 4),         //pairs 28,       uniq 28
            TwoDecks | TwoDecksDuel => (6, 16, 10, 8), //pairs & uniq 56
            _ => (3, 4, 5, 2),
        };
        (
            deck_size + self.level as u16 * ct_jump,
            suite_size + self.level as u16 * mx_jump,
            self.couplets,
        )
    }
    pub fn level_up(&mut self) {
        self.level = min(5, self.level + 1);
    }
    pub fn level_down(&mut self) {
        self.level = self.level.saturating_sub(1);
    }
    pub fn to_string(&self) -> String {
        format!(
            "Level: {}, Couplets: {}, Mode: {:?}",
            self.level, self.couplets, self.mode
        )
    }
}
/*
fn change(x: u16, dir: bool, jump: u16, ul: u16, ll: u16) -> u16 {
    if dir {
        min(x + jump, ul)
    } else {
        max(x - jump, ll)
    }
}
*/
