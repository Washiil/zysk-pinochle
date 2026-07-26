use zysk_pinochle::card::iter_cards;
use zysk_pinochle::rules::do_action;
use zysk_pinochle::state::new_hand;
use zysk_pinochle::types::{Action, Phase, Rank, Suit};
use zysk_pinochle::{Player, RandomPlayer};

fn card_name(index: u8) -> String {
    let suit = Suit::from_index(index);
    let rank = Rank::from_index(index);
    let suit_char = match suit {
        Suit::Spades => '♠',
        Suit::Clubs => '♣',
        Suit::Hearts => '♥',
        Suit::Diamonds => '♦',
    };
    let rank_str = match rank {
        Rank::Nine => "9",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
        Rank::Ten => "10",
        Rank::Ace => "A",
    };
    format!("{}{}", rank_str, suit_char)
}

fn hand_string(hand: u64) -> String {
    let mut cards: Vec<u8> = iter_cards(hand).collect();
    cards.sort_unstable();
    cards.iter().map(|&i| card_name(i)).collect::<Vec<_>>().join(" ")
}

fn suit_symbol(suit: Suit) -> char {
    match suit {
        Suit::Spades => '♠',
        Suit::Clubs => '♣',
        Suit::Hearts => '♥',
        Suit::Diamonds => '♦',
    }
}

fn main() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer, &RandomPlayer, &RandomPlayer, &RandomPlayer,
    ];
    let mut state = new_hand();

    println!("=== HANDS ===");
    for p in 0..4 {
        println!("P{}: {}", p, hand_string(state.hands[p]));
    }

    println!("\n=== BIDDING ===");
    while state.phase == Phase::Bidding {
        let player_idx = state.turn as usize;
        let bid = players[player_idx].bid(&state);
        let prev_hands = state.hands;
        do_action(&mut state, Action::Bid(bid));

        if bid == 0 && state.hands != prev_hands {
            println!("  → Redeal (all passed)");
            println!("\n=== NEW HANDS ===");
            for p in 0..4 {
                println!("P{}: {}", p, hand_string(state.hands[p]));
            }
        } else if bid == 0 {
            println!("P{} passes", player_idx);
        } else {
            println!("P{} bids {}", player_idx, bid);
        }

        if state.phase == Phase::TrickTaking {
            println!(
                "→ Declarer: P{}, Bid: {}, Trump: {}",
                state.declarer,
                state.current_bid,
                suit_symbol(state.trump_suit),
            );
        }
    }

    println!("\n=== TRICK-TAKING ===");
    let mut trick_log: Vec<(u8, u8)> = Vec::with_capacity(4);

    while state.phase != Phase::Finished {
        let player_idx = state.turn as usize;
        let card = players[player_idx].play(&state);
        let prev_tricks = state.tricks_played;
        let prev_points = state.trick_points;

        trick_log.push((state.turn, card));
        do_action(&mut state, Action::Play(card));

        if state.tricks_played > prev_tricks {
            let trick_num = state.tricks_played;
            println!("\n─── Trick {} ───", trick_num);
            for &(p, c) in &trick_log {
                let tp = if Suit::from_index(c) == state.trump_suit {
                    "*"
                } else {
                    " "
                };
                println!("  P{}: {}{}", p, card_name(c), tp);
            }
            let team0_diff = state.trick_points[0] - prev_points[0];
            let team1_diff = state.trick_points[1] - prev_points[1];
            let winner_team = if team0_diff > 0 { 0 } else { 1 };
            let pts = team0_diff.max(team1_diff);
            let p0 = winner_team;
            let p1 = winner_team + 2;
            println!("  → Team {winner_team} (P{p0}+P{p1}) wins (+{pts} pts)");
            trick_log.clear();
        }
    }

    println!("\n=== RESULTS ===");
    println!("Declarer: P{}, Bid: {}", state.declarer, state.current_bid);
    println!("Trump: {}", suit_symbol(state.trump_suit));
    for t in 0..2 {
        let p0 = t;
        let p1 = t + 2;
        let hand_total = state.trick_points[t] + state.meld_scores[t];
        println!(
            "Team {t} (P{p0}+P{p1}): trick={} meld={} hand_total={} → score={}",
            state.trick_points[t],
            state.meld_scores[t],
            hand_total,
            state.scores[t],
        );
    }
}
