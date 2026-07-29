/// Basic high performance tic tac toe compute shader

@group(0) @binding(0)
var<storage, read> input: array<u32>;
@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

const BOARD_SIZE: u32 = 9u;
const MAX_DEPTH: u32 = 10u;

const X: u32 = 1;
const O: u32 = 2;

/// 1 represents X and 2 represents O
fn get_cell(board: u32, idx: u32) -> u32 {
    // Because 00_00_11 
    return (board >> (idx * 2u)) & 0x3u;
}

const WINNING_LINES: array<u32, 8> = array<u32, 8>(
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
    won = won | ((board & (WINNING_LINES[0] * player)) == (WINNING_LINES[0] * player));
    won = won | ((board & (WINNING_LINES[1] * player)) == (WINNING_LINES[1] * player));
    won = won | ((board & (WINNING_LINES[2] * player)) == (WINNING_LINES[2] * player));
    won = won | ((board & (WINNING_LINES[3] * player)) == (WINNING_LINES[3] * player));
    won = won | ((board & (WINNING_LINES[4] * player)) == (WINNING_LINES[4] * player));
    won = won | ((board & (WINNING_LINES[5] * player)) == (WINNING_LINES[5] * player));
    won = won | ((board & (WINNING_LINES[6] * player)) == (WINNING_LINES[6] * player));
    won = won | ((board & (WINNING_LINES[7] * player)) == (WINNING_LINES[7] * player));

    return won;
}

fn count_x(board: u32) -> u32 {
    return countOneBits(board & 0x15555u); // 0b01_01_01_01_01_01_01_01_01
}

fn count_o(board: u32) -> u32 {
    return countOneBits(board & (0x15555 << 1u)); // 0b10_10_10_10_10_10_10_10_10
}

fn two_in_a_row() -> u32 {
    return 0u;
}

// Newell and Simon's 1972 tic-tac-toe program
fn get_best_move(board: u32, turn: u32) -> u32 {
    // Blocking an opponent's fork: If there is only one possible fork for the opponent, the player should block it. Otherwise, the player should block all forks in any way that simultaneously allows them to make two in a row. Otherwise, the player should make a two in a row to force the opponent into defending, as long as it does not result in them producing a fork. For example, if "X" has two opposite corners and "O" has the center, "O" must not play a corner move to win. (Playing a corner move in this scenario produces a fork for "X" to win.)
    // Center: A player marks the center. (If it is the first move of the game, playing a corner move gives the second player more opportunities to make a mistake and may therefore be the better choice; however, it makes no difference between perfect players.)
    // Opposite corner: If the opponent is in the corner, the player plays the opposite corner.
    // Empty corner: The player plays in a corner square.
    // Empty side: The player plays in a middle square on any of the four sides.
    
    let opponent = 3 - turn;
    
    // Win: If the player has two in a row, they can place a third to get three in a row.
    for (var i = 0u; i < 9u; i = i + 1u) {
        if (get_cell(board, i) == 0u) {
            if (check_win(board | (turn << (i * 2u)), turn)) { return i; }
        }
    }
    
    // Block: If the opponent has two in a row, the player must play the third themselves to block the opponent.
    for (var i = 0u; i < 9u; i = i + 1u) {
        if (get_cell(board, i) == 0u) {
            if (check_win(board | (opponent << (i * 2u)), opponent)) { return i; }
        }
    }
    
    // Fork: Cause a scenario where the player has two ways to win (two non-blocked WINNING_LINES of 2).


    return 0u;
}

@compute @workgroup_size(64)
fn solve_tic_tac_toe(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let board_index = global_id.x;
    let board_count = arrayLength(&input);

    // Bounds check FIRST before indexing arrays
    if (board_index >= board_count) {
        return;
    }

    let board = input[board_index];

    let x_count = count_x(board);
    let o_count = count_o(board);

    // Early termination
    if (check_win(board, X) || check_win(board, O) || x_count + o_count == 9u) {
        output[board_index] = 9u; 
        return;
    }

    let turn = (x_count % 2) + 1;
    let best_move = get_best_move(board, turn);

    output[board_index] = u32(best_move);
}
