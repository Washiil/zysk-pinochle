use zysk_pinochle::bench::run_game;
use zysk_pinochle::{Player, RandomPlayer};

fn main() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];
    let state = run_game(&players);

    println!("Game over!");
    println!("Team 0 (players 0,2): {} points", state.scores[0]);
    println!("Team 1 (players 1,3): {} points", state.scores[1]);
    println!("Declarer: player {}", state.declarer);
    println!("Bid: {}", state.current_bid);
}
