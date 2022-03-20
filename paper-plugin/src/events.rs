use crate::components::Idx;

#[derive(Debug, Copy, Clone)]
pub struct CardFlipEvent(pub Idx);

#[derive(Debug, Copy, Clone)]
pub struct DeckCompletedEvent;
