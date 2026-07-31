// Shader for pinochle Monte carlo simulation

struct RolloutConstants {
    // Ground-truth hand for perspective_player
    p0_hand: array<u32, 2>,
    p1_hand: array<u32, 2>,
    p2_hand: array<u32, 2>,
    p3_hand: array<u32, 2>,

    current_trick_cards: array<u32, 2>,
    unseen_pool:         array<u32, 2>, // undealt cards, redealt each rollout
    exposed_cards:       array<u32, 2>, // publicly known cards (melds etc.)
    played_out_cards:    array<u32, 2>, // already played & collected this hand

    packed_metadata:    u32, // winning_card_idx | winning_player | lead_player |
                              // current_player | trump_suit | lead_suit | tricks_played
    packed_scores:      u32, // team_a_score | team_b_score << 16
    packed_bids:        u32, // team_a_bid   | team_b_bid   << 16
    packed_constraints: u32, // void_suits (16b, 4b/player) | hand_sizes (16b, 4b/player)

    packed_extra: u32, // perspective_player(2b) | exposed_owner(2b/player = 6b) | bidding_player(2b) | trump_broken(1b) | reserved
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct MctsRolloutOutput {
    team_0_2_score: u32,
    team_1_3_score: u32,
}

// Bind group buffers
@group(0) @binding(0) var<storage, read> inputs: array<MctsRolloutInput>;
@group(0) @binding(1) var<storage, read_write> outputs: array<MctsRolloutOutput>;

// Helper: Checks if a specific bit (0 to 255) is set inside a 256-bit block.
fn is_bit_set(hand: Bit256, bit_index: u32) -> bool {
    // Determine which of the 8 u32 elements contains our target bit
    let element_index = bit_index / 32u;
    // Determine the offset (0 to 31) inside that u32
    let bit_offset = bit_index % 32u;

    // Use bitwise AND with a left-shifted mask to test the bit
    return (hand.data[element_index] & (1u << bit_offset)) != 0u;
}

// Your custom business logic evaluating the 256-bit hand vector.
fn simulate_hand(hand: Bit256) -> bool {
    // --- Example Simulation Logic ---
    // Rule: Check if bit 0 AND bit 64 are both set.
    let rule_a = is_bit_set(hand, 0u) && is_bit_set(hand, 64u);

    // Rule: Check if the first 32-bit chunk (data[0]) matches a bitmask pattern.
    let rule_b = (hand.data[0] & 0xFF00FF00u) != 0u;

    // Return true if either rule condition passes
    return rule_a || rule_b;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Get the linear thread index
    let index = global_id.x;

    // Out-of-bounds safety check against the number of input 256-bit hands
    let total_hands = arrayLength(&input_hands);
    if (index >= total_hands) {
        return;
    }

    // Read the 256-bit hand vector for this thread
    let hand = input_hands[index];

    // Run the simulation logic
    let result: bool = simulate_hand(hand);

    // Store the boolean output as 1u (true) or 0u (false)
    output_results[index] = select(0u, 1u, result);
}