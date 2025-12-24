use crate::core::{Suit, Rank};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinochleHand {
    pub spades: u16,
    pub hearts: u16,
    pub diamonds: u16,
    pub clubs: u16,
}

impl PinochleHand {
    pub fn new() -> Self {
        Self::default()
    }

    // Add high-performance bitwise methods here
    pub fn add_card(&mut self, suit: Suit, rank: Rank) {
        let bit = 1 << (rank as u8);
        match suit {
            Suit::Spades => self.spades |= bit,
            Suit::Hearts => self.hearts |= bit,
            Suit::Diamonds => self.diamonds |= bit,
            Suit::Clubs => self.clubs |= bit,
        }
    }
}