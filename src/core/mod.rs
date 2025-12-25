#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Suit {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Nine = 0,
    Jack = 1,
    Queen = 2,
    King = 3,
    Ten = 4,
    Ace = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    copy: u8,
    rank: Rank,
    suit: Suit,
}

impl Card {
    #[inline(always)]
    pub const fn new(suit: Suit, rank: Rank, copy: u8) -> Self {
        Self {
            suit,
            rank,
            copy: copy & 1,
        }
    }

    /// Aligned with bitboard position (0-47)
    /// Layout: Suit (0, 12, 24, 36) + Rank (0, 2, 4, 6, 8, 10) + Copy (0, 1)
    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        (self.suit as u8 * 12) + (self.rank as u8 * 2) + self.copy
    }

    /// Reconstructs a card from an index (0-47)
    /// This is now compatible with bitboard trailing_zeros()
    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 48);

        // Simple and branchless operations
        let suit_id = index / 12;
        let rank_id = (index % 12) / 2;
        let copy = index % 2;

        Self {
            suit: match suit_id {
                0 => Suit::Spades,
                1 => Suit::Hearts,
                2 => Suit::Diamonds,
                _ => Suit::Clubs,
            },
            rank: match rank_id {
                0 => Rank::Nine,
                1 => Rank::Jack,
                2 => Rank::Queen,
                3 => Rank::King,
                4 => Rank::Ten,
                _ => Rank::Ace,
            },
            copy,
        }
    }

    #[inline(always)]
    pub const fn suit(self) -> Suit { self.suit }

    #[inline(always)]
    pub const fn rank(self) -> Rank { self.rank }

    #[inline(always)]
    pub const fn copy(self) -> u8 { self.copy }

    /// Returns a bitboard (u64) with only this card's bit set
    #[inline(always)]
    pub const fn bitmask(self) -> u64 {
        1u64 << self.to_index()
    }
}

impl Suit {
    #[inline(always)]
    pub const fn bitmask(self) -> u64 {
        0xFFF << (self as u64 * 12)
    }
}

impl Rank {
    /// Returns a mask for both copies of this rank across ALL suits.
    pub const fn global_mask(self) -> u64 {
        let suit_bits = 0b11 << (self as u64 * 2);
        suit_bits | (suit_bits << 12) | (suit_bits << 24) | (suit_bits << 36)
    }

    #[inline(always)]
    pub const fn points(self) -> u8 {
        match self {
            Rank::Ace => 11,
            Rank::Ten => 10,
            Rank::King => 4,
            Rank::Queen => 3,
            Rank::Jack => 2,
            Rank::Nine => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency() {
        for i in 0..48 {
            let card = Card::from_index(i);
            assert_eq!(card.to_index(), i, "Roundtrip failed for index {}", i);
        }
    }

    #[test]
    fn test_specific_mappings() {
        // Spades, Nine, Copy 0 should be 0
        assert_eq!(Card::new(Suit::Spades, Rank::Nine, 0).to_index(), 0);
        // Spades, Ace, Copy 1 should be 11
        assert_eq!(Card::new(Suit::Spades, Rank::Ace, 1).to_index(), 11);
        // Clubs, Ace, Copy 1 should be 47
        assert_eq!(Card::new(Suit::Clubs, Rank::Ace, 1).to_index(), 47);
    }

    #[test]
    fn test_rank_masks() {
        // Ensure the global mask for Aces has 8 bits set (2 for each suit)
        let aces_mask = Rank::Ace.global_mask();
        assert_eq!(aces_mask.count_ones(), 8);

        // Verify a specific Ace is in the mask
        let ace_of_spades = Card::new(Suit::Spades, Rank::Ace, 0).bitmask();
        assert_ne!(aces_mask & ace_of_spades, 0);
    }
}