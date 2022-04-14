use bevy::prelude::Component;
pub use coordinates::*;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Open;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Close;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct ScoreBoard;

/// Uncover component, indicates a covered tile that should be uncovered
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Revealed;

/// Collection specifying corresponing assets
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub enum Collection {
    Eng,
    Tel,
    Clubs,
    Diamonds,
    Spades,
    Hearts,
    Dice,
}

mod coordinates;
