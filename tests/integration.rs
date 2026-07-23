use zysk_pinochle::{
    new_hand, legal_moves, apply_action, is_hand_over,
    Action, Phase, Player, RandomPlayer, DECK_MASK,
};

#[test]
fn test_full_game() {
    let players: [&dyn Player; 4] = [
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
        &RandomPlayer,
    ];

    let mut state = new_hand();

    // Verify initial state
    let total_cards = state.hands[0] | state.hands[1] | state.hands[2] | state.hands[3];
    assert_eq!(total_cards, DECK_MASK);
    assert_eq!(state.hands[0].count_ones(), 12);
    assert_eq!(state.hands[1].count_ones(), 12);
    assert_eq!(state.hands[2].count_ones(), 12);
    assert_eq!(state.hands[3].count_ones(), 12);

    // Play through bidding
    while state.phase == Phase::Bidding {
        let player = players[state.turn as usize];
        let bid = player.bid(&state);

        // Verify bid legality before applying
        if bid != 0 {
            assert!(bid % 5 == 0);
            if state.current_bid > 0 {
                assert!(bid > state.current_bid);
            }
        }

        state = apply_action(state, Action::Bid(bid));
    }

    // Verify we transitioned correctly
    assert_eq!(state.phase, Phase::TrickTaking);

    // Play all 12 tricks
    let mut cards_played = 0usize;
    while !is_hand_over(&state) {
        let player = players[state.turn as usize];
        let card = player.play(&state);

        // Verify card is legal
        let legal = legal_moves(&state);
        assert_ne!(legal & (1u64 << card), 0, "played illegal card");

        state = apply_action(state, Action::Play(card));
        cards_played += 1;
    }

    // 12 tricks × 4 cards = 48 cards played
    assert_eq!(cards_played, 48);
    assert_eq!(state.tricks_played, 12);
    assert_eq!(state.phase, Phase::Finished);

    // Total trick points should be 250 (240 in 48 cards + 10 last trick)
    let total_trick_points = state.trick_points[0] + state.trick_points[1];
    assert_eq!(total_trick_points, 250);
}

#[test]
fn test_multiple_games() {
    for _ in 0..100 {
        let players: [&dyn Player; 4] = [
            &RandomPlayer,
            &RandomPlayer,
            &RandomPlayer,
            &RandomPlayer,
        ];
        let mut state = new_hand();

        while state.phase == Phase::Bidding {
            let player = players[state.turn as usize];
            let bid = player.bid(&state);
            state = apply_action(state, Action::Bid(bid));
        }

        while !is_hand_over(&state) {
            let player = players[state.turn as usize];
            let card = player.play(&state);
            // Verify legal move
            let legal = legal_moves(&state);
            assert_ne!(legal & (1u64 << card), 0);
            state = apply_action(state, Action::Play(card));
        }

        assert_eq!(state.phase, Phase::Finished);
        assert_eq!(state.tricks_played, 12);
        assert_eq!(state.trick_points[0] + state.trick_points[1], 250);
    }
}
