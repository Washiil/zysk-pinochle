use crate::core::Suit;
use crate::pinochle::GamePhase;

/*
u64
000000000000_000000000000_000000000000_000000000001_00000000_00000000
Single Suit: A/1 A/2, 10/1, 10/2, K/1, K/2, Q/1, Q/2, J/1, J/2, 9/1, 9/2 -- 12 bits
4 * 12 = 48 bits
Left with 16 bits "free space"
 */

pub struct GameState {
    pub hands: [u64; 4], // Cards we know a given player has / our deal

    pub current_trick: [Option<u16>; 4],
    pub cards_in_current_trick: u8,

    pub trump: Option<Suit>,
    pub turn: usize,
    pub lead_player: usize,

    pub scores: [i32; 2], // Two teams
    pub current_bid: u16,
    pub winning_bidder: usize,

    pub void_mask: u16,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            hands: [0, 0, 0, 0],
            current_trick: [None, None, None, None],
            cards_in_current_trick: 0,
            trump: None,
            turn: 0,
            lead_player: 0,
            scores: [0, 0],
            current_bid: 0,
            winning_bidder: 0,
            void_mask: 0,
        }
    }
}