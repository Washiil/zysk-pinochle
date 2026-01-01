use crate::types::Suit;

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinochleState {
    // hands[0] = Player 0 (Dealer left)
    pub hands: [u64; 4], // 8 * 4 = 32

    // Cards the table has "seen"
    pub dead_cards: [u64; 2], // 8 * 2 = 16

    // Stores card INDICES (0-47).
    // trick_cards[0] is always Player 0's card, regardless of who led.
    pub trick_cards: [u8; 4], // 1 * 4 = 4

    // Store the running game score computed
    pub scores: [u16; 2], // 2 * 2 = 4

    pub trump_suit: Suit, // 0..3 1 * 1 = 1
    pub turn: u8,         // 0..3 (Current player ID) 1 * 1 = 1
    pub leader: u8,       // 0..3 (Who led this trick) 1 * 1 = 1

    // Padding to fill 64 byte cache line
    pub _padding: [u8; 5],
}

impl PinochleState {
    pub fn new() -> Self {
        Self {
            hands: [0; 4],
            dead_cards: [0; 2],
            trick_cards: [0; 4],
            scores: [0; 2],
            trump_suit: Suit::Clubs,
            turn: 0,
            leader: 0,
            _padding: [0; 5],
        }
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_memory_layout_size_and_alignment() {
        // 1. Verify exact size is 64 bytes (Cache Line)
        assert_eq!(
            mem::size_of::<PinochleState>(),
            64,
            "PinochleState must be exactly 64 bytes to fit in a cache line"
        );

        // 2. Verify alignment is 64 bytes
        assert_eq!(
            mem::align_of::<PinochleState>(),
            64,
            "PinochleState must be aligned to 64 bytes"
        );
    }

    #[test]
    fn test_derive_copy() {
        // Ensure the struct is actually Copy (stack efficient)
        let s1 = PinochleState::new();
        let s2 = s1; // Copy happens here

        // If s1 is still usable, Copy is implemented correctly
        assert_eq!(s1.turn, s2.turn);
    }
}