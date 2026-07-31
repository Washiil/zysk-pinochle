// Shader for pinochle Monte carlo simulation

// # region Input structs + helpers
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

struct MctsRolloutOutput {
    team_0_2_score: u32,
    team_1_3_score: u32,
}

// Bind group buffers
@group(0) @binding(0) var<storage, read> inputs: array<MctsRolloutInput>;
@group(0) @binding(1) var<storage, read_write> outputs: array<MctsRolloutOutput>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;
    
    // Load into local register
    let input = inputs[thread_id];
    
    var metadata = unpack_metadata(input.packed_metadata);
    var rng_seed = input.rng_state;

    // If it's P2's turn, check if they are void in the lead suit
    let p2_voids = get_void_suits(input.packed_constraints, 2u);
    
    // Shift a 1 to the lead_suit index. If the AND is > 0, they are void.
    let is_void_in_lead = (p2_voids & (1u << metadata.lead_suit)) != 0u;

    
}