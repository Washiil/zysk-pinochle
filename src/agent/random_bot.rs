use crate::agent::PinochleAgent;

pub struct RandomBot;

impl PinochleAgent for RandomBot {
    fn choose_bid(&self, _: u64, _: u16) -> u16 { todo!() }
    fn play_card(&self, _: u64, _: &[u16]) -> u16 { todo!() }
}