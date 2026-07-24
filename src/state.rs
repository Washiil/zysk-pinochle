use crate::card;
use crate::types::{Phase, Suit, NO_CARD, NO_PLAYER};
use rand::seq::SliceRandom;

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub hands: [u64; 4],
    pub trick_cards: [u8; 4],
    pub scores: [i16; 2],
    pub trick_points: [u16; 2],
    pub meld_scores: [u16; 2],
    pub current_bid: u16,
    pub trump_suit: Suit,
    pub turn: u8,
    pub leader: u8,
    pub declarer: u8,
    pub phase: Phase,
    pub tricks_played: u8,
    pub pass_count: u8,
}

#[inline(always)]
pub const fn team(player: u8) -> usize {
    (player & 1) as usize
}

pub fn new_hand() -> GameState {
    let mut state = GameState {
        hands: [0; 4],
        trick_cards: [NO_CARD; 4],
        scores: [0, 0],
        trick_points: [0, 0],
        meld_scores: [0, 0],
        current_bid: 0,
        trump_suit: Suit::Spades,
        turn: 0,
        leader: NO_PLAYER,
        declarer: NO_PLAYER,
        phase: Phase::Bidding,
        tricks_played: 0,
        pass_count: 0,
    };
    reset_hand(&mut state);
    state.scores = [0, 0]; // reset_hand preserves scores; new_hand does not
    state
}

/// Resets all hand-specific fields in-place for a new deal.
/// Preserves `scores` so multi-hand games accumulate correctly.
/// The struct stays at its current address for cache residency.
pub fn reset_hand(state: &mut GameState) {
    state.hands = deal();
    state.trick_cards = [NO_CARD; 4];
    state.trick_points = [0, 0];
    state.meld_scores = [0, 0];
    state.current_bid = 0;
    state.trump_suit = Suit::Spades;
    state.turn = 0;
    state.leader = NO_PLAYER;
    state.declarer = NO_PLAYER;
    state.phase = Phase::Bidding;
    state.tricks_played = 0;
    state.pass_count = 0;
}

pub fn deal() -> [u64; 4] {
    let mut hands = [0u64; 4];
    let mut indices: [u8; 48] = std::array::from_fn(|i| i as u8);
    indices.shuffle(&mut rand::rng());

    for (p, chunk) in indices.chunks(12).enumerate() {
        for &idx in chunk {
            hands[p] |= card::card_mask(idx);
        }
    }

    hands
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_game_state_size() {
        assert_eq!(mem::size_of::<GameState>(), 64);
        assert_eq!(mem::align_of::<GameState>(), 64);
    }

    #[test]
    fn test_deal_hands() {
        let hands = deal();
        for i in 0..4 {
            assert_eq!(hands[i].count_ones(), 12, "player {} has {} cards", i, hands[i].count_ones());
        }
        // No overlap between any two hands
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_eq!(hands[i] & hands[j], 0, "hands {} and {} overlap", i, j);
            }
        }
        // All 48 cards accounted for
        let total = hands[0] | hands[1] | hands[2] | hands[3];
        assert_eq!(total, card::DECK_MASK);
    }

    #[test]
    fn test_new_hand() {
        let state = new_hand();
        assert_eq!(state.phase, Phase::Bidding);
        assert_eq!(state.turn, 0);
        assert_eq!(state.tricks_played, 0);
        assert_eq!(state.current_bid, 0);
        assert_eq!(state.trick_cards, [NO_CARD; 4]);
        // All 48 cards dealt
        let total = state.hands[0] | state.hands[1] | state.hands[2] | state.hands[3];
        assert_eq!(total, card::DECK_MASK);
    }

    #[test]
    fn test_team() {
        assert_eq!(team(0), 0);
        assert_eq!(team(1), 1);
        assert_eq!(team(2), 0);
        assert_eq!(team(3), 1);
    }

}
