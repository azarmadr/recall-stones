use bevy::prelude::Component;
pub use coordinates::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Open;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Score;

/// Uncover component, indicates a covered tile that should be uncovered
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct Revealed;

mod coordinates;

/*
mod open;
mod revealed;
mod opening;
*/
