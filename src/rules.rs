use crate::card;
use crate::meld::hand_meld;
use crate::state::{team, GameState};
use crate::types::{Action, Phase, Rank, Suit, NO_CARD, NO_PLAYER};

const CARD_POINTS: [u8; 6] = [0, 2, 3, 4, 10, 11]; // Nine, Jack, Queen, King, Ten, Ace

#[inline(always)]
pub fn card_points(rank: Rank) -> u8 {
    CARD_POINTS[rank.power_index()]
}

pub fn legal_moves(state: &GameState) -> u64 {
    debug_assert!(state.phase == Phase::TrickTaking, "legal_moves called outside TrickTaking");

    let hand = state.hands[state.turn as usize];

    // Leading player: any card in hand is legal
    if state.leader == NO_PLAYER || state.leader == state.turn {
        return hand;
    }

    let lead_card_index = state.trick_cards[state.leader as usize];
    debug_assert!(lead_card_index != NO_CARD, "leader has not played a card yet");

    let lead_suit = Suit::from_index(lead_card_index);

    // Must follow suit if possible
    let follow_suit = hand & lead_suit.mask();
    if follow_suit != 0 {
        return follow_suit;
    }

    // Can't follow lead suit: check if we have trump
    let trump_mask = state.trump_suit.mask();
    let my_trump = hand & trump_mask;

    if my_trump != 0 {
        // Must play trump, and must beat current best trump if possible
        let best = best_trump(state);
        let must_beat = best.higher_global_mask() & trump_mask;
        let can_beat = my_trump & must_beat;
        if can_beat != 0 {
            return can_beat;
        }
        return my_trump;
    }

    // Can't follow suit, no trump: any card is legal
    hand
}

pub fn best_trump(state: &GameState) -> Rank {
    let trump_mask = state.trump_suit.mask();
    let mut best = Rank::Nine;

    for i in 0..4 {
        let c = state.trick_cards[i];
        if c == NO_CARD {
            continue;
        }
        let mask = card::card_mask(c);
        if mask & trump_mask != 0 {
            let rank = Rank::from_index(c);
            if rank > best {
                best = rank;
            }
        }
    }

    best
}

pub fn trick_winner(state: &GameState) -> (u8, u16) {
    let leader = state.leader;
    let lead_card = state.trick_cards[leader as usize];
    let lead_suit = Suit::from_index(lead_card);
    let trump = state.trump_suit;

    let mut winner = leader;
    let mut best_rank = Rank::from_index(lead_card);
    let mut best_is_trump = lead_suit == trump;
    let mut best_is_lead = true;

    let mut points = 0u16;

    for p in 0..4 {
        let card_idx = state.trick_cards[p as usize];
        let card_rank = Rank::from_index(card_idx);
        let card_suit = Suit::from_index(card_idx);
        let is_trump = card_suit == trump;
        let is_lead = card_suit == lead_suit;
        points += card_points(card_rank) as u16;

        let beats = if is_trump {
            !best_is_trump || card_rank > best_rank
        } else if is_lead {
            !best_is_trump && (!best_is_lead || card_rank > best_rank)
        } else {
            false
        };

        if beats {
            winner = p;
            best_rank = card_rank;
            best_is_trump = is_trump;
            best_is_lead = is_lead;
        }
    }

    // Last trick bonus
    if state.tricks_played == 11 {
        points += 10;
    }

    (winner, points)
}

pub fn play_card(state: &mut GameState, card_index: u8) {
    debug_assert!(state.phase == Phase::TrickTaking, "play_card called outside TrickTaking phase");
    debug_assert!(card_index < 48, "card_index {} out of range 0-47", card_index);
    debug_assert!(state.hands[state.turn as usize] & card::card_mask(card_index) != 0, "card {} not in player {} hand", card_index, state.turn);

    let mask = card::card_mask(card_index);
    state.hands[state.turn as usize] &= !mask;
    state.trick_cards[state.turn as usize] = card_index;

    // Advance turn
    let next_player = (state.turn + 1) % 4;

    // Check if trick is complete
    if next_player == state.leader {
        let (winner, points) = trick_winner(state);
        let winner_team = team(winner);
        state.trick_points[winner_team] += points;
        state.trick_cards = [NO_CARD; 4];
        state.tricks_played += 1;
        state.leader = winner;
        state.turn = winner;

        if state.tricks_played == 12 {
            end_hand(state);
        }
    } else {
        state.turn = next_player;
    }
}

pub fn min_bid(state: &GameState) -> u16 {
    if state.current_bid == 0 {
        15
    } else {
        state.current_bid + 5
    }
}

pub fn make_bid(state: &mut GameState, bid: u16) {
    debug_assert!(state.phase == Phase::Bidding, "make_bid called outside Bidding phase");
    debug_assert!(bid == 0 || (bid <= 250 && bid % 5 == 0), "invalid bid {}", bid);

    if bid == 0 {
        state.pass_count += 1;
        // All 4 pass with no bid: redeal (preserve accumulated scores)
        if state.pass_count >= 4 && state.current_bid == 0 {
            let scores = state.scores;
            *state = crate::state::new_hand();
            state.scores = scores;
            return;
        }
        // 3 consecutive passes after a non-zero bid ends bidding
        if state.pass_count >= 3 && state.current_bid > 0 {
            state.phase = Phase::TrickTaking;
            state.trump_suit = Suit::Spades; // scaffold: default trump
            // Compute meld from hands before trick-taking begins
            state.meld_scores[0] = 0;
            state.meld_scores[1] = 0;
            for p in 0..4 {
                let meld = hand_meld(state.hands[p], state.trump_suit);
                state.meld_scores[team(p as u8)] += meld;
            }
            // Player to left of declarer leads the first trick
            state.turn = (state.declarer + 1) % 4;
            state.leader = state.turn;
        } else {
            state.turn = (state.turn + 1) % 4;
        }
    } else {
        state.current_bid = bid;
        state.declarer = state.turn;
        state.pass_count = 0;
        state.turn = (state.turn + 1) % 4;
    }
}

pub fn do_action(state: &mut GameState, action: Action) {
    match action {
        Action::Bid(bid) => make_bid(state, bid),
        Action::Play(card_index) => play_card(state, card_index),
    }
}

pub fn end_hand(state: &mut GameState) {
    debug_assert!(state.declarer != NO_PLAYER, "end_hand called with no declarer");
    // Meld was pre-computed at the bidding→trick transition (make_bid).
    // Add trick points + meld to total scores
    for t in 0..2 {
        let total = state.trick_points[t] + state.meld_scores[t];
        // Check if declarer's team made their bid
        if team(state.declarer) == t {
            if total >= state.current_bid {
                state.scores[t] += total as i16;
            } else {
                state.scores[t] -= state.current_bid as i16;
            }
        } else {
            state.scores[t] += total as i16;
        }
    }

    state.phase = Phase::Finished;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_hand;

    #[test]
    fn test_card_points() {
        assert_eq!(card_points(Rank::Nine), 0);
        assert_eq!(card_points(Rank::Jack), 2);
        assert_eq!(card_points(Rank::Queen), 3);
        assert_eq!(card_points(Rank::King), 4);
        assert_eq!(card_points(Rank::Ten), 10);
        assert_eq!(card_points(Rank::Ace), 11);
    }

    #[test]
    fn test_legal_moves_leader() {
        // Set up a trick-taking state manually
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.turn = 0;
        state.leader = 0;
        // Leading player: all cards in hand should be legal
        let moves = legal_moves(&state);
        assert_eq!(moves, state.hands[0]);
    }

    #[test]
    fn test_legal_moves_follow_suit() {
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.leader = 0;
        state.turn = 1;
        state.trump_suit = Suit::Hearts;

        // Give player 1 a specific hand with spades
        state.hands[1] = card::card_mask(0) | card::card_mask(1) | card::card_mask(12); // 2 spades, 1 club
        // Player 0 led with a spade
        state.trick_cards[0] = 2; // Jack of Spades copy 0
        state.hands[0] = 0; // Clear for simplicity

        let moves = legal_moves(&state);
        // Must follow spades (indices 0, 1)
        assert_eq!(moves, card::card_mask(0) | card::card_mask(1));
    }

    #[test]
    fn test_legal_moves_must_trump() {
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.leader = 0;
        state.turn = 1;
        state.trump_suit = Suit::Hearts;

        // Player 1 has only hearts (trump) and clubs, no spades
        state.hands[1] = card::card_mask(24) | card::card_mask(25) | card::card_mask(12); // 2 hearts, 1 club
        // Player 0 led with a spade
        state.trick_cards[0] = 0; // Nine of Spades
        state.hands[0] = 0;

        let moves = legal_moves(&state);
        // No spades, has trump — must play trump
        assert_eq!(moves, card::card_mask(24) | card::card_mask(25));
    }

    #[test]
    fn test_legal_moves_no_trump() {
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.leader = 0;
        state.turn = 1;
        state.trump_suit = Suit::Hearts;

        // Player 1 has only clubs, no spades, no hearts
        state.hands[1] = card::card_mask(12) | card::card_mask(13);
        // Player 0 led with a spade
        state.trick_cards[0] = 0;
        state.hands[0] = 0;

        let moves = legal_moves(&state);
        // Can't follow, no trump: any card
        assert_eq!(moves, state.hands[1]);
    }

    #[test]
    fn test_min_bid() {
        let mut state = new_hand();
        assert_eq!(min_bid(&state), 15);
        state.current_bid = 20;
        assert_eq!(min_bid(&state), 25);
    }

    #[test]
    fn test_apply_bid_pass() {
        let mut state = new_hand();
        make_bid(&mut state, 0);
        assert_eq!(state.turn, 1);
        assert_eq!(state.pass_count, 1);
        assert_eq!(state.phase, Phase::Bidding);
    }

    #[test]
    fn test_apply_bid_bid() {
        let mut state = new_hand();
        make_bid(&mut state, 20);
        assert_eq!(state.current_bid, 20);
        assert_eq!(state.declarer, 0);
        assert_eq!(state.pass_count, 0);
        assert_eq!(state.turn, 1);
    }

    #[test]
    fn test_bidding_ends() {
        let mut state = new_hand();
        state.current_bid = 20;
        state.declarer = 0;
        state.pass_count = 2;
        state.turn = 2;
        // Third consecutive pass should end bidding
        make_bid(&mut state, 0);
        assert_eq!(state.phase, Phase::TrickTaking);
        // Player to left of declarer leads
        assert_eq!(state.turn, 1);
        assert_eq!(state.leader, 1);
    }

    #[test]
    fn test_evaluate_trick_trump_wins() {
        let mut state = new_hand();
        state.trump_suit = Suit::Hearts;
        state.leader = 0;

        // Player 0 leads with Nine of Spades
        state.trick_cards[0] = 0; // Nine of Spades
        // Player 1 plays Nine of Hearts (trump!)
        state.trick_cards[1] = 24; // Nine of Hearts
        // Player 2 plays Ace of Spades (lead suit, higher)
        state.trick_cards[2] = 10; // Ace of Spades
        // Player 3 plays King of Spades
        state.trick_cards[3] = 6; // King of Spades

        let (winner, points) = trick_winner(&state);
        assert_eq!(winner, 1); // Player 1's trump wins
        // Points: 0 (Nine) + 0 (Nine) + 11 (Ace) + 4 (King) = 15
        assert_eq!(points, 15);
    }

    #[test]
    fn test_evaluate_trick_no_trump() {
        let mut state = new_hand();
        state.trump_suit = Suit::Hearts;
        state.leader = 0;

        // All spades, no trump
        state.trick_cards[0] = 0;  // Nine of Spades
        state.trick_cards[1] = 2;  // Jack of Spades
        state.trick_cards[2] = 4;  // Queen of Spades
        state.trick_cards[3] = 6;  // King of Spades

        let (winner, points) = trick_winner(&state);
        assert_eq!(winner, 3); // Player 3 has King, highest of lead suit
        // Points: 0 + 2 + 3 + 4 = 9
        assert_eq!(points, 9);
    }

    #[test]
    fn test_apply_play_single_card() {
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.turn = 0;
        state.leader = 0;

        let hand_before = state.hands[0];
        let card_to_play = card::iter_cards(hand_before).next().unwrap();

        play_card(&mut state, card_to_play);
        assert_eq!(state.trick_cards[0], card_to_play);
        assert_eq!(state.hands[0] & card::card_mask(card_to_play), 0);
        assert_eq!(state.turn, 1); // Next player
    }

    #[test]
    fn test_full_trick_playthrough() {
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.turn = 0;
        state.leader = 0;
        state.trump_suit = Suit::Spades;
        state.declarer = 0; // Must be set for end_hand

        // Play all 12 tricks (48 cards)
        for _trick in 0..12 {
            for _player in 0..4 {
                let moves = legal_moves(&state);
                assert_ne!(moves, 0, "No legal moves at trick {}, player {}", _trick, state.turn);
                let card = card::iter_cards(moves).next().unwrap();
                play_card(&mut state, card);
            }
        }

        assert_eq!(state.phase, Phase::Finished);
        assert_eq!(state.tricks_played, 12);
        // Total trick points should be 250 (240 in cards + 10 last trick)
        assert_eq!(state.trick_points[0] + state.trick_points[1], 250);
    }
}
