use crate::player::{Player, RandomPlayer};
use crate::rules::apply_action;
use crate::state::new_hand;
use crate::types::{Action, Phase};
use std::hint::black_box;

pub fn simulate_one_game() {
    let players: [&dyn Player; 4] = [&RandomPlayer, &RandomPlayer, &RandomPlayer, &RandomPlayer];
    let mut state = black_box(new_hand());

    while state.phase == Phase::Bidding {
        let player = players[state.turn as usize];
        let bid = player.bid(&state);
        state = apply_action(state, Action::Bid(bid));
    }

    while state.phase != Phase::Finished {
        let player = players[state.turn as usize];
        let card = player.play(&state);
        state = apply_action(state, Action::Play(card));
    }

    black_box(state);
}

pub fn throughput(games: usize) -> f64 {
    let start = std::time::Instant::now();
    for _ in 0..games {
        simulate_one_game();
    }
    games as f64 / start.elapsed().as_secs_f64()
}
