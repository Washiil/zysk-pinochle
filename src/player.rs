use crate::rules;
use crate::state::GameState;
use rand::Rng;

#[cfg(test)]
use crate::card;

pub trait Player {
    fn bid(&self, state: &GameState) -> u16;
    fn play(&self, state: &GameState) -> u8;
}

pub struct RandomPlayer;

impl Player for RandomPlayer {
    fn bid(&self, state: &GameState) -> u16 {
        let rng = &mut rand::rng();
        if rng.random_bool(0.3) || state.current_bid >= 250 {
            0
        } else {
            let minimum = rules::min_bid(state);
            minimum
        }
    }

    fn play(&self, state: &GameState) -> u8 {
        let legal = rules::legal_moves(state);
        debug_assert!(legal != 0, "no legal moves");

        let rng = &mut rand::rng();
        let count = legal.count_ones();
        let pick = rng.random_range(0..count);

        let mut remaining = legal;
        for _ in 0..pick {
            remaining &= remaining - 1;
        }
        remaining.trailing_zeros() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_hand;
    use crate::types::Phase;

    #[test]
    fn test_random_player_bid() {
        let player = RandomPlayer;
        let state = new_hand();
        let bid = player.bid(&state);
        // Either 0 (pass) or >= 15
        assert!(bid == 0 || bid >= 15);
    }

    #[test]
    fn test_random_player_play() {
        let player = RandomPlayer;
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.leader = 0;
        state.turn = 0;

        let card = player.play(&state);
        // Card must be in player's hand
        assert_ne!(state.hands[0] & card::card_mask(card), 0);
    }

    #[test]
    fn test_random_player_plays_all_legal() {
        let player = RandomPlayer;
        let mut state = new_hand();
        state.phase = Phase::TrickTaking;
        state.turn = 0;
        state.leader = 0;

        for _ in 0..20 {
            let card = player.play(&state);
            let legal = rules::legal_moves(&state);
            assert_ne!(legal & card::card_mask(card), 0, "played illegal card");
        }
    }
}
