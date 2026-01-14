use rand::seq::SliceRandom;
use log::debug;
use crate::types::{Action, Rank, Suit, Player, GamePhase};

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinochleState {
    // --- 32 Bytes ---
    // hands[0] = Player 0. Bitboard (bits 0-47 set for cards held)
    pub hands: [u64; 4],

    // --- 4 Bytes ---
    // Cards currently on the table.
    // Index matches Player ID. 255 = No card played.
    pub trick_cards: [u8; 4],

    // --- 4 Bytes ---
    // Global Game Score (e.g., towards 1500)
    // Using i16 allows for negative scores if a team goes set.
    pub scores: [i16; 2],

    // --- 4 Bytes ---
    // Points taken in tricks *this hand only* (Counters + Last Trick Bonus)
    // Needed to verify against the bid at the end.
    pub trick_points: [u16; 2],

    // --- 4 Bytes ---
    // Meld points declared at start of hand.
    // Changed to i16 because meld can exceed 255 (e.g., 300 for Double Marriage).
    pub meld_score: [i16; 2],

    // --- 2 Bytes ---
    // The winning bid amount (e.g., 50, 60, ... 500)
    pub current_bid: u16,

    // --- 5 Bytes (Control Flags) ---
    pub trump_suit: Suit,     // 0-3 (Consider 255 for "No Trump Selected Yet")
    pub turn: Player,         // 0-3 (Whose action is it?)
    pub leader: Option<Player>,       // 0-3 (Who led the current trick?)
    pub bid_winner: Option<Player>,   // 0-3 (Who won the bid? 255 if bidding in progress)
    pub phase: GamePhase,        // Enum discriminant (Bidding, Passing, Melding, Playing)

    // --- Remaining Bytes ---
    // 9 bytes of padding automatically added here to reach 64-byte alignment
}

impl PinochleState {
    pub fn new() -> Self {
        let starting_hands = Self::deal();

        Self {
            hands: starting_hands,
            trick_cards: [255; 4],
            scores: [0, 0],
            trick_points: [0, 0],
            meld_score: [0, 0],
            current_bid: 0,
            trump_suit: Suit::Spades,
            turn: Player::One,
            leader: None,
            bid_winner: None,
            phase: GamePhase::Bidding,
        }
    }

    /// Deals cards from a standard deck into 4 piles "randomly"
    fn deal() -> [u64; 4] {
        let mut rng = rand::rng();

        // Create array on stack (no heap allocation)
        let mut deck: [u8; 48] = [0; 48];
        for i in 0..48 { deck[i] = i as u8; }

        deck.shuffle(&mut rng);

        let mut hands = [0u64; 4];

        for i in 0..48 {
            let card_index = deck[i];
            let player = i / 12;

            hands[player] |= 1u64 << card_index;
        }

        hands
    }

    /// Computes the legal moves for a given game state
    pub fn legal_moves(&self, turn: Player) -> u64 {
        if self.phase == GamePhase::Bidding {
            return 0;
        }

        let Some(leader) = self.leader else {
            return 0;
        };

        let hand = self.hands[turn as usize];

        if leader == turn {
            return hand;
        }

        let lead_card_idx = self.trick_cards[leader as usize];
        if lead_card_idx > 48 {
            dbg!("Leader has not played a card yet! {:?}", self.trick_cards);
            return 0;
        }

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

    pub fn apply(&mut self, player: Player, action: Action) -> bool {
        if player != self.turn {
            return false
        }

        match action {
            Action::Play(card) => {
                // Ensure they have the card
                let legal_moves = self.legal_moves(self.turn);
                if legal_moves & card.mask() == 0 {
                    return false
                }



                true
            }
            Action::Bid(bid) => {
                true
            }
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
    fn test_legal_moves_leader() {
        let mut s1 = PinochleState::new();
        s1.hands = [
        //  |Diamonds     |Hearts      |Clubs       |Spades
            0b000000000111_111000000000_111000000000_000000000111, // Player::One
            0b000000111000_000111000000_000111000000_000000111000,
            0b000111000000_000000111000_000000111000_000111000000,
            0b111000000000_000000000111_000000000111_111000000000  // Player:: Four
        //   47           35           23           11          0
        ];

        s1.trump_suit = Suit::Spades;
        s1.phase = GamePhase::TrickTaking;
        s1.leader = Some(Player::One);
        let moves = s1.legal_moves(Player::One);

        assert_eq!(moves, s1.hands[Player::One as usize]);
        assert_eq!(s1.legal_moves(Player::Two), 0);
        assert_eq!(s1.legal_moves(Player::Three), 0);
        assert_eq!(s1.legal_moves(Player::Four), 0);
    }

    #[test]
    fn test_legal_moves_in_suit() {
        let mut s1 = PinochleState::new();
        s1.hands = [
            //  |Diamonds     |Hearts      |Clubs       |Spades
            0b000000000111_111000000000_111000000000_000000000111, // Player::One
            0b000000111000_000111000000_000111000000_000000111000,
            0b000111000000_000000111000_000000111000_000111000000,
            0b111000000000_000000000111_000000000111_111000000000  // Player:: Four
            //   47           35           23           11          0
        ];

        s1.trump_suit = Suit::Spades;
        s1.phase = GamePhase::TrickTaking;
        s1.leader = Some(Player::Two);

        s1.trick_cards = [
            255, 1, 2, 3
        ];

        assert_eq!(s1.legal_moves(Player::One),   s1.hands[Player::One as usize] & Suit::Spades.mask());
        assert_eq!(s1.legal_moves(Player::Two),   s1.hands[Player::Two as usize]);
        assert_eq!(s1.legal_moves(Player::Three), s1.hands[Player::Three as usize] & Suit::Spades.mask());
        assert_eq!(s1.legal_moves(Player::Four),  s1.hands[Player::Four as usize] & Suit::Spades.mask());
    }

    #[test]
    fn test_legal_moves_no_lead_suit() {
        let mut s1 = PinochleState::new();
        s1.hands = [
            //  |Diamonds     |Hearts      |Clubs       |Spades
            0b000000000111_111000000000_111000000000_000000000111, // Player::One
            0b000000111000_000111000000_000111000000_000000000000,
            0b000111000000_000000111000_000000111000_000111000000,
            0b111000000000_000000000111_000000000111_111000000000  // Player:: Four
            //47           35           23           11          0
        ];

        s1.trump_suit = Suit::Hearts;
        s1.phase = GamePhase::TrickTaking;
        s1.leader = Some(Player::One);

        s1.trick_cards = [
            0, 1, 2, 3
        ];

        assert_eq!(s1.legal_moves(Player::Two), s1.hands[Player::Two as usize] & s1.trump_suit.mask());
    }

    #[test]
    fn test_legal_moves_leading_trump() {
        let mut s1 = PinochleState::new();
        s1.hands = [
            //  |Diamonds     |Hearts      |Clubs       |Spades
            0b000000000111_111000000000_111000000000_000000000111, // Player::One
            0b000000111000_000111000000_000111000000_000000000000,
            0b000111000000_000000111000_000000111000_000111000000,
            0b111000000000_000000000111_000000000111_111000000000  // Player:: Four
            //47           35           23           11          0
        ];

        s1.trump_suit = Suit::Hearts;
        s1.phase = GamePhase::TrickTaking;
        s1.leader = Some(Player::Three);

        s1.trick_cards = [
            34, 255, 27, 255
        ];

        assert_eq!(s1.legal_moves(Player::Four), s1.hands[Player::Four as usize] & s1.trump_suit.mask());
    }
}