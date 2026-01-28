use zysk_pinochle::agent::{Agent, RandomAgent};
use zysk_pinochle::game::PinochleState;
use zysk_pinochle::types::{Action};

fn main() {
    println!("=== zysk Pinochle Bot ===\n");

    // Create 4 random agents
    let mut agents: [Box<dyn Agent>; 4] = [
        Box::new(RandomAgent::new()),
        Box::new(RandomAgent::new()),
        Box::new(RandomAgent::new()),
        Box::new(RandomAgent::new()),
    ];

    println!("Players:");
    for (i, agent) in agents.iter().enumerate() {
        println!("  Player {} (Team {}): {}", i + 1, (i % 2) + 1, agent.name());
    }
    println!();

    // Run a single game
    run_game(&mut agents);

    println!("\n=== Game Complete ===");
}

fn run_game(agents: &mut [Box<dyn Agent>; 4]) {
    let mut state = PinochleState::new();
    let mut move_count = 0;
    const MAX_MOVES: usize = 1000;
    let mut bidding_over = false;

    println!("Starting game...\n");

    while !state.is_terminal() && move_count < MAX_MOVES {
        let current_player = state.turn;
        let action = agents[current_player as usize].get_action(&state);

        match action {
            Action::Bid(bid) => {
                if bid == 0 {
                    println!(
                        "Player {} passes (current bid: {})",
                        (current_player as usize) + 1,
                        state.current_bid
                    );
                } else {
                    println!(
                        "Player {} bids {} (previous: {})",
                        (current_player as usize) + 1,
                        bid,
                        state.current_bid
                    );
                }
            }
            Action::Play(card) => {
                if !bidding_over {
                    bidding_over = true;
                    dbg!(state);
                }
                if move_count % 4 == 0 && move_count > 0 {
                    println!();
                }
                println!(
                    "Player {} plays {:?} of {:?}",
                    (current_player as usize) + 1,
                    card.rank,
                    card.suit
                );
            }
        }

        if !state.apply(current_player, action) {
            println!("ERROR: Invalid move by Player {}!", (current_player as usize) + 1);
            break;
        }

        move_count += 1;

        // Show trick completion
        if state.phase == zysk_pinochle::types::GamePhase::TrickTaking
            && state.trick_cards.iter().all(|&x| x == 255)
            && move_count > 0
        {
            println!(
                "  -> Player {} wins the trick!",
                (state.turn as usize) + 1
            );
            println!("Tricks Played: {}", state.tricks_played)
        }
    }

    if move_count >= MAX_MOVES {
        println!("\nGame reached maximum move limit!");
    }

    // Final scores
    println!("\n=== Final Scores ===");
    println!("Team 1 (Players 1 & 3): {} points", state.scores[0]);
    println!("Team 2 (Players 2 & 4): {} points", state.scores[1]);

    if let Some(winner) = state.winning_team() {
        println!("\nTeam {} wins!", winner + 1);
    } else {
        println!("\nGame ended without a winner (200+ points needed)");
    }

    // Notify agents of game end
    for agent in agents.iter_mut() {
        agent.on_game_over(&state);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_game_runs_to_completion() {
        let mut agents: [Box<dyn Agent>; 4] = [
            Box::new(RandomAgent::new()),
            Box::new(RandomAgent::new()),
            Box::new(RandomAgent::new()),
            Box::new(RandomAgent::new()),
        ];

        run_game(&mut agents);
    }
}