pub mod meld;
pub mod trick;
pub mod game_state;
pub use game_state::GameState;

pub enum GamePhase {
    Bidding,
    Meld,
    Play,
}