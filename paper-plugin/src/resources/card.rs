/// Enum describing a Minesweeper tile
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Card(pub u16);

impl Card {
    #[inline]
    #[must_use]
    pub fn val(&self) -> u16 {
        let Card(v) = *self;
        v
    }

    #[cfg(feature = "debug")]
    pub fn console_output(&self) -> String {
        format!("{}", self.val())
    }
}
