// Represents a 256-bit vector using 8 x 32-bit unsigned integers (8 * 32 = 256 bits).
struct Bit256 {
    // data[0] holds bits 0..31, data[1] holds bits 32..63, ..., data[7] holds bits 224..255
    data: array<u32, 8>,
}

// Input buffer: An array of N 256-bit vectors.
@group(0) @binding(0)
var<storage, read> input_hands: array<Bit256>;

// Output buffer: An array of N boolean results (stored as 0u for false, 1u for true).
@group(0) @binding(1)
var<storage, read_write> output_results: array<u32>;

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