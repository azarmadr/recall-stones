use super::{Collection, Collection::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::*;

/// Card size options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardSize {
    /// Fixed card size
    Fixed(f32),
    /// Window adaptative card size
    Adaptive {
        min: f32,
        max: f32,
        window: (f32, f32),
    },
}
impl Default for CardSize {
    fn default() -> Self {
        Self::Adaptive {
            min: 10.0,
            max: 50.0,
            window: (720., 480.),
        }
    }
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
    /// Only once flip each turn
    OneFlip,
    /// Each turn can only flip from your half side of the deck
    /// After each turn, roles are changed (first becomes second)
    HalfPlate,
    /// Deck arrangement - Circles or Trianles or any other
    Fancy,
    /// Deck arrangement - Duh Duh!
    Spaghetti,
    /// With numbers
    Pexeso,
}
use Mode::*;
impl Mode {
    pub fn desc(&self) -> &str {
        match self {
            AnyColor => "Pairs need only to be of same rank",
            SameColor => "Pairs need to be of same rank and color",
            Zebra => "Pairs need to be of same rank but color should be of opposite",
            TwoDecks => "Pairs need to be of same rank and suite",
            CheckeredDeck => "Pairs need to be of same rank and suite,\ncards have different backs for easy differentiation",
            OneFlip => "Only once flip each turn",
            HalfPlate => "Each turn can only flip from your half side of the deck.\nAfter each turn, roles are changed",
            Fancy => "Deck arrangement - Circles or Trianles or any other",
            Spaghetti => "Deck arrangement - Duh Doy!",
            Pexeso => "With numbers",
        }
    }
    pub fn example(&self) -> &str {
        match self {
            AnyColor => "2 == 2",
            SameColor => "2red == 2red",
            Zebra => "2red == 2black",
            TwoDecks => "2redHearts == 2redHearts",
            CheckeredDeck => "2redHearts == 2redHearts",
            HalfPlate => "Turn 1: first player starts; Turn 2: second player starts",
            _ => "",
        }
    }
}
/// Board generation options. Must be used as a resource
// We use serde to allow saving option presets and loading them at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOptions {
    /// Board world position
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
    pub players: (u8, u8),
}
impl Default for BoardPosition {
    fn default() -> Self {
        Self::Centered {
            offset: Default::default(),
            //offset: Vec3::new(0., 25., 0.),
        }
    }
}
impl BoardOptions {
    fn default() -> Self {
        Self {
            level: 3,
            couplets: 2,
            position: Default::default(),
            card_size: Default::default(),
            card_padding: 3.,
            collections: vec![
                Clubs, Spades, Hearts, Diamonds, //Collection::Tel, Collection::Eng
            ],
            //mode: AnyColor,
            mode: Zebra,
            players: (1, 0),
        }
    }
    /// Computes a card size that matches the window according to the card map size
    pub fn card_size(&self, width: u8, height: u8) -> f32 {
        match self.card_size {
            CardSize::Fixed(v) => v,
            CardSize::Adaptive { min, max, window } => {
                let max_width = window.0 / width as f32;
                let max_heigth = window.1 / height as f32;
                max_width.min(max_heigth).clamp(min, max)
            }
        }
    }
    pub fn board_position(&self, board_size: Vec2) -> Vec3 {
        match self.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        }
    }
    pub fn col_is_suites(&self) -> bool {
        self.collections.contains(&Clubs)
            || self.collections.contains(&Spades)
            || self.collections.contains(&Hearts)
            || self.collections.contains(&Diamonds)
    }
    pub fn deck_params(&self) -> (u8, u8, u8) {
        let (deck_size, suite_size, ct_jump, mx_jump): (u8, u8, u8, u8) = match self.mode {
            AnyColor => (3, 4, 5, 2),          //pairs 28,      uniq 14
            SameColor | Zebra => (3, 8, 5, 4), //pairs 28,       uniq 28
            TwoDecks => (6, 16, 10, 8),        //pairs & uniq 56
            _ => (3, 4, 5, 2),
        };
        (
            deck_size + self.level * ct_jump,
            suite_size + self.level * mx_jump,
            self.couplets,
        )
    }
    pub fn to_string(&self) -> String {
        format!(
            "Level: {}, Mode: {:?}, Humans: {}, Bots: {}",
            self.level, self.mode, self.players.0, self.players.1
        )
    }
}
impl FromWorld for BoardOptions {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let windows = world.get_resource::<Windows>().unwrap();
        let window = windows.get_primary().unwrap();
        BoardOptions {
            card_size: CardSize::Adaptive {
                min: 10.0,
                max: 50.0,
                window: (window.width(), window.height()),
            },
            ..BoardOptions::default()
        }
    }
}
