use crate::player::{Player, RandomPlayer};
use crate::rules::do_action;
use crate::state::{new_hand, reset_hand, GameState};
use crate::types::{Action, Phase};
use std::hint::black_box;

/// Run a full game on an existing GameState, mutating it in place.
/// Resets the board first (preserving scores), then plays bidding + 12 tricks.
pub fn run_game(state: &mut GameState, players: &[&dyn Player; 4]) {
    reset_hand(state);

    while state.phase == Phase::Bidding {
        let player = players[state.turn as usize];
        let bid = player.bid(state);
        do_action(state, Action::Bid(bid));
    }

    while state.phase != Phase::Finished {
        let player = players[state.turn as usize];
        let card = player.play(state);
        do_action(state, Action::Play(card));
    }
}

pub fn simulate_one_game() {
    let players: [&dyn Player; 4] = [&RandomPlayer, &RandomPlayer, &RandomPlayer, &RandomPlayer];
    let mut state = black_box(new_hand());
    run_game(&mut state, &players);
    black_box(state);
}

pub fn throughput(games: usize) -> f64 {
    let start = std::time::Instant::now();
    for _ in 0..games {
        simulate_one_game();
    }
    games as f64 / start.elapsed().as_secs_f64()
}
