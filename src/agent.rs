use crate::game::PinochleState;
use rand::rngs::ThreadRng;
use rand::Rng;
use crate::types::{Action, GamePhase};

pub trait Agent {
    /// The engine asks the agent to select a move given the current state.
    fn get_action(&mut self, state: &PinochleState) -> Action;

    /// Inform agent of the result
    fn on_game_over(&mut self, _final_state: &PinochleState) {}

    /// Identifier often with an ID
    fn name(&self) -> &str { "Unknown Agent" }
}

struct RandomAgent;

impl Agent for RandomAgent {
    fn get_action(&mut self, state: &PinochleState) -> Action {
        todo!()
    }

    fn on_game_over(&mut self, _final_state: &PinochleState) {
        // We have no training
        return
    }

    fn name(&self) -> &str {
        "Rando Bot"
    }
}