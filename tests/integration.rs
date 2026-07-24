use zysk_pinochle::bench::run_game;
use zysk_pinochle::{Player, RandomPlayer, DECK_MASK};

#[test]
fn test_full_game() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];

    // Verify fresh deal
    let state = zysk_pinochle::new_hand();
    let total_cards = state.hands[0] | state.hands[1] | state.hands[2] | state.hands[3];
    assert_eq!(total_cards, DECK_MASK);
    assert_eq!(state.hands[0].count_ones(), 12);
    assert_eq!(state.hands[1].count_ones(), 12);
    assert_eq!(state.hands[2].count_ones(), 12);
    assert_eq!(state.hands[3].count_ones(), 12);

    // Run a full game
    let mut state = zysk_pinochle::new_hand();
    run_game(&mut state, &players);

    assert_eq!(state.tricks_played, 12);
    assert_eq!(state.phase, zysk_pinochle::Phase::Finished);

    // Total trick points should be 250 (240 in 48 cards + 10 last trick)
    let total_trick_points = state.trick_points[0] + state.trick_points[1];
    assert_eq!(total_trick_points, 250);
}

#[test]
fn test_multiple_games() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];

    for _ in 0..100 {
        let mut state = zysk_pinochle::new_hand();
        run_game(&mut state, &players);
        assert_eq!(state.phase, zysk_pinochle::Phase::Finished);
        assert_eq!(state.tricks_played, 12);
        assert_eq!(state.trick_points[0] + state.trick_points[1], 250);
    }
}
