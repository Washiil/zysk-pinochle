// Shader for pinochle Monte carlo simulation

// # region Structs
struct MctsRolloutInput {
    p0_known: vec2<u32>,
    p1_known: vec2<u32>,
    p2_known: vec2<u32>,
    p3_known: vec2<u32>,

    current_trick_cards: vec2<u32>,
    unseen_pool: vec2<u32>,
    played_out_cards: vec2<u32>,

    scores: vec2<i32>,

    packed_metadata: u32,
    packed_bids: u32,
    packed_constraints: u32,
    packed_meld: u32,
    
    rng_state: vec2<u32>,
}

struct Metadata {
    winning_card_idx: u32,
    winning_player: u32,
    lead_player: u32,
    current_player: u32,
    trump_suit: u32,
    lead_suit: u32,
    tricks_played: u32,
}

struct MctsRolloutOutput {
    team_0_2_score: u32,
    team_1_3_score: u32,
}

// # endregion

fn unpack_metadata(packed: u32) -> Metadata {
    var data: Metadata;
    data.winning_card_idx = (packed >> 0u)  & 0x3Fu;  // 6 bits
    data.winning_player   = (packed >> 6u)  & 0x03u;  // 2 bits
    data.lead_player      = (packed >> 8u)  & 0x03u;  // 2 bits
    data.current_player   = (packed >> 10u) & 0x03u;  // 2 bits
    data.trump_suit       = (packed >> 12u) & 0x03u;  // 2 bits
    data.lead_suit        = (packed >> 14u) & 0x07u;  // 3 bits
    data.tricks_played    = (packed >> 17u) & 0x0Fu;  // 4 bits
    return data;
}

// Gets the 4-bit void suit mask for a specific player (0-3)
// Format: 0bS_H_D_C (Spades, Hearts, Diamonds, Clubs)
fn get_void_suits(packed_constraints: u32, player_id: u32) -> u32 {
    let shift = player_id * 4u;
    return (packed_constraints >> shift) & 0x0Fu;
}

// Gets the exact number of cards the player should be holding (0-12)
fn get_hand_size(packed_constraints: u32, player_id: u32) -> u32 {
    let shift = 16u + (player_id * 4u);
    return (packed_constraints >> shift) & 0x0Fu;
}

fn get_team_a_bid(packed_bids: u32) -> u32 {
    return packed_bids & 0xFFFFu;
}

fn get_team_b_bid(packed_bids: u32) -> u32 {
    return packed_bids >> 16u;
}

// # endregion

// # region Card & game helpers 

// Returns the point value of a card (0-47).
fn card_point(card: u32) -> i32 {
    let r = card % 12u;               // 12 cards per suit
    if      (r < 2u)  { return 0;  }  // 9
    else if (r < 4u)  { return 2;  }  // J
    else if (r < 6u)  { return 3;  }  // Q
    else if (r < 8u)  { return 4;  }  // K
    else if (r < 10u) { return 10; }  // 10
    else              { return 11; }  // A
}

// Choose a legal card from `hand` (bitmask) given the lead suit and trump.
// If player is leading, set lead_suit = 4 (or any value >=4) to mean "any".
// Removes the chosen card from the hand and returns its index (0-47).
fn select_legal_card(
    hand: ptr<function, vec2<u32>>,
    lead_suit: u32,
    trump: u32,
    rng: ptr<function, u32>
) -> u32 {
    let must_follow = lead_suit < 4u;
    var candidates: u32 = 0u;
    var candidate_list: array<u32, 48>;
    let hand_lo = (*hand)[0];
    let hand_hi = (*hand)[1];

    // cards that must follow suit
    for (var i = 0u; i < 48u; i++) {
        let half = i / 32u;
        let bit  = i % 32u;
        let mask = 1u << bit;

        let in_hand = select(
            (hand_hi & mask) != 0u,   // false value (half == 1)
            (hand_lo & mask) != 0u,   // true  value (half == 0)
            half == 0u
        );

        if (in_hand) {
            if (must_follow) {
                let suit = i / 12u;
                if (suit == lead_suit) {
                    candidate_list[candidates] = i;
                    candidates += 1u;
                }
            } else {
                candidate_list[candidates] = i;
                candidates += 1u;
            }
        }
    }

    // if no matching cards, play any card
    if (candidates == 0u) {
        for (var i = 0u; i < 48u; i++) {
            let half = i / 32u;
            let bit  = i % 32u;
            let mask = 1u << bit;

            let in_hand = select(
                (hand_hi & mask) != 0u,
                (hand_lo & mask) != 0u,
                half == 0u
            );

            if (in_hand) {
                candidate_list[candidates] = i;
                candidates += 1u;
            }
        }
    }

    let idx = rand_in_range(rng, candidates);
    let chosen = candidate_list[idx];

    // Remove chosen card from hand
    let half = chosen / 32u;
    let bit  = chosen % 32u;
    let mask = 1u << bit;
    if (half == 0u) {
        (*hand)[0] &= ~mask;
    } else {
        (*hand)[1] &= ~mask;
    }
    return chosen;
}

// Determine the winner of a trick given the four cards played and the trump suit.
// Returns (winner_player, trick_points).
fn evaluate_trick(
    cards: array<u32, 4>,
    lead_player: u32,
    trump: u32
) -> vec2<u32> {
    var best_card: u32 = cards[0];
    var winner: u32 = lead_player;
    let lead_suit = best_card / 12u;
    var total_points: i32 = card_point(best_card);

    for (var i = 1u; i < 4u; i++) {
        let card = cards[i];
        total_points += card_point(card);
        let suit = card / 12u;
        let rank = card % 12u;
        let best_rank = best_card % 12u;

        var beats = false;
        if (suit == trump && (best_card / 12u) != trump) {
            beats = true;               // trump beats non‑trump
        } else if (suit == (best_card / 12u)) {
            // same suit – higher rank wins (higher index within suit = higher rank)
            if (rank > best_rank) {
                beats = true;
            }
        }
        // else: can't beat, off‑suit non‑trump loses
        if (beats) {
            best_card = card;
            winner = (lead_player + i) % 4u;
        }
    }

    return vec2<u32>(winner, u32(total_points));
}

// # endregion


// #region RNG Helpers

/// Fast, stateful RNG. Mutates the seed and returns a random 32-bit integer.
fn next_rand(rng_state: ptr<function, u32>) -> u32 {
    var x = *rng_state;
    x ^= x << 13u;
    x ^= x >> 17u;
    x ^= x << 5u;
    *rng_state = x;
    return x;
}

/// Helper to get a random integer within a range [0, max)
fn rand_in_range(rng_state: ptr<function, u32>, max_val: u32) -> u32 {
    return next_rand(rng_state) % max_val;
}

// #endregion

// Bind group buffers
@group(0) @binding(0) var<storage, read> inputs: array<MctsRolloutInput>;
@group(0) @binding(1) var<storage, read_write> outputs: array<MctsRolloutOutput>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;
    let input = inputs[thread_id];

    var metadata = unpack_metadata(input.packed_metadata);
    var rng = input.rng_state.x;

    // precompute player state
    var void_suits: array<u32, 4>;
    var hand_sizes: array<u32, 4>;
    for (var p = 0u; p < 4u; p++) {
        void_suits[p] = get_void_suits(input.packed_constraints, p);
        hand_sizes[p]  = get_hand_size(input.packed_constraints, p);
    }

    // deal unseen cards
    var hands: array<vec2<u32>, 4>;
    hands[0] = input.p0_known;
    hands[1] = input.p1_known;
    hands[2] = input.p2_known;
    hands[3] = input.p3_known;

    var unseen = input.unseen_pool;

    for (var i = 0u; i < 48u; i++) {
        let half = i / 32u;
        let bit  = i % 32u;
        let mask = 1u << bit;
        if ((unseen[half] & mask) == 0u) { continue; }

        let suit = i / 12u;
        var candidate_mask = 0u;
        var candidate_count = 0u;

        for (var p = 0u; p < 4u; p++) {
            let is_full = (hand_sizes[p] == 0u);
            let is_void = ((void_suits[p] & (1u << suit)) != 0u);
            if (!is_full && !is_void) {
                candidate_mask |= (1u << p);
                candidate_count += 1u;
            }
        }

        // Safety fallback, prevents GPU hang.
        if (candidate_count == 0u) {
            for (var p = 0u; p < 4u; p++) {
                if (hand_sizes[p] > 0u) {
                    candidate_mask |= (1u << p);
                    candidate_count += 1u;
                }
            }
        }

        let choice = rand_in_range(&rng, candidate_count);
        var chosen = 0u;
        var seen = 0u;
        for (var p = 0u; p < 4u; p++) {
            if ((candidate_mask & (1u << p)) != 0u) {
                if (seen == choice) { chosen = p; }
                seen += 1u;
            }
        }

        hands[chosen][half] |= mask;
        hand_sizes[chosen] -= 1u;
    }

    // Simulate remaining tricks
    var scores = input.scores;                // team A=0&2, team B=1&3
    let total_tricks = 12u;
    let completed = metadata.tricks_played;   // already finished
    var leader = metadata.lead_player;

    for (var trick = completed; trick < total_tricks; trick++) {
        var trick_cards: array<u32, 4>;
        var first_suit: u32 = 4u; // 4 = “no suit yet” (leader can play anything)

        for (var turn = 0u; turn < 4u; turn++) {
            let player = (leader + turn) % 4u;
            let chosen = select_legal_card(
                &hands[player],
                first_suit,
                metadata.trump_suit,
                &rng
            );
            trick_cards[turn] = chosen;
            if (turn == 0u) {
                first_suit = chosen / 12u;   // set lead suit for followers
            }
        }

        let result = evaluate_trick(trick_cards, leader, metadata.trump_suit);
        let winner = result.x;
        let points = result.y;

        // Add points to the winning team
        if (winner == 0u || winner == 2u) {
            scores.x += i32(points);
        } else {
            scores.y += i32(points);
        }

        leader = winner;   // winner leads next trick
    }

    // Write output
    outputs[thread_id].team_0_2_score = u32(scores.x);
    outputs[thread_id].team_1_3_score = u32(scores.y);
}