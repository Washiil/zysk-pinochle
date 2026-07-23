use zysk_pinochle::{new_hand, apply_action, is_hand_over, Action, Phase, Player, RandomPlayer};

fn run_game(players: &[&dyn Player; 4]) {
    let mut state = new_hand();

    // Bidding phase
    while state.phase == Phase::Bidding {
        let player = players[state.turn as usize];
        let bid = player.bid(&state);
        state = apply_action(state, Action::Bid(bid));
    }

    // Trick-taking phase
    while !is_hand_over(&state) {
        let player = players[state.turn as usize];
        let card = player.play(&state);
        state = apply_action(state, Action::Play(card));
    }

    println!("Game over!");
    println!("Team 0 (players 0,2): {} points", state.scores[0]);
    println!("Team 1 (players 1,3): {} points", state.scores[1]);
    println!("Declarer: player {}", state.declarer);
    println!("Bid: {}", state.current_bid);
}

fn main() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];
    run_game(&players);
}
