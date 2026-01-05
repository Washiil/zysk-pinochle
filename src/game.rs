use rand::seq::SliceRandom;
use crate::types::{Action, Rank, Suit};

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinochleState {
    // hands[0] = Player 0 (Dealer left)
    pub hands: [u64; 4], // 8 * 4 = 32

    // Cards the table has "seen"
    pub dead_cards: [u64; 2], // 8 * 2 = 16

    // Stores card INDICES (0-47).
    // trick_cards[0] is always Player 0's card, regardless of who led.
    // 255 is a sentinel value for non-played card
    pub trick_cards: [u8; 4], // 1 * 4 = 4

    // Store the running game score computed
    pub scores: [u16; 2], // 2 * 2 = 4
    pub active_meld: [u8; 2], // 1 * 2 = 2

    pub trump_suit: Suit, // 0..3 1 * 1 = 1
    pub turn: u8,         // 0..3 (Current player ID) 1 * 1 = 1
    pub leader: u8,       // 0..3 (Who led this trick) 1 * 1 = 1

    // Padding to fill 64 byte cache line
    pub game_state: GameState,
}

impl PinochleState {
    pub fn new() -> Self {
        Self {
            hands: Self::deal(),
            dead_cards: [0; 2],
            trick_cards: [0; 4],
            scores: [0; 2],
            active_meld: [0; 2],
            trump_suit: Suit::Clubs,
            turn: 0,
            leader: 0,
            _padding: [0; 3],
        }
    }

    fn deal() -> [u64; 4] {
        let mut rng = rand::rng();
        let mut deck: Vec<u8> = (0..48).collect();
        deck.shuffle(&mut rng);
        let mut hands = [0u64; 4];

        for (i, &card_index) in deck.iter().enumerate() {
            let player = i / 12;

            // Turn the card index into a bitmask and add it to the player's hand
            hands[player] |= 1u64 << card_index;
        }
        hands
    }

    pub fn legal_moves(&self) -> u64 {
        let hand = self.hands[self.turn as usize];

        // Leader can lead any card
        if self.leader == self.turn {
            return hand;
        }

        let lead_card_idx = self.trick_cards[self.leader as usize];
        assert_ne!(lead_card_idx, 255, "Leader did not play card.");

        let lead_suit = Suit::from_index(lead_card_idx);
        let mut trump_to_beat = Rank::Nine;

        // Calculating highest trump played
        for i in self.trick_cards {
            // Ignore unplayed cards
            if i == 255 { continue; }
            let played_suit = Suit::from_index(i);
            if played_suit != self.trump_suit {
                continue;
            }
            let played_rank = Rank::from_index(i);
            trump_to_beat = std::cmp::max(trump_to_beat, played_rank);
        }

        // If you can plan in suit then you must
        if hand & lead_suit.mask() > 0 {
            return hand & lead_suit.mask();
        }

        // Can consider reducing branching in the future
        // Assuming you have some trump you must play it
        if hand & self.trump_suit.mask() > 0 {
            // Must head (beat) trump when possible
            if hand & self.trump_suit.mask() & trump_to_beat.higher_global_mask() > 0 {
                return hand & self.trump_suit.mask() & trump_to_beat.higher_global_mask();
            }
            // Otherwise you still have to play some trump
            return hand & self.trump_suit.mask();
        }

        // You can't play in suit or trump
        hand
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Play(_) => {}
            Action::Bid(_) => {}
        }
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    use std::mem;
    use crate::types::Card;

    #[test]
    fn test_memory_layout_size_and_alignment() {
        // 1. Verify exact size is 64 bytes (Cache Line)
        assert_eq!(
            mem::size_of::<PinochleState>(),
            64,
            "PinochleState must be exactly 64 bytes to fit in a cache line"
        );

        // 2. Verify alignment is 64 bytes
        assert_eq!(
            mem::align_of::<PinochleState>(),
            64,
            "PinochleState must be aligned to 64 bytes"
        );
    }

    #[test]
    fn test_derive_copy() {
        // Ensure the struct is actually Copy (stack efficient)
        let s1 = PinochleState::new();
        let s2 = s1; // Copy happens here

        // If s1 is still usable, Copy is implemented correctly
        assert_eq!(s1.turn, s2.turn);
    }

    #[test]
    fn test_legal_moves() {
        let hand = 0b000000000000_000000000000_000000000000_100000000001;
        let s1 = PinochleState {
            hands: [hand, 0, 0, 0],
            dead_cards: [0, 0],
            trick_cards: [0, 1, 2, 3],
            scores: [0, 0],
            active_meld: [0, 0],
            trump_suit: Suit::Spades,
            turn: 0,
            leader: 1,
            _padding: [0; 3],
        };
        let moves = s1.legal_moves();

        assert_eq!(moves, 0b000000000000_000000000000_000000000000_100000000000)
    }
}