use std::cmp::min;
use crate::core::{Card, Rank, Suit};
use std::sync::OnceLock;

pub struct MeldTable {
    pub trump_scores: [u8; 4096],
    pub side_scores: [u8; 4096],
}

static MELD_LUT: OnceLock<MeldTable> = OnceLock::new();

/// Global accessor for the LUT
pub fn get_meld_table() -> &'static MeldTable {
    MELD_LUT.get_or_init(|| {
        let mut trump_scores = [0u8; 4096];
        let mut side_scores = [0u8; 4096];

        for bits in 0..4096u16 {
            let a = ((bits >> 10) & 0b11).count_ones() as u8;
            let t = ((bits >> 8)  & 0b11).count_ones() as u8;
            let k = ((bits >> 6)  & 0b11).count_ones() as u8;
            let q = ((bits >> 4)  & 0b11).count_ones() as u8;
            let j = ((bits >> 2)  & 0b11).count_ones() as u8;
            let n = (bits & 0b11).count_ones() as u8;

            // Just marriages
            let marriages = k.min(q);
            side_scores[bits as usize] = marriages * 2;

            // Trump logic
            // Runs (15/30), Royal Marriages (4), Dix (1)
            let runs = a.min(t).min(k).min(q).min(j);
            let run_points = match runs {
                2 => 30,
                1 => 15,
                _ => 0,
            };

            // Only score marriages that aren't already part of a run
            let extra_marriages = marriages.saturating_sub(runs);

            trump_scores[bits as usize] = run_points + (extra_marriages * 4) + n;
        }

        MeldTable { trump_scores, side_scores }
    })
}

#[inline(always)]
fn count_marriage(cards: u64, suit: Suit) -> u8 {
    let kings = (cards & suit.mask() & Rank::King.mask()).count_ones();
    let queens = (cards & suit.mask() & Rank::Queen.mask()).count_ones();
    kings.min(queens) as u8
}

#[inline(always)]
fn count_runs(hand: u64, suit: Suit) -> u8 {

}

#[inline(always)]
fn count_rank_around(hand: u64, rank: Rank) -> u8 {
    let mask = rank.mask();
    let cards = hand & mask;

    let s = (cards & Suit::Spades.mask()).count_ones();
    let h = (cards & Suit::Hearts.mask()).count_ones();
    let d = (cards & Suit::Diamonds.mask()).count_ones();
    let c = (cards & Suit::Clubs.mask()).count_ones();

    s.min(h).min(d).min(c) as u8
}

#[inline(always)]
fn count_pinochle(hand: u64) -> u8 {
    let jacks = (Rank::Jack.mask() & Suit::Diamonds.mask() & hand).count_ones();
    let queens = (Rank::Queen.mask() & Suit::Spades.mask() & hand).count_ones();

    jacks.min(queens) as u8
}


pub fn compute_meld_hpc(hand: u64, trump: Option<Suit>) -> u8 {
    let mut total_meld = 0u8;

    // 1. "Arounds" and Pinochle (Keep these as bitwise logic,
    // as they span across multiple suits and don't fit the 12-bit LUT)
    total_meld += count_rank_around(hand, Rank::Ace) * 10;
    total_meld += count_rank_around(hand, Rank::King) * 8;
    total_meld += count_rank_around(hand, Rank::Queen) * 6;
    total_meld += count_rank_around(hand, Rank::Jack) * 4;
    total_meld += count_pinochle(hand) * 4;

    // 2. Suit-based meld (Marriages and Runs)
    // We unroll this for maximum speed
    let trump_idx = trump.map(|s| s as usize).unwrap_or(99); // 99 as a sentinel

    // Extract suit bits
    let s_bits = ((hand >> 0)  & 0xFFF) as usize;
    let h_bits = ((hand >> 12) & 0xFFF) as usize;
    let d_bits = ((hand >> 24) & 0xFFF) as usize;
    let c_bits = ((hand >> 36) & 0xFFF) as usize;

    // Add scores based on whether suit is trump
    total_meld += if trump_idx == 0 { MELD_LUT.trump_scores[s_bits] } else { MELD_LUT.side_scores[s_bits] };
    total_meld += if trump_idx == 1 { MELD_LUT.trump_scores[h_bits] } else { MELD_LUT.side_scores[h_bits] };
    total_meld += if trump_idx == 2 { MELD_LUT.trump_scores[d_bits] } else { MELD_LUT.side_scores[d_bits] };
    total_meld += if trump_idx == 3 { MELD_LUT.trump_scores[c_bits] } else { MELD_LUT.side_scores[c_bits] };

    total_meld
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPADES_MARRIAGE: u64 = Card::new(Suit::Spades, Rank::King, 0).bitmask() | Card::new(Suit::Spades, Rank::Queen, 0).bitmask();
    const ACES_AROUND: u64 = Card::new(Suit::Spades, Rank::Ace, 0).bitmask() | Card::new(Suit::Clubs, Rank::Ace, 0).bitmask() | Card::new(Suit::Hearts, Rank::Ace, 0).bitmask() | Card::new(Suit::Diamonds, Rank::Ace, 0).bitmask();
    const ACES_AROUND_AROUND: u64 = Card::new(Suit::Spades, Rank::Ace, 1).bitmask() | Card::new(Suit::Clubs, Rank::Ace, 1).bitmask() | Card::new(Suit::Hearts, Rank::Ace, 1).bitmask() | Card::new(Suit::Diamonds, Rank::Ace, 1).bitmask();
    const PINOCHLE: u64 = Card::new(Suit::Diamonds, Rank::Jack, 0).bitmask() | Card::new(Suit::Spades, Rank::Queen, 0).bitmask();
    const PINOCHLE_1: u64 = Card::new(Suit::Diamonds, Rank::Jack, 1).bitmask() | Card::new(Suit::Spades, Rank::Queen, 1).bitmask();
    #[test]
    fn test_marriage() {
        assert_eq!(count_marriage(SPADES_MARRIAGE, Suit::Spades), 1);
    }

    #[test]
    fn test_rank_around() {
        assert_eq!(count_rank_around(ACES_AROUND, Rank::Queen), 0);
        assert_eq!(count_rank_around(ACES_AROUND, Rank::King), 0);
        assert_eq!(count_rank_around(ACES_AROUND, Rank::Ace), 1);
        assert_eq!(count_rank_around(ACES_AROUND | ACES_AROUND_AROUND, Rank::Queen), 0);
        assert_eq!(count_rank_around(ACES_AROUND | ACES_AROUND_AROUND, Rank::King), 0);
        assert_eq!(count_rank_around(ACES_AROUND | ACES_AROUND_AROUND, Rank::Ace), 2);
    }

    #[test]
    fn test_pinochle() {
        assert_eq!(count_pinochle(SPADES_MARRIAGE), 0);
        assert_eq!(count_pinochle(PINOCHLE), 1);
        assert_eq!(count_pinochle(PINOCHLE | PINOCHLE_1), 2);
    }

    #[test]
    fn test_meld() {
        assert_eq!(compute_meld(SPADES_MARRIAGE, Some(Suit::Hearts)), 2);
        assert_eq!(compute_meld(SPADES_MARRIAGE, Some(Suit::Spades)), 4);
    }
}