#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds
}

impl Suit {
    const BIT_WIDTH: usize = 12;

    /// Maps a Suit to its 0-3 index in the bitboard.
    #[inline(always)]
    pub const fn index(self) -> usize {
        match self {
            Suit::Spades   => 0, // Spades
            Suit::Clubs    => 1, // Clubs
            Suit::Hearts   => 2, // Hearts
            Suit::Diamonds => 3, // Diamonds
        }
    }

    /// Returns the shift amount (0, 12, 24, or 36)
    #[inline(always)]
    pub const fn shift(self) -> usize {
        self.index() * Self::BIT_WIDTH
    }

    /// Returns the mask for this specific suit (0xFFF shifted)
    #[inline(always)]
    pub const fn mask(self) -> u64 {
        0xFFF << self.shift()
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace
}

impl Rank {
    const COPIES_PER_RANK: usize = 2;
    const PAIR_MASK: u64 = 0b11;
    const SUIT_WIDTH: usize = 12;

    #[inline(always)]
    pub const fn power_index(self) -> usize {
        match self {
            Rank::Nine  => 0,
            Rank::Jack  => 1,
            Rank::Queen => 2,
            Rank::King  => 3,
            Rank::Ten   => 4,
            Rank::Ace   => 5,
        }
    }

    #[inline(always)]
    pub const fn relative_mask(self) -> u64 {
        Self::PAIR_MASK << (self.power_index() * Self::COPIES_PER_RANK)
    }

    #[inline(always)]
    pub const fn global_mask(self) -> u64 {
        let m = self.relative_mask();

        m | (m << Self::SUIT_WIDTH) | (m << (Self::SUIT_WIDTH * 2)) | (m << (Self::SUIT_WIDTH * 3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_memory_size() {
        assert_eq!(
            mem::size_of::<Suit>(),
            1,
            "Suit must be exactly 1 bytes"
        );

        assert_eq!(
            mem::size_of::<Rank>(),
            1,
            "Rank must be exactly 1 bytes"
        );
    }

    #[test]
    fn test_suit_masks() {
        let deck: u64 = 0b111111111111_111111111111_111111111111_111111111111;

        assert_eq!(deck & Suit::Spades.mask(),   0b000000000000_000000000000_000000000000_111111111111);
        assert_eq!(deck & Suit::Clubs.mask(),    0b000000000000_000000000000_111111111111_000000000000);
        assert_eq!(deck & Suit::Hearts.mask(),   0b000000000000_111111111111_000000000000_000000000000);
        assert_eq!(deck & Suit::Diamonds.mask(), 0b111111111111_000000000000_000000000000_000000000000);
        assert_eq!(deck & Suit::Clubs.mask() & Suit::Spades.mask(),  0);
    }

    #[test]
    fn test_rank_mask() {
        let deck: u64 = 0b111111111111_111111111111_111111111111_111111111111;

        // Global Masks
        assert_eq!(deck & Rank::Nine.global_mask(),  0b000000000011_000000000011_000000000011_000000000011);
        assert_eq!(deck & Rank::Jack.global_mask(),  0b000000001100_000000001100_000000001100_000000001100);
        assert_eq!(deck & Rank::Queen.global_mask(), 0b000000110000_000000110000_000000110000_000000110000);
        assert_eq!(deck & Rank::King.global_mask(),  0b000011000000_000011000000_000011000000_000011000000);
        assert_eq!(deck & Rank::Ten.global_mask(),   0b001100000000_001100000000_001100000000_001100000000);
        assert_eq!(deck & Rank::Ace.global_mask(),   0b110000000000_110000000000_110000000000_110000000000);        assert_eq!(deck & Suit::Clubs.mask() & Suit::Spades.mask(),  0);
        assert_eq!(deck & Rank::Nine.global_mask() & Rank::Ten.global_mask(),  0);

        // Single Suit
        assert_eq!(deck & Rank::Nine.relative_mask(),  0b000000000011);
        assert_eq!(deck & Rank::Jack.relative_mask(),  0b000000001100);
        assert_eq!(deck & Rank::Queen.relative_mask(), 0b000000110000);
        assert_eq!(deck & Rank::King.relative_mask(),  0b000011000000);
        assert_eq!(deck & Rank::Ten.relative_mask(),   0b001100000000);
        assert_eq!(deck & Rank::Ace.relative_mask(),   0b110000000000);
        assert_eq!(deck & Rank::Nine.relative_mask() & Rank::Ten.relative_mask(),  0);
    }
}