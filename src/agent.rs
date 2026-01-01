use crate::game::PinochleState;
use rand::rngs::ThreadRng;
use rand::Rng;

pub trait Agent {
    /// The engine asks the agent to select a move given the current state.
    fn get_action(&mut self, state: &PinochleState) -> u8;

    /// Inform agent of the result
    fn on_game_over(&mut self, _final_state: &PinochleState) {}

    /// Identifier often with an ID
    fn name(&self) -> &str { "Unknown Agent" }
}