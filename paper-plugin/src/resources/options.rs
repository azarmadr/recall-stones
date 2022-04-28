use {
    super::{MatchRules::*, Mode},
    crate::components::*,
    bevy::prelude::*,
    rand::{distributions::WeightedIndex, prelude::*},
    serde::{Deserialize, Serialize},
};

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
            max: 30.0,
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
            level: 0,
            couplets: 2,
            position: Default::default(),
            card_size: Default::default(),
            card_padding: 3.,
            mode: Mode {
                rule: Zebra,
                combo: true,
                full_plate: true,
            },
            players: (1, 1),
        }
    }
    /// Computes a card size that matches the window according to the card map size
    pub fn card_size(&self, width: f32, height: f32) -> f32 {
        match self.card_size {
            CardSize::Fixed(v) => v,
            CardSize::Adaptive { min, max, window } => {
                let max_width = window.0 / width;
                let max_heigth = window.1 / height;
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
    pub fn deck_params(&self) -> (u8, u8, u8) {
        let (deck_size, ct_jump): (u8, u8) = match self.mode.rule {
            TwoDecks => (6, 10), //pairs & uniq 56
            _ => (3, 5),
        };
        (
            deck_size + self.level * ct_jump,
            4 + self.level * 2,
            self.couplets,
        )
    }
    pub fn to_string(&self) -> String {
        format!(
            "Level: {}, Mode: {:?}, Humans: {}, Bots: {}",
            self.level, self.mode, self.players.0, self.players.1
        )
    }
    pub fn create_players(&self) -> Vec<Player> {
        let mut weights = [self.players.0, self.players.1];
        let mut players = vec![];
        let mut rng = thread_rng();
        let mut idx = 0u8;
        while !weights.iter().all(|&x| x == 0) {
            let dist = WeightedIndex::new(&weights).unwrap();
            let choice = if idx == 0 { 0 } else { dist.sample(&mut rng) };
            weights[choice] -= 1;
            players.push(if choice == 1 {
                Player::Bolts(Bolts(idx, 0))
            } else {
                Player::Flesh(Flesh(idx, 0))
            });
            idx += 1;
        }
        players
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
                max: 30.0,
                window: (window.width(), window.height()),
            },
            ..BoardOptions::default()
        }
    }
}
