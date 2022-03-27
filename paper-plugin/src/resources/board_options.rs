use bevy::prelude::Vec3;
use std::collections::HashSet;
use crate::resources::Collection;
use serde::{Deserialize, Serialize};

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

/// Board generation options. Must be used as a resource
// We use serde to allow saving option presets and loading them at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOptions {
    /// Tuple of Deck size, max card value, couplet length
    pub deck_params: (u16, u16, u8),
    /// Baard world position
    pub position: BoardPosition,
    /// Card world size
    pub card_size: CardSize,
    /// Padding between cards
    pub card_padding: f32,
    /// Various collection from which cards are generated
    pub collections: HashSet<Collection>,
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
            deck_params: (4, 30, 2),
            position: Default::default(),
            card_size: Default::default(),
            card_padding: 3.,
            collections: HashSet::from([Collection::Tel,Collection::Eng]),
        }
    }
}
