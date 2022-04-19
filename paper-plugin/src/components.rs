use {
    bevy::prelude::*,
    duplicate::*,
    enum_dispatch::enum_dispatch,
    std::{
        fmt::{self, Display, Formatter},
        ops::{Deref, DerefMut},
    },
};

duplicate! {[
    component_type  comment;
    [Open]          [Open];
    [Close]         [Close];
    [ScoreBoard]    [ScoreBoard];
    [Revealed]      [Revealed];
    [Turn]          [Turn];
]
    ///comment
    #[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
    #[derive(Debug, Copy, Clone, Component)]
    #[component(storage="SparseSet")]
    pub struct component_type;
}

duplicate! {[component_type; [Idx]; [Flesh]; [Bolts];]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Copy, Clone, Debug)]
pub struct component_type(pub u8, pub u8);
impl component_type {
    pub fn from2d(x: u8, y: u8, width: u8) -> Self {
        component_type(x + width * y, 0)
    }
}
impl Deref for component_type {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}
impl DerefMut for component_type {
    fn deref_mut(&mut self) -> &mut u8 {
        &mut self.1
    }
}
impl Display for component_type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let component_type(v, t) = self;
        write!(f, "id: {} {}", v, t)
    }
}
}

#[enum_dispatch]
pub trait PlayerOps {
    fn inc_turn(&mut self);
}
#[duplicate_item(pl; [Flesh]; [Bolts];)]
impl PlayerOps for pl {
    fn inc_turn(&mut self) {
        **self += 1;
    }
}
#[enum_dispatch(PlayerOps, Deref, DerefMut)]
#[derive(Debug, Component)]
pub enum Player {
    Flesh(Flesh),
    Bolts(Bolts),
}
impl Player {
    pub fn deref(&self) -> u8 {
        match self {
            Player::Flesh(x) => **x,
            Player::Bolts(x) => **x,
        }
    }
}
