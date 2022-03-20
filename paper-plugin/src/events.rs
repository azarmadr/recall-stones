use crate::components::{Coordinates, Idx};

#[derive(Debug, Copy, Clone)]
pub struct TileMarkEvent(pub Coordinates);

#[derive(Debug, Copy, Clone)]
pub struct CardFlipEvent(pub Idx);

#[derive(Debug, Copy, Clone)]
pub struct DeckCompletedEvent;
#[derive(Debug, Copy, Clone)]
pub struct BombExplosionEvent;
