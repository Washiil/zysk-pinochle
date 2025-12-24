pub mod hand;

pub use crate::core::hand::PinochleHand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Nine, Jack, Queen, King, Ten, Ace,
}

impl Suit {
    #[inline(always)]
    pub const fn shift(self) -> u32 {
        match self {
            Suit::Spades => 0,
            Suit::Hearts => 12,
            Suit::Diamonds => 24,
            Suit::Clubs => 36,
        }
    }
}

impl Rank {
    #[inline(always)]
    pub const fn mask(self) -> u64 {
        match self {
            Rank::Nine => 0b000000000011,
            Rank::Jack => 0b000000001100,
            Rank::Queen => 0b000000110000,
            Rank::King => 0b000011000000,
            Rank::Ten => 0b001100000000,
            Rank::Ace => 0b110000000000,
        }
    }
}