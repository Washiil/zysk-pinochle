// src/main.rs

use zysk_pinochle::agent::{PinochleAgent, RandomBot};
use zysk_pinochle::core::{Suit, Rank, PinochleHand};
use zysk_pinochle::pinochle::GameState;

use rayon::prelude::*;


fn main() {
    println!("--- Pinochle HPC Engine Initializing ---");

    // Players should be defined using traits so we can easily have them play one another
    let player1 = RandomBot;
    let player2 = RandomBot;
    let player3 = RandomBot;
    let player4 = RandomBot;

    // Run a tournament
    println!("Starting parallel simulation playouts...");

    // Parallel runs
    let results: Vec<i32> = (0..100)
        .into_par_iter()
        .map(|i| {
            if i % 10 == 0 { println!("Match {} started...", i); }

            // This calls our generic, game loop
            run_match(&player1, &player2, &player3, &player4)
        })
        .collect();

    // Output the results of tournaments
    let wins = results.iter().filter(|&&res| res > 0).count();
    println!("--- Tournament Results ---");
    println!("MCTS Bot Win Rate: {}%", wins);
}

/// A generic game runner that can take ANY type of player.
/// This is 'zero-cost' because Rust compiles a specific version 
/// for the exact types passed in.
fn run_match<P1, P2, P3, P4>(p1: &P1, p2: &P2, p3: &P3, p4: &P4) -> i32
where
    P1: PinochleAgent, P2: PinochleAgent, P3: PinochleAgent, P4: PinochleAgent
{
    let mut game = GameState::new();

    // Simple state machine for the game flow
    while !game.is_over() {
        let current_p = game.current_turn();
        let hand = game.get_hand(current_p);

        let card = match current_p {
            0 => p1.play_card(hand, &game.table),
            1 => p2.play_card(hand, &game.table),
            2 => p3.play_card(hand, &game.table),
            _ => p4.play_card(hand, &game.table),
        };

        game.apply_move(card);
    }

    game.get_score(0) // Return score for the team of player 1
}