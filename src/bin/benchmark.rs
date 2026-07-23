use std::hint::black_box;
use std::time::Instant;
use rayon::prelude::*;
use zysk_pinochle::{new_hand, apply_action, is_hand_over, Action, Phase, Player, RandomPlayer};

fn simulate_one_game() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];

    let mut state = black_box(new_hand());

    while state.phase == Phase::Bidding {
        let player = players[state.turn as usize];
        let bid = player.bid(&state);
        state = apply_action(state, Action::Bid(bid));
    }

    while !is_hand_over(&state) {
        let player = players[state.turn as usize];
        let card = player.play(&state);
        state = apply_action(state, Action::Play(card));
    }

    black_box(state);
}

fn main() {
    const GAMES: usize = 50_000;

    println!("Benchmarking {} games...", GAMES);
    let start = Instant::now();

    (0..GAMES).into_par_iter().for_each(|_| {
        simulate_one_game();
    });

    let elapsed = start.elapsed();
    let per_game = elapsed / GAMES as u32;
    let per_second = GAMES as f64 / elapsed.as_secs_f64();

    println!("Total: {:?}", elapsed);
    println!("Per game: {:?}", per_game);
    println!("Throughput: {:.0} games/sec", per_second);
}
