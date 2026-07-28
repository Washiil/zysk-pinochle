/// Basic high performance tic tac toe compute shader

@group(0) @binding(0)
var<storage, read> input: array<u32>;
@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

const BOARD_SIZE: u32 = 9u;
const MAX_DEPTH: u32 = 10u;

/// 1 represents X and 2 represents O
fn get_cell(board: u32, idx: u32) -> i32 {
    // Because 00_00_11 
    return i32((board >> (idx * 2u)) & 0x3u);
}

const LINES: array<u32, 8> = array<u32, 8>(
    // Rows
    (1u << (0u * 2u)) | (1u << (1u * 2u)) | (1u << (2u * 2u)), // Top Row    (0, 1, 2)
    (1u << (3u * 2u)) | (1u << (4u * 2u)) | (1u << (5u * 2u)), // Mid Row    (3, 4, 5)
    (1u << (6u * 2u)) | (1u << (7u * 2u)) | (1u << (8u * 2u)), // Bot Row    (6, 7, 8)

    // Columns
    (1u << (0u * 2u)) | (1u << (3u * 2u)) | (1u << (6u * 2u)), // Left Col   (0, 3, 6)
    (1u << (1u * 2u)) | (1u << (4u * 2u)) | (1u << (7u * 2u)), // Mid Col    (1, 4, 7)
    (1u << (2u * 2u)) | (1u << (5u * 2u)) | (1u << (8u * 2u)), // Right Col  (2, 5, 8)

    // Diagonals
    (1u << (0u * 2u)) | (1u << (4u * 2u)) | (1u << (8u * 2u)), // Main Diag  (0, 4, 8)
    (1u << (2u * 2u)) | (1u << (4u * 2u)) | (1u << (6u * 2u))  // Anti Diag  (2, 4, 6)
);

fn check_win(board: u32, player: u32) -> bool {
    var won = false;
    
    // The GPU compiler will unroll this entirely into 8 parallel mask checks
    // using bitwise logic without branching instructions
    won = won | ((board & (LINES[0] * player)) == (LINES[0] * player));
    won = won | ((board & (LINES[1] * player)) == (LINES[1] * player));
    won = won | ((board & (LINES[2] * player)) == (LINES[2] * player));
    won = won | ((board & (LINES[3] * player)) == (LINES[3] * player));
    won = won | ((board & (LINES[4] * player)) == (LINES[4] * player));
    won = won | ((board & (LINES[5] * player)) == (LINES[5] * player));
    won = won | ((board & (LINES[6] * player)) == (LINES[6] * player));
    won = won | ((board & (LINES[7] * player)) == (LINES[7] * player));

    return won;
}

@compute @workgroup_size(64)
fn solve_tic_tac_toe(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let board_count = arrayLength(&input) / BOARD_SIZE;
    let board_index = global_id.x;
    if (board_index >= board_count) {
        return;
    }

    let made_move = input[board_index] & 0x3FFFFu;

    let low_bits  = input[board_index] & 0x15555u; // 0b01_01_01_01_01_01_01_01_01
    let high_bits = (input[board_index] & 0x2AAAAu) >> 1u; // 0b10_10_10_10_10_10_10_10_10

    // Write [best_move, score] for this board.
    output[board_index * 2u] = 0;
    output[board_index * 2u + 1u] = 0;
}
