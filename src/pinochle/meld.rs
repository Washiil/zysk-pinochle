use std::cmp::min;
use crate::core::{Card, Rank, Suit};

/// Check for a Marriage in a specific suit (K + Q)
fn has_marriage(cards: u64) -> bool {
    let king_mask = Rank::King.mask() & cards;
    let queen_mask = Rank::Queen.mask() & cards;

    king_mask.count_ones() > 0 && queen_mask.count_ones() > 0
}

#[inline(always)]
fn count_rank_around(hand: u64, rank: Rank) -> u8 {
    let mask = rank.mask();
    let cards = hand & mask;

    let s = (cards & Suit::Spades.bitmask()).count_ones();
    let h = (cards & Suit::Hearts.bitmask()).count_ones();
    let d = (cards & Suit::Diamonds.bitmask()).count_ones();
    let c = (cards & Suit::Clubs.bitmask()).count_ones();

    s.min(h).min(d).min(c) as u8
}

pub fn compute_meld(hand: u64, trump: Option<Suit>) -> u8 {
    let mut meld = 0u8;

    let trump_bitset: u8 = match trump {
        Some(s) => 1 << (s as u8),
        None => 0,
    };

    for &suit in &Suit::ALL {
        let suit_id = suit as u8;

        let suit_shift = suit_id as u64 * 12;
        let kings = (hand >> (suit_shift + 6)) & 0b11;
        let queens = (hand >> (suit_shift + 4)) & 0b11;

        // Count how many marriages (min of K and Q counts)
        let count = min(kings.count_ones(), queens.count_ones()) as u8;

        // Branchless Logic
        let is_trump_flag = (trump_bitset >> suit_id) & 1;
        let multiplier = 2 + (is_trump_flag * 2);

        meld += count * multiplier;
    }

    meld
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPADES_MARRIAGE: u64 = Card::new(Suit::Spades, Rank::King, 0).bitmask() | Card::new(Suit::Spades, Rank::Queen, 0).bitmask();
    const ACES_AROUND: u64 = Card::new(Suit::Spades, Rank::Ace, 0).bitmask() | Card::new(Suit::Clubs, Rank::Ace, 0).bitmask() | Card::new(Suit::Hearts, Rank::Ace, 0).bitmask() | Card::new(Suit::Diamonds, Rank::Ace, 0).bitmask();
    const ACES_AROUND_AROUND: u64 = Card::new(Suit::Spades, Rank::Ace, 1).bitmask() | Card::new(Suit::Clubs, Rank::Ace, 1).bitmask() | Card::new(Suit::Hearts, Rank::Ace, 1).bitmask() | Card::new(Suit::Diamonds, Rank::Ace, 1).bitmask();
    #[test]
    fn test_marriage() {
        assert!(has_marriage(SPADES_MARRIAGE));
    }

    #[test]
    fn test_aces_around() {
        assert_eq!(count_rank_around(ACES_AROUND, Rank::Ace), 1);
        assert_eq!(count_rank_around(ACES_AROUND | ACES_AROUND_AROUND, Rank::Ace), 2);
    }

    #[test]
    fn test_meld() {
        assert_eq!(compute_meld(SPADES_MARRIAGE, Some(Suit::Hearts)), 2);
        assert_eq!(compute_meld(SPADES_MARRIAGE, Some(Suit::Spades)), 4);
    }
}