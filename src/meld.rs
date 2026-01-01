use crate::types::*;

// 4096 entries for every possible 12-bit combination
pub static TRUMP_MELD_TABLE: [u16; 4096] = {
    let mut table = [0; 4096];
    let mut i = 0;
    while i < 4096 {
        table[i] = compute_suit_meld(i as u16, true);
        i += 1;
    }
    table
};

pub static PLAIN_MELD_TABLE: [u16; 4096] = {
    let mut table = [0; 4096];
    let mut i = 0;
    while i < 4096 {
        table[i] = compute_suit_meld(i as u16, false);
        i += 1;
    }
    table
};

const fn compute_suit_meld(bits: u16, is_trump: bool) -> u16 {
    let ace_count   = (bits & Rank::Ace.relative_mask()).count_ones() as u16;
    let ten_count   = (bits & Rank::Ten.relative_mask()).count_ones() as u16;
    let king_count  = (bits & Rank::King.relative_mask()).count_ones() as u16;
    let queen_count = (bits & Rank::Queen.relative_mask()).count_ones() as u16;
    let jack_count  = (bits & Rank::Jack.relative_mask()).count_ones() as u16;
    let nine_count  = (bits & Rank::Nine.relative_mask()).count_ones() as u16;

    let mut score = 0;

    if is_trump {
        // Double Run
        if ace_count == 2 && ten_count == 2 && king_count == 2 && queen_count == 2 && jack_count == 2 {
            score += 150;
        }
        // Single Run
        else if ace_count >= 1 && ten_count >= 1 && king_count >= 1 && queen_count >= 1 && jack_count >= 1 {
            score += 15;

            // Check for "Extra Marriage" in a Run
            if king_count == 2 && queen_count == 2 {
                score += 4;
            }
        }
        else {
            // No Run, just count Royal Marriages
            let pairs = if king_count < queen_count { king_count } else { queen_count };
            score += pairs * 4;
        }

        // 9 of trumps
        score += nine_count;
    }
    else {
        // Only Common Marriages
        let pairs = if king_count < queen_count { king_count } else { queen_count };
        score += pairs * 2;
    }

    score
}

#[cfg(test)]
mod meld_tests {
    use super::*;

    #[test]
    fn test_run_meld() {
        // Single straight
        let deck = 0b101010101010;

        assert_eq!(TRUMP_MELD_TABLE[deck], 16);
        assert_eq!(PLAIN_MELD_TABLE[deck], 2);

        let deck = 0b111111111111;
        assert_eq!(TRUMP_MELD_TABLE[deck], 152);
        assert_eq!(PLAIN_MELD_TABLE[deck], 4);
    }

    #[test]
    fn test_marriages() {
        let deck = 0b000011110000;
        assert_eq!(TRUMP_MELD_TABLE[deck], 8);
        assert_eq!(PLAIN_MELD_TABLE[deck], 4);

    }
}