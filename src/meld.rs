use crate::card;
use crate::types::{Rank, Suit};
use std::sync::LazyLock;

/// Lookup table: given a 12-bit suit pattern (bits 0-1=Nine, 2-3=Jack, 4-5=Queen,
/// 6-7=King, 8-9=Ten, 10-11=Ace), returns the meld value when that suit is TRUMP.
pub static TRUMP_MELD_TABLE: LazyLock<[u16; 4096]> = LazyLock::new(|| {
    let mut table = [0u16; 4096];
    for bits in 0..4096u16 {
        table[bits as usize] = suit_meld(bits, true);
    }
    table
});

/// Lookup table: given a 12-bit suit pattern, returns the meld value when that suit is PLAIN.
pub static PLAIN_MELD_TABLE: LazyLock<[u16; 4096]> = LazyLock::new(|| {
    let mut table = [0u16; 4096];
    for bits in 0..4096u16 {
        table[bits as usize] = suit_meld(bits, false);
    }
    table
});

fn count_rank(bits: u16, rank: Rank) -> u8 {
    let mask = Rank::relative_mask(rank);
    let masked = bits & mask;
    let shift = rank.power_index() * 2;
    (masked >> shift).count_ones() as u8
}

fn suit_meld(bits: u16, is_trump: bool) -> u16 {
    let mut score = 0u16;

    // Dix: 10 points per nine (trump only)
    if is_trump {
        score += count_rank(bits, Rank::Nine) as u16 * 10;
    }

    // Count marriages (K+Q combos). Each K can pair with each Q.
    let kings = count_rank(bits, Rank::King);
    let queens = count_rank(bits, Rank::Queen);
    let marriages = kings.min(queens);

    // Count runs (A+10+K+Q+J, each at least one copy)
    let aces_c = count_rank(bits, Rank::Ace);
    let tens = count_rank(bits, Rank::Ten);
    let jacks = count_rank(bits, Rank::Jack);
    let runs = aces_c.min(tens).min(kings).min(queens).min(jacks);

    if is_trump {
        // Trump marriages: 40 pts for royal marriage (K+Q), but only for combos
        // not consumed by a run. A run implies a marriage, so: runs get 150,
        // additional marriages get 40 each.
        let remaining_marriages = marriages.saturating_sub(runs);
        score += runs as u16 * 150;
        score += remaining_marriages as u16 * 40;
    } else {
        // Plain marriage: 20 points
        score += marriages as u16 * 20;
    }

    score
}

/// Returns the meld score for a player's hand given the trump suit.
/// Handles both per-suit melds and cross-suit ("around") melds.
pub fn hand_meld(hand: u64, trump: Suit) -> u16 {
    let mut total = 0u16;

    // Per-suit melds via lookup tables
    for suit in [Suit::Spades, Suit::Clubs, Suit::Hearts, Suit::Diamonds] {
        let suit_bits = ((hand & Suit::mask(suit)) >> Suit::shift(suit)) as u16;
        if suit == trump {
            total += TRUMP_MELD_TABLE[suit_bits as usize];
        } else {
            total += PLAIN_MELD_TABLE[suit_bits as usize];
        }
    }

    // Cross-suit "around" melds: count how many of each rank across ALL suits
    let mut rank_counts = [0u8; 6];
    for idx in card::iter_cards(hand) {
        rank_counts[Rank::from_index(idx).power_index()] += 1;
    }

    // Aces around: 100 per set (need 1 ace in each suit = 4 aces per set)
    // With double pinochle, max 2 sets (8 aces possible)
    total += (rank_counts[Rank::Ace.power_index()] / 4).min(2) as u16 * 100;
    total += (rank_counts[Rank::King.power_index()] / 4).min(2) as u16 * 80;
    total += (rank_counts[Rank::Queen.power_index()] / 4).min(2) as u16 * 60;
    total += (rank_counts[Rank::Jack.power_index()] / 4).min(2) as u16 * 40;

    // Pinochle: J diamonds + Queen Spades = 40 points per combo
    let qs_bits = (hand & Suit::mask(Suit::Spades)) >> Suit::shift(Suit::Spades);
    let jd_bits = (hand & Suit::mask(Suit::Diamonds)) >> Suit::shift(Suit::Diamonds);
    let qs_count = count_rank(qs_bits as u16, Rank::Queen);
    let jd_count = count_rank(jd_bits as u16, Rank::Jack);
    total += qs_count.min(jd_count) as u16 * 40;

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card;

    #[test]
    fn test_empty_hand() {
        assert_eq!(hand_meld(0, Suit::Spades), 0);
    }

    #[test]
    fn test_dix_trump() {
        // A hand with just a 9 of spades, trump is spades
        let hand = card::card_mask(0); // index 0 = Nine of Spades copy 0
        assert_eq!(hand_meld(hand, Suit::Spades), 10);
    }

    #[test]
    fn test_dix_non_trump() {
        // A nine in a non-trump suit is worth 0
        let hand = card::card_mask(0); // Nine of Spades, trump is Hearts
        assert_eq!(hand_meld(hand, Suit::Hearts), 0);
    }

    #[test]
    fn test_marriage_plain() {
        // K+Q of spades, trump is hearts
        // Index 4 = Queen of Spades copy 0, Index 6 = King of Spades copy 0
        let hand = card::card_mask(4) | card::card_mask(6);
        assert_eq!(hand_meld(hand, Suit::Hearts), 20);
    }

    #[test]
    fn test_royal_marriage_trump() {
        // K+Q of spades, trump is spades
        let hand = card::card_mask(4) | card::card_mask(6);
        assert_eq!(hand_meld(hand, Suit::Spades), 40);
    }

    #[test]
    fn test_run_trump() {
        // A+10+K+Q+J of spades, trump is spades
        // Nine: 0-1, Jack: 2-3, Queen: 4-5, King: 6-7, Ten: 8-9, Ace: 10-11
        let hand = card::card_mask(2)   // Jack copy 0
            | card::card_mask(4)   // Queen copy 0
            | card::card_mask(6)   // King copy 0
            | card::card_mask(8)   // Ten copy 0
            | card::card_mask(10); // Ace copy 0
        assert_eq!(hand_meld(hand, Suit::Spades), 150);
    }

    #[test]
    fn test_pinochle() {
        // Q♠ + J♦
        let hand = card::card_mask(4)  // Queen of Spades copy 0
            | card::card_mask(38); // Jack of Diamonds copy 0 (Diamonds start at 36, Jack = +2)
        assert_eq!(hand_meld(hand, Suit::Spades), 40);
    }

    #[test]
    fn test_aces_around() {
        // A in each suit (one copy each)
        let hand = card::card_mask(10)  // Ace of Spades copy 0
            | card::card_mask(22)  // Ace of Clubs copy 0 (12+10=22)
            | card::card_mask(34)  // Ace of Hearts copy 0 (24+10=34)
            | card::card_mask(46); // Ace of Diamonds copy 0 (36+10=46)
        assert_eq!(hand_meld(hand, Suit::Spades), 100);
    }

    #[test]
    fn test_double_copy_count() {
        // Both copies of K+Q of spades (plain suit, trump=Hearts)
        // Copy 0: K=6, Q=4. Copy 1: K=7, Q=5.
        let hand = card::card_mask(4) | card::card_mask(5)  // Both Queens
            | card::card_mask(6) | card::card_mask(7);  // Both Kings
        // 2 marriages (K+Q pairs) × 20 pts = 40
        assert_eq!(hand_meld(hand, Suit::Hearts), 40);
    }
}
