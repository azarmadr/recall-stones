use bevy::prelude::*;

/// Button action type
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component)]
pub enum ButtonAction {
    //Clear,
    LevelUp,
    LevelDown,
    CoupletUp,
    CoupletDown,
    Generate,
}

#[derive(Default, Debug)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}
