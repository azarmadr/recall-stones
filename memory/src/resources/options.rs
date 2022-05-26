use {
    super::{MatchRules::*, Mode},
    crate::components::*,
    bevy::prelude::*,
    rand::{distributions::WeightedIndex, prelude::*},
    serde::{Deserialize, Serialize},
};

/// Card size options
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
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
/// Board generation options. Must be used as a resource
// We use serde to allow saving option presets and loading them at runtime
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGOpts {
    /// Card world size
    pub card_size: CardSize,
    /// Padding between cards
    pub card_padding: f32,
    /// Game Mode
    pub mode: Mode,
    #[cfg_attr(feature = "debug", inspectable(min = 0, max = 5))]
    pub level: u8,
    //#[cfg_attr(feature="debug",inspectable(min = (1,0), max = (2,1)))]
    pub players: (u8, u8),
}
impl MemoryGOpts {
    fn default() -> Self {
        Self {
            level: 0,
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
    pub fn deck_params(&self) -> (u8, u8) {
        let (deck_size, ct_jump): (u8, u8) = match self.mode.rule {
            TwoDecks | CheckeredDeck => (6, 10), //pairs & uniq 56
            _ => (3, 5),
        };
        (deck_size + self.level * ct_jump, 4 + self.level * 2)
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
impl FromWorld for MemoryGOpts {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let windows = world.get_resource::<Windows>().unwrap();
        let window = windows.get_primary().unwrap();
        MemoryGOpts {
            card_size: CardSize::Adaptive {
                min: 10.0,
                max: 30.0,
                window: (window.width(), window.height()),
            },
            ..MemoryGOpts::default()
        }
    }
}
