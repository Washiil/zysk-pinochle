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