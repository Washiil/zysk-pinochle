use std::cmp::Ordering;

pub const NO_CARD: u8 = 255;
pub const NO_PLAYER: u8 = 255;

#[inline(always)]
pub const fn replicate_suit(mask_12bit: u64) -> u64 {
    mask_12bit | (mask_12bit << 12) | (mask_12bit << 24) | (mask_12bit << 36)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Spades = 0,
    Clubs = 1,
    Hearts = 2,
    Diamonds = 3,
}

impl Suit {
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn shift(self) -> usize {
        self.index() * 12
    }

    #[inline(always)]
    pub const fn mask(self) -> u64 {
        0xFFF << self.shift()
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Suit {
        match index / 12 {
            0 => Suit::Spades,
            1 => Suit::Clubs,
            2 => Suit::Hearts,
            3 => Suit::Diamonds,
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rank {
    Nine = 0,
    Jack = 1,
    Queen = 2,
    King = 3,
    Ten = 4,
    Ace = 5,
}

impl Rank {
    const COPIES_PER_RANK: usize = 2;
    const PAIR_MASK: u16 = 0b11;

    #[inline(always)]
    pub const fn power_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn relative_mask(self) -> u16 {
        Self::PAIR_MASK << (self.power_index() * Self::COPIES_PER_RANK)
    }

    #[inline(always)]
    pub const fn global_mask(self) -> u64 {
        replicate_suit(self.relative_mask() as u64)
    }

    #[inline(always)]
    pub const fn higher_suit_mask(self) -> u16 {
        const SUIT_MASK: u16 = 0b1111_1111_1111;
        let shift = (self.power_index() + 1) * Self::COPIES_PER_RANK;
        if shift >= 12 {
            0
        } else {
            (!0u16 << shift) & SUIT_MASK
        }
    }

    #[inline(always)]
    pub const fn higher_global_mask(self) -> u64 {
        replicate_suit(self.higher_suit_mask() as u64)
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Rank {
        match (index % 12) / 2 {
            0 => Rank::Nine,
            1 => Rank::Jack,
            2 => Rank::Queen,
            3 => Rank::King,
            4 => Rank::Ten,
            5 => Rank::Ace,
            _ => unreachable!(),
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

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Bidding = 0,
    TrickTaking = 1,
    Finished = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Play(u8),
    Bid(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_sizes() {
        assert_eq!(size_of::<Suit>(), 1);
        assert_eq!(size_of::<Rank>(), 1);
        assert_eq!(size_of::<Phase>(), 1);
    }

    #[test]
    fn test_suit_masks() {
        assert_eq!(Suit::Spades.mask(),   0x0000000000000FFF);
        assert_eq!(Suit::Clubs.mask(),    0x0000000000FFF000);
        assert_eq!(Suit::Hearts.mask(),   0x0000000FFF000000);
        assert_eq!(Suit::Diamonds.mask(), 0x0000FFF000000000);
    }

    #[test]
    fn test_suit_from_index() {
        for i in 0..12 { assert_eq!(Suit::from_index(i), Suit::Spades); }
        for i in 12..24 { assert_eq!(Suit::from_index(i), Suit::Clubs); }
        for i in 24..36 { assert_eq!(Suit::from_index(i), Suit::Hearts); }
        for i in 36..48 { assert_eq!(Suit::from_index(i), Suit::Diamonds); }
    }

    #[test]
    fn test_rank_global_mask() {
        let deck: u64 = 0x0000_FFFF_FFFF_FFFF;
        assert_eq!(deck & Rank::Nine.global_mask(),  0x0000003003003003);
        assert_eq!(deck & Rank::Jack.global_mask(),  0x000000C00C00C00C);
        assert_eq!(deck & Rank::Queen.global_mask(), 0x0000030030030030);
        assert_eq!(deck & Rank::King.global_mask(),  0x00000C00C00C00C0);
        assert_eq!(deck & Rank::Ten.global_mask(),   0x0000300300300300);
        assert_eq!(deck & Rank::Ace.global_mask(),   0x0000C00C00C00C00);
        assert_eq!(Rank::Nine.global_mask() & Rank::Ten.global_mask(), 0);
    }

    #[test]
    fn test_rank_from_index() {
        assert_eq!(Rank::from_index(0), Rank::Nine);
        assert_eq!(Rank::from_index(1), Rank::Nine);
        assert_eq!(Rank::from_index(2), Rank::Jack);
        assert_eq!(Rank::from_index(4), Rank::Queen);
        assert_eq!(Rank::from_index(6), Rank::King);
        assert_eq!(Rank::from_index(8), Rank::Ten);
        assert_eq!(Rank::from_index(10), Rank::Ace);
    }

    #[test]
    fn test_higher_suit_mask() {
        assert_eq!(Rank::Nine.higher_suit_mask(),  0b111111111100);
        assert_eq!(Rank::Jack.higher_suit_mask(),  0b111111110000);
        assert_eq!(Rank::Queen.higher_suit_mask(), 0b111111000000);
        assert_eq!(Rank::King.higher_suit_mask(),  0b111100000000);
        assert_eq!(Rank::Ten.higher_suit_mask(),   0b110000000000);
        assert_eq!(Rank::Ace.higher_suit_mask(),   0b000000000000);
    }

    #[test]
    fn test_rank_ordering() {
        assert!(Rank::Ace > Rank::Ten);
        assert!(Rank::Ten > Rank::King);
        assert!(Rank::King > Rank::Queen);
        assert!(Rank::Queen > Rank::Jack);
        assert!(Rank::Jack > Rank::Nine);
    }
}
