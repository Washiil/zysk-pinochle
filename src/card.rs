pub const DECK_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

#[inline(always)]
pub const fn card_mask(index: u8) -> u64 {
    1u64 << (index as u64)
}

pub struct CardIter {
    bits: u64,
}

impl CardIter {
    pub fn new(bits: u64) -> Self {
        Self { bits }
    }
}

impl Iterator for CardIter {
    type Item = u8;

    #[inline(always)]
    fn next(&mut self) -> Option<u8> {
        if self.bits == 0 {
            return None;
        }
        let index = self.bits.trailing_zeros() as u8;
        self.bits &= self.bits - 1;
        Some(index)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.bits.count_ones() as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for CardIter {}

pub fn iter_cards(bits: u64) -> CardIter {
    CardIter::new(bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Rank, Suit};

    #[test]
    fn test_deck_mask() {
        assert_eq!(DECK_MASK.count_ones(), 48);
        assert_eq!(DECK_MASK >> 48, 0);
    }

    #[test]
    fn test_card_mask() {
        for i in 0..48u8 {
            let mask = card_mask(i);
            assert_eq!(mask.count_ones(), 1);
            assert_eq!(mask.trailing_zeros() as u8, i);
        }
    }

    #[test]
    fn test_suit_masks_from_types() {
        assert_eq!(Suit::Spades.mask(), 0x0000000000000FFF);
        assert_eq!(Suit::Clubs.mask(), 0x0000000000FFF000);
        assert_eq!(Suit::Hearts.mask(), 0x0000000FFF000000);
        assert_eq!(Suit::Diamonds.mask(), 0x0000FFF000000000);

        let sum = Suit::Spades.mask() | Suit::Clubs.mask() | Suit::Hearts.mask() | Suit::Diamonds.mask();
        assert_eq!(sum, DECK_MASK);

        // Disjoint
        assert_eq!(Suit::Spades.mask() & Suit::Clubs.mask(), 0);
        assert_eq!(Suit::Spades.mask() & Suit::Hearts.mask(), 0);
    }

    #[test]
    fn test_rank_global_masks_from_types() {
        let mut sum = 0u64;
        for rank in [Rank::Nine, Rank::Jack, Rank::Queen, Rank::King, Rank::Ten, Rank::Ace] {
            let mask = rank.global_mask();
            sum |= mask;
            // Each rank mask should be disjoint from others
            for other in [Rank::Nine, Rank::Jack, Rank::Queen, Rank::King, Rank::Ten, Rank::Ace] {
                if rank != other {
                    assert_eq!(mask & other.global_mask(), 0);
                }
            }
        }
        assert_eq!(sum, DECK_MASK);
    }

    #[test]
    fn test_higher_global_mask_from_types() {
        assert_eq!(Rank::Ace.higher_global_mask(), 0);
        let above_nine = Rank::Nine.higher_global_mask();
        assert_eq!(above_nine & Rank::Nine.global_mask(), 0);
        for rank in [Rank::Jack, Rank::Queen, Rank::King, Rank::Ten, Rank::Ace] {
            assert_eq!(above_nine & rank.global_mask(), rank.global_mask());
        }
    }

    #[test]
    fn test_card_iter() {
        let mut bits = 0u64;
        for i in (0..48).step_by(2) {
            bits |= card_mask(i);
        }
        let indices: Vec<u8> = iter_cards(bits).collect();
        assert_eq!(indices.len(), 24);
        for (i, idx) in indices.iter().enumerate() {
            assert_eq!(idx % 2, 0);
            assert_eq!(*idx as usize, i * 2);
        }
    }

    #[test]
    fn test_card_iter_empty() {
        let indices: Vec<u8> = iter_cards(0).collect();
        assert!(indices.is_empty());
    }

    #[test]
    fn test_card_iter_full_deck() {
        let count = iter_cards(DECK_MASK).count();
        assert_eq!(count, 48);
    }

    #[test]
    fn test_card_iter_exact_size() {
        let mut iter = iter_cards(0x000000000000000F);
        assert_eq!(iter.len(), 4);
        iter.next();
        assert_eq!(iter.len(), 3);
        iter.next();
        assert_eq!(iter.len(), 2);
        iter.next();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }
}
