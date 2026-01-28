use rand::seq::SliceRandom;
use log::debug;
use crate::types::{Action, Rank, Suit, Player, GamePhase};

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinochleState {
    // 32 Bytes
    // hands[0] = Player 0. Bitboard (bits 0-47 set for cards held)
    pub hands: [u64; 4],

    // 4 Bytes
    // Cards currently on the table.
    // Index matches Player ID. 255 = No card played.
    pub trick_cards: [u8; 4],

    // 4 Bytes
    // Global Game Score (e.g., towards 1500)
    // Using i16 allows for negative scores if a team goes set.
    pub scores: [i16; 2],

    // 4 Bytes
    // Points taken in tricks *this hand only* (Counters + Last Trick Bonus)
    // Needed to verify against the bid at the end.
    pub trick_points: [u16; 2],

    // 4 Bytes
    // Meld points declared at start of hand.
    // Changed to i16 because meld can exceed 255 (e.g., 300 for Double Marriage).
    pub meld_score: [u16; 2],

    // 2 Bytes
    // The winning bid amount (e.g., 50, 60, ... 500)
    pub current_bid: u16,

    // 5 Bytes (Control Flags)
    pub trump_suit: Suit,     // 0-3 (Consider 255 for "No Trump Selected Yet")
    pub turn: Player,         // 0-3 (Whose action is it?)
    pub leader: Option<Player>,       // 0-3 (Who led the current trick?)
    pub bid_winner: Option<Player>,   // 0-3 (Who won the bid? 255 if bidding in progress)
    pub phase: GamePhase,        // Enum discriminant (Bidding, Passing, Melding, Playing)

    pub tricks_played: u8,

    // Remaining 8 Bytes
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
            tricks_played: 0,
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
            dbg!("No legal moves in bidding phase");
            return 0;
        }

        if self.trick_cards[turn as usize] != 255 {
            dbg!("Player has already played a card");
            return 0;
        }

        let Some(leader) = self.leader else {
            dbg!("There must be a leader to start the trick");
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
        for &i in &self.trick_cards {
            // Ignore unplayed cards
            if i == 255 { continue; }
            let played_suit = Suit::from_index(i);
            if played_suit != self.trump_suit {
                continue;
            }
            let played_rank = Rank::from_index(i);
            trump_to_beat = std::cmp::max(trump_to_beat, played_rank);
        }

        // If you can play in suit then you must
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

    fn score_trick(&mut self) -> Option<Player> {
        let start_player = self.leader?;
        let start_index = start_player as usize;

        let lead_card = self.trick_cards[start_index];
        let mut best_suit = Suit::from_index(lead_card);
        let mut best_rank = Rank::from_index(lead_card);

        let mut winning_player = start_player;
        let mut points = 0;

        if lead_card % 12 >= 6 {
            points += 1;
        }

        for i in 1..=3 {
            let current_index = (start_index + i) % 4;
            let card = self.trick_cards[current_index];

            let suit = Suit::from_index(card);
            let rank = Rank::from_index(card);

            if card % 12 > 6 { points += 1 }

            let is_trump = suit == self.trump_suit;
            let best_is_trump = best_suit == self.trump_suit;

            // First trump card being played
            if is_trump && !best_is_trump {
                best_suit = suit;
                best_rank = rank;
                winning_player = Player::from_usize(current_index).unwrap();
            }
            // Check if it beats current rank
            else if suit == best_suit {
                if rank > best_rank {
                    best_rank = rank;
                    winning_player = Player::from_usize(current_index).unwrap();
                }
            }
        }

        let team_idx = (winning_player as usize) % 2;
        self.trick_points[team_idx] += points;

        self.trick_cards = [255; 4];
        self.leader = Some(winning_player);
        self.turn = winning_player;
        self.tricks_played += 1;

        Some(winning_player)
    }

    fn finalize_hand(&mut self) {
        let Some(bid_winner) = self.bid_winner else {
            return;
        };

        let bidding_team = (bid_winner as usize) % 2;
        let defending_team = 1 - bidding_team;

        self.trick_points[self.leader.unwrap() as usize % 2] += 1;
        let total_points = self.meld_score[bidding_team] + self.trick_points[bidding_team];

        if total_points >= self.current_bid {
            // Made the bid
            self.scores[bidding_team] += total_points as i16;
        } else {
            // In the hole
            self.scores[bidding_team] -= self.current_bid as i16;
        }
        self.scores[defending_team] += (self.meld_score[defending_team] + self.trick_points[defending_team]) as i16;
    }

    pub fn is_terminal(&self) -> bool {
        self.phase == GamePhase::Finished
    }

    pub fn winning_team(&self) -> Option<usize> {
        if self.scores[0] >= 200 {
            Some(0)
        } else if self.scores[1] >= 200 {
            Some(1)
        } else {
            None
        }
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

                self.hands[player as usize] &= !card.mask();
                self.trick_cards[player as usize] = card.to_index();

                // Check if trick is complete
                if self.trick_cards.iter().all(|&x| x != 255) {
                    self.score_trick();

                    // Check if hand is finished (12 tricks played)
                    if self.tricks_played >= 12 {
                        self.phase = GamePhase::Finished;
                        self.finalize_hand();
                    }
                } else {
                    self.turn = Player::from_usize((player as usize + 1) % 4).unwrap();
                }

                true
            }
            Action::Bid(bid) => {
                if self.phase != GamePhase::Bidding {
                    return false
                };

                if bid == 0 {
                    // Get the next player
                    self.turn = Player::from_usize((player as usize + 1) % 4).unwrap();
                    return true;
                }

                let min_bid = if self.current_bid == 0 { 15 } else { self.current_bid + 5 };
                if bid < min_bid || bid > 250 || bid % 5 != 0 {
                    return false;
                }

                self.current_bid = bid;
                self.bid_winner = Some(player);
                self.turn = Player::from_usize((player as usize + 1) % 4).unwrap();

                // Simple bidding: after one round, start trick-taking
                if self.turn == Player::One && self.bid_winner.is_some() {
                    self.phase = GamePhase::TrickTaking;
                    self.turn = self.bid_winner.unwrap();
                    self.leader = Some(self.turn);
                }

                true
            }
        }
    }
}

impl Default for PinochleState {
    fn default() -> Self {
        Self::new()
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
        s1.leader = Some(Player::Four);

        s1.trick_cards = [
            255, 255, 4, 3
        ];

        assert_eq!(s1.legal_moves(Player::One),   s1.hands[Player::One as usize] & Suit::Spades.mask());
        assert_eq!(s1.legal_moves(Player::Two),   s1.hands[Player::Two as usize] & Suit::Spades.mask());
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
            0, 255, 2, 3
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

    #[test]
    fn test_score_trick() {
        let mut s1 = PinochleState::new();
        s1.trump_suit = Suit::Spades;
        s1.phase = GamePhase::TrickTaking;
        s1.leader = Some(Player::One);

        s1.trick_cards = [
            Card::new(Rank::Ace, Suit::Hearts).to_index(),
            Card::new(Rank::Ten, Suit::Hearts).to_index(),
            Card::new(Rank::Nine, Suit::Hearts).to_index(),
            Card::new(Rank::Ace, Suit::Hearts).to_index()
        ];

        assert_eq!(s1.score_trick(), Some(Player::One));

        s1.trick_cards = [
            Card::new(Rank::Ace, Suit::Hearts).to_index(),
            Card::new(Rank::Ten, Suit::Hearts).to_index(),
            Card::new(Rank::Nine, Suit::Hearts).to_index(),
            Card::new(Rank::Ace, Suit::Spades).to_index()
        ];

        assert_eq!(s1.score_trick(), Some(Player::Four));
    }
}