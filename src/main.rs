use zysk_pinochle::agent::{Agent};
use zysk_pinochle::game::PinochleState;

fn main() {
    println!("zysk Pinochle Bot");

    let game = PinochleState::new();
    println!("{:?}", game);
}
