use std::cmp::Ordering;

pub const UNPLAYED_CARD_INDEX: u8 = 255;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameError {
    NotPlayerTurn,
    CardNotLegalPlay,
    InvalidBidAmount,
    PhaseMismatch,
    InvalidPlayerIndex,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
}

impl TryFrom<usize> for Player {
    type Error = GameError;

    fn try_from(n: usize) -> Result<Player, Self::Error> {
        match n {
            0 => Ok(Player::One),
            1 => Ok(Player::Two),
            2 => Ok(Player::Three),
            3 => Ok(Player::Four),
            _ => Err(GameError::InvalidPlayerIndex),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GamePhase {
    Bidding,
    TrickTaking,
    Finished,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Play a card (0..47)
    Play(Card),
    /// Place a Bid (need to add a suit here as well)
    Bid(u16),
}

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

    #[inline(always)]
    pub const fn from_index(index: u8) -> Suit {
        let val = index / 12;
        match val {
            0 => Suit::Spades,
            1 => Suit::Clubs,
            2 => Suit::Hearts,
            3 => Suit::Diamonds,
            _ => unreachable!()
        }
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
    const PAIR_MASK: u16 = 0b11;
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
    pub const fn relative_mask(self) -> u16 {
        Self::PAIR_MASK << (self.power_index() * Self::COPIES_PER_RANK)
    }

    #[inline(always)]
    pub const fn global_mask(self) -> u64 {
        let m = self.relative_mask() as u64;

        m | (m << Self::SUIT_WIDTH) | (m << (Self::SUIT_WIDTH * 2)) | (m << (Self::SUIT_WIDTH * 3))
    }

    #[inline(always)]
    pub const fn higher_global_mask(self) -> u64 {
        let suit_mask = self.higher_mask() as u64;

        suit_mask
            | (suit_mask << Self::SUIT_WIDTH)
            | (suit_mask << (Self::SUIT_WIDTH * 2))
            | (suit_mask << (Self::SUIT_WIDTH * 3))
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Rank {
        let val = index % 12;
        match val {
            0..=1 => Rank::Nine,
            2..=3 => Rank::Jack,
            4..=5 => Rank::Queen,
            6..=7 => Rank::King,
            8..=9 => Rank::Ten,
            10..=11 => Rank::Ace,
            _ => unreachable!()
        }
    }

    #[inline(always)]
    pub const fn higher_mask(self) -> u16 {
        const SUIT_MASK: u16 = 0b1111_1111_1111;
        // This shift represents starting at the next highest rank
        let shift = (self.power_index() + 1) * Self::COPIES_PER_RANK;

        if shift >= 12 {
            0
        } else {
            (!0u16 << shift) & SUIT_MASK
        }
    }
}

impl Ord for Rank {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.power_index().cmp(&other.power_index())
    }
}

impl PartialOrd for Rank {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    pub fn from_index(index: u8) -> Card {
        let suit = Suit::from_index(index);
        let rank = Rank::from_index(index);
        Self { rank, suit }
    }

    pub fn to_index(self) -> u8 {
        (self.suit.index() * 12 + self.rank.power_index()) as u8
    }

    pub fn mask(self) -> u64 {
        self.rank.global_mask() & self.suit.mask()
    }
}

#[cfg(test)]
mod types_tests {
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

        assert_eq!(
            mem::size_of::<Card>(),
            2,
            "Card must be exactly 2 bytes"
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
    fn test_suit_from_index() {
        assert_eq!(Suit::from_index(0),  Suit::Spades);
        assert_eq!(Suit::from_index(11), Suit::Spades);
        assert_eq!(Suit::from_index(12), Suit::Clubs);
        assert_eq!(Suit::from_index(23), Suit::Clubs);
        assert_eq!(Suit::from_index(24), Suit::Hearts);
        assert_eq!(Suit::from_index(35), Suit::Hearts);
        assert_eq!(Suit::from_index(36), Suit::Diamonds);
        assert_eq!(Suit::from_index(47), Suit::Diamonds);
    }

    #[test]
    fn test_rank_mask() {
        let deck: u64 = 0b111111111111_111111111111_111111111111_111111111111;
        assert_eq!(deck & Rank::Nine.global_mask(),  0b000000000011_000000000011_000000000011_000000000011);
        assert_eq!(deck & Rank::Jack.global_mask(),  0b000000001100_000000001100_000000001100_000000001100);
        assert_eq!(deck & Rank::Queen.global_mask(), 0b000000110000_000000110000_000000110000_000000110000);
        assert_eq!(deck & Rank::King.global_mask(),  0b000011000000_000011000000_000011000000_000011000000);
        assert_eq!(deck & Rank::Ten.global_mask(),   0b001100000000_001100000000_001100000000_001100000000);
        assert_eq!(deck & Rank::Ace.global_mask(),   0b110000000000_110000000000_110000000000_110000000000);        assert_eq!(deck & Suit::Clubs.mask() & Suit::Spades.mask(),  0);
        assert_eq!(deck & Rank::Nine.global_mask() & Rank::Ten.global_mask(),  0);

        let single_suit: u16 = 0b111111111111;
        assert_eq!(single_suit & Rank::Nine.relative_mask(), 0b000000000011);
        assert_eq!(single_suit & Rank::Jack.relative_mask(), 0b000000001100);
        assert_eq!(single_suit & Rank::Queen.relative_mask(), 0b000000110000);
        assert_eq!(single_suit & Rank::King.relative_mask(), 0b000011000000);
        assert_eq!(single_suit & Rank::Ten.relative_mask(), 0b001100000000);
        assert_eq!(single_suit & Rank::Ace.relative_mask(), 0b110000000000);
        assert_eq!(single_suit & Rank::Nine.relative_mask() & Rank::Ten.relative_mask(), 0);
    }

    #[test]
    fn test_rank_from_index() {
        assert_eq!(Rank::from_index(0), Rank::Nine);
        assert_eq!(Rank::from_index(1), Rank::Nine);
        assert_eq!(Rank::from_index(2), Rank::Jack);
        assert_eq!(Rank::from_index(3), Rank::Jack);
        assert_eq!(Rank::from_index(4), Rank::Queen);
        assert_eq!(Rank::from_index(5), Rank::Queen);
        assert_eq!(Rank::from_index(6), Rank::King);
        assert_eq!(Rank::from_index(7), Rank::King);
        assert_eq!(Rank::from_index(8), Rank::Ten);
        assert_eq!(Rank::from_index(9), Rank::Ten);
        assert_eq!(Rank::from_index(10), Rank::Ace);
        assert_eq!(Rank::from_index(11), Rank::Ace);
    }

    #[test]
    fn test_higher_mask() {
        assert_eq!(Rank::Nine.higher_mask(),  0b111111111100);
        assert_eq!(Rank::Jack.higher_mask(),  0b111111110000);
        assert_eq!(Rank::Queen.higher_mask(), 0b111111000000);
        assert_eq!(Rank::King.higher_mask(),  0b111100000000);
        assert_eq!(Rank::Ten.higher_mask(),   0b110000000000);
        assert_eq!(Rank::Ace.higher_mask(),   0b000000000000);
    }
}