use crate::game::PinochleState;
use crate::types::{Action, Card, GamePhase, Player};
use rand::Rng;

pub trait Agent {
    /// The engine asks the agent to select a move given the current state.
    fn get_action(&mut self, state: &PinochleState) -> Action;

    /// Inform agent of the result
    fn on_game_over(&mut self, _final_state: &PinochleState) {}

    /// Identifier often with an ID
    fn name(&self) -> &str {
        "Unknown Agent"
    }
}

pub struct RandomAgent {
    rng: rand::rngs::ThreadRng,
}

impl RandomAgent {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
        }
    }
}

impl Default for RandomAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for RandomAgent {
    fn get_action(&mut self, state: &PinochleState) -> Action {
        match state.phase {
            GamePhase::Bidding => {
                // Random bid between current_bid + 5 and min(current_bid + 5)
                // Or pass (represented as bid of 0)
                if state.current_bid >= 250 || self.rng.random_bool(0.3) {
                    Action::Bid(0) // Pass
                } else {
                    let min_bid = if state.current_bid == 0 {
                        15
                    } else {
                        state.current_bid + 5
                    };
                    Action::Bid(min_bid)
                }
            }
            GamePhase::TrickTaking => {
                let legal_moves = state.legal_moves(state.turn);

                if legal_moves == 0 {
                    panic!("This is undefined and unreachable behavior.")
                } else {
                    // Select random legal move
                    let mut moves = Vec::with_capacity(12);
                    for i in 0..48 {
                        if legal_moves & (1u64 << i) != 0 {
                            moves.push(i);
                        }
                    }

                    let card_index = moves[self.rng.random_range(0..moves.len())] as u8;
                    Action::Play(Card::from_index(card_index))
                }
            }
            GamePhase::Finished => {
                // Game is over, this shouldn't be called
                Action::Bid(0)
            }
        }
    }

    fn on_game_over(&mut self, _final_state: &PinochleState) {
        // Random agent has no learning
        dbg!("Random Agent knows the game is over");
    }

    fn name(&self) -> &str {
        "Random Agent"
    }
}

#[cfg(test)]
mod agent_tests {
    use super::*;

    #[test]
    fn test_random_agent_creation() {
        let agent = RandomAgent::new();
        assert_eq!(agent.name(), "Random Agent");
    }

    #[test]
    fn test_random_agent_bidding() {
        let mut agent = RandomAgent::new();
        let state = PinochleState::new();

        for _ in 0..10 {
            let action = agent.get_action(&state);
            match action {
                Action::Bid(bid) => {
                    assert_eq!(bid % 5, 0);
                }
                _ => panic!("Expected bid action in bidding phase"),
            }
        }
    }

    #[test]
    fn test_random_agent_playing() {
        let mut agent = RandomAgent::new();
        let mut state = PinochleState::new();
        state.phase = GamePhase::TrickTaking;
        state.leader = Some(Player::One);
        state.turn = Player::One;

        let action = agent.get_action(&state);
        match action {
            Action::Play(card) => {
                let hand = state.hands[state.turn as usize];
                assert_ne!(hand & card.mask(), 0, "Agent must play card from hand");
            }
            _ => panic!("Expected play action in trick-taking phase"),
        }
    }
}