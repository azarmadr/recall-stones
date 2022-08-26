use {
    bevy::prelude::*,
    duplicate::*,
    enum_dispatch::enum_dispatch,
    std::{
        fmt::{self, Display, Formatter},
        ops::Deref,
    },
};

duplicate! {[
    component_type  comment;
    [Open]          [Open];
    [Close]         [Close];
    [Revealed]      [Revealed];
]
    ///comment
    #[derive(Debug, Copy, Clone, Component)]
    #[component(storage="SparseSet")]
    pub struct component_type;
}

duplicate! {[component t; [Idx] [usize]; [Flesh] [u8]; [Bolts] [u8];]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Copy, Clone, Debug)]
pub struct component(pub t, pub u8);
impl Deref for component {
    type Target = t;
    fn deref(&self) -> &t {
        &self.0
    }
}
impl Display for component {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let component(v, t) = self;
        write!(f, "component: {} {}", v, t)
    }
}
}
impl Idx {
    pub fn from2d(x: usize, y: usize, width: usize) -> Self {
        Idx(x + width * y, 0)
    }
}

#[enum_dispatch]
pub trait PlayerOps {
    fn inc_turn(&mut self);
}
#[duplicate_item(pl; [Flesh]; [Bolts];)]
impl PlayerOps for pl {
    fn inc_turn(&mut self) {
        self.1 += 1;
    }
}
#[enum_dispatch(PlayerOps, Deref)]
#[derive(Debug, Component, Copy, Clone)]
pub enum Player {
    Flesh(Flesh),
    Bolts(Bolts),
}
impl Player {
    pub fn deref(&self) -> (u8, u8) {
        match self {
            Player::Flesh(Flesh(x, y)) => (*x, *y),
            Player::Bolts(Bolts(x, y)) => (*x, *y),
        }
    }
    pub fn id(&self) -> u8 {
        match self {
            Player::Flesh(Flesh(x,_)) => *x,
            Player::Bolts(Bolts(x,_)) => *x,
        }
    }
    pub fn is_bot(&self) -> bool {
        match self {
            Player::Flesh(_) => false,
            Player::Bolts(_) => true,
        }
    }
}
