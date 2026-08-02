// Shader for pinochle Monte Carlo simulation

//  Input structs + helpers 
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

fn unpack_metadata(packed: u32) -> Metadata {
    var data: Metadata;
    data.winning_card_idx = (packed >> 0u)  & 0x3Fu;
    data.winning_player   = (packed >> 6u)  & 0x03u;
    data.lead_player      = (packed >> 8u)  & 0x03u;
    data.current_player   = (packed >> 10u) & 0x03u;
    data.trump_suit       = (packed >> 12u) & 0x03u;
    data.lead_suit        = (packed >> 14u) & 0x07u;
    data.tricks_played    = (packed >> 17u) & 0x0Fu;
    return data;
}

fn get_void_suits(packed_constraints: u32, player_id: u32) -> u32 {
    let shift = player_id * 4u;
    return (packed_constraints >> shift) & 0x0Fu;
}

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

struct MctsRolloutOutput {
    team_0_2_score: u32,
    team_1_3_score: u32,
}

//  RNG (unchanged) 
fn next_rand(rng_state: ptr<function, u32>) -> u32 {
    var x = *rng_state;
    x ^= x << 13u;
    x ^= x >> 17u;
    x ^= x << 5u;
    *rng_state = x;
    return x;
}

fn rand_in_range(rng_state: ptr<function, u32>, max_val: u32) -> u32 {
    return next_rand(rng_state) % max_val;
}

//  Card helpers 
fn card_point(card: u32) -> u32 {
    let r = card % 12u;
    if (r < 6u)  { 
        return 0;  
    }  // Q
    else { 
        return 1;  
    }  // K
}

// Result of selecting a legal card: the chosen card index and the hand with that card removed.
struct LegalCardResult {
    chosen: u32,
    updated_hand: vec2<u32>,
}

/// Pick a legal card from `hand` given the lead suit and trump.
/// If the player is leading, set `lead_suit = 4` (or >=4) to allow any card.
/// Removes the chosen card from the returned hand.
fn select_legal_card(
    hand: vec2<u32>,
    lead_suit: u32,
    trump: u32,
    rng: ptr<function, u32>
) -> LegalCardResult {
    let must_follow = lead_suit < 4u;
    var candidate_count: u32 = 0u;

    // First pass: count how many cards we can legally play
    for (var i = 0u; i < 48u; i++) {
        let half = i / 32u;
        let bit  = i % 32u;
        let mask = 1u << bit;
        let in_hand = select(
            (hand[1] & mask) != 0u,   // false case: half == 1
            (hand[0] & mask) != 0u,   // true  case: half == 0
            half == 0u
        );
        if (in_hand) {
            if (must_follow) {
                let suit = i / 12u;
                if (suit == lead_suit) {
                    candidate_count += 1u;
                }
            } else {
                candidate_count += 1u;
            }
        }
    }

    // If we couldn't follow suit, any card is allowed (second pass)
    if (candidate_count == 0u) {
        for (var i = 0u; i < 48u; i++) {
            let half = i / 32u;
            let bit  = i % 32u;
            let mask = 1u << bit;
            let in_hand = select(
                (hand[1] & mask) != 0u,
                (hand[0] & mask) != 0u,
                half == 0u
            );
            if (in_hand) {
                candidate_count += 1u;
            }
        }
    }

    // Random index in [0, candidate_count)
    let target_card = rand_in_range(rng, candidate_count);

    // Second pass: find the target-th candidate and remove it
    var found: u32 = 0u;
    var chosen: u32 = 0u;
    var updated_hand = hand;

    for (var i = 0u; i < 48u; i++) {
        let half = i / 32u;
        let bit  = i % 32u;
        let mask = 1u << bit;
        let in_hand = select(
            (updated_hand[1] & mask) != 0u,
            (updated_hand[0] & mask) != 0u,
            half == 0u
        );
        if (!in_hand) { continue; }

        // Check legality (same logic as counting pass)
        var legal = true;
        if (must_follow) {
            let suit = i / 12u;
            legal = (suit == lead_suit);
        }
        // (If must_follow is false, everything is legal)

        if (legal) {
            if (found == target_card) {
                chosen = i;
                // Remove from hand
                if (half == 0u) {
                    updated_hand[0] &= ~mask;
                } else {
                    updated_hand[1] &= ~mask;
                }
                break;
            }
            found += 1u;
        }
    }

    // (Should never happen, but fallback to first card)
    if (found == 0u) {
        // emergency: just take the first card we see
        for (var i = 0u; i < 48u; i++) {
            let half = i / 32u;
            let bit  = i % 32u;
            let mask = 1u << bit;
            let in_hand = select(
                (updated_hand[1] & mask) != 0u,
                (updated_hand[0] & mask) != 0u,
                half == 0u
            );
            if (in_hand) {
                chosen = i;
                if (half == 0u) {
                    updated_hand[0] &= ~mask;
                } else {
                    updated_hand[1] &= ~mask;
                }
                break;
            }
        }
    }

    return LegalCardResult(chosen, updated_hand);
}

// Evaluate a trick: returns (winner_player, trick_points)
fn evaluate_trick(
    cards: array<u32, 4>,
    lead_player: u32,
    trump: u32
) -> vec2<u32> {
    var best_card: u32 = cards[0];
    var winner: u32 = lead_player;
    var total_points: u32 = 0;

    for (var i = 1u; i < 4u; i++) {
        let card = cards[i];
        total_points += card_point(card);

        let suit = card / 12u;
        let rank = card % 12u;
        let best_rank = best_card % 12u;
        let best_suit = best_card / 12u;

        var beats = false;
        if (suit == trump && best_suit != trump) {
            beats = true;
        } else if (suit == best_suit && rank > best_rank) {
            beats = true;
        }

        if (beats) {
            best_card = card;
            winner = (lead_player + i) % 4u;
        }
    }

    return vec2<u32>(winner, total_points);
}
// # endregion

@group(0) @binding(0) var<storage, read> inputs: array<MctsRolloutInput>;
@group(0) @binding(1) var<storage, read_write> outputs: array<MctsRolloutOutput>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;
    let input = inputs[thread_id];

    var metadata = unpack_metadata(input.packed_metadata);
    var rng = input.rng_state.x;

    // Pre‑compute void suits and hand sizes
    var void_suits: array<u32, 4>;
    var hand_sizes: array<u32, 4>;
    for (var p = 0u; p < 4u; p++) {
        void_suits[p] = get_void_suits(input.packed_constraints, p);
        hand_sizes[p]  = get_hand_size(input.packed_constraints, p);
    }

    // Copy known hands
    var hands: array<vec2<u32>, 4>;
    hands[0] = input.p0_known;
    hands[1] = input.p1_known;
    hands[2] = input.p2_known;
    hands[3] = input.p3_known;

    var unseen = input.unseen_pool;

    // Deal unseen cards
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

        // Safety fallback
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

    //  Play remaining tricks
    var scores = input.scores;
    let total_tricks = 12u;
    let completed = metadata.tricks_played;
    var leader = metadata.lead_player;

    for (var trick = completed; trick < total_tricks; trick++) {
        var trick_cards: array<u32, 4>;
        var first_suit: u32 = 4u; // 4 = "any"

        for (var turn = 0u; turn < 4u; turn++) {
            let player = (leader + turn) % 4u;

            // Call by value, get updated hand back
            let result = select_legal_card(
                hands[player],
                first_suit,
                metadata.trump_suit,
                &rng
            );
            hands[player] = result.updated_hand;
            trick_cards[turn] = result.chosen;

            if (turn == 0u) {
                first_suit = result.chosen / 12u;
            }
        }

        let res = evaluate_trick(trick_cards, leader, metadata.trump_suit);
        let winner = res.x;
        let points = res.y;

        if (winner == 0u || winner == 2u) {
            scores.x += i32(points);
        } else {
            scores.y += i32(points);
        }

        leader = winner;
    }

    outputs[thread_id].team_0_2_score = u32(scores.x);
    outputs[thread_id].team_1_3_score = u32(scores.y);
}