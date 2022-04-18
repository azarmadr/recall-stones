use bevy::prelude::*;
pub use coordinates::*;

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

mod coordinates;
