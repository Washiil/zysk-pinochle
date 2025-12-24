pub mod random_bot;
pub use random_bot::RandomBot;

pub trait PinochleAgent {
    fn choose_bid(&self, hand: u64, current_bid: u16) -> u16;

    fn play_card(&self, hand: u64, current_card: &[u16]) -> u16;
}