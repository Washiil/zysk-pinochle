# Zysk Pinochle
Zysk, meaning profit in Polish, is meant to provide an extremely high performance 
implementation of the card game pinochle. Note that because the game of Pinochle 
has so many variations both official and personal I have chosen to arbitrarily 
implement the version I've played my whole life.

## Performance Considerations
### Bitboards
Bit boards are the most critical enhancement as we are dealing with a 48 card deck that fits nicely into a `u64`.

Additionally, this allows us to make use of bit masks when calculating meld, valid moves, and scoring.

A deck representation can be understood from the following:
```
// I am ommiting the 16 unused bits at the front of the deck
Deck:   | 000000000000_000000000000_000000000000_000000000000 |
Suits:  | DDDDDDDDDDDD_HHHHHHHHHHHH_CCCCCCCCCCCC_SSSSSSSSSSSS |
Ranks:  | AATTKKQQJJ99_AATTKKQQJJ99_AATTKKQQJJ99_AATTKKQQJJ99 |
```

The main downside to this approach is that logic can become ambiguous to someone
unfamiliar with the codebase. Importantly consider there are 12 cards per suit and
8 cards per rank. 

### Cache Line Consideration
In an effort to ensure we squeeze an unresonable amount of performance from this
program I decided to ensure the game object itself remains in a single 64 byte cache line.
This ensures we avoid invalidation and allow for single clock reads.

```rust
pub struct PinochleState {
    pub hands: [u64; 4],      // 8 * 4 = 32
    pub dead_cards: [u64; 2], // 8 * 2 = 16
    pub trick_cards: [u8; 4], // 1 * 4 = 4
    pub scores: [u16; 2],     // 2 * 2 = 4
    pub trump_suit: Suit,     // 1 * 1 = 1
    pub turn: u8,             // 1 * 1 = 1
    pub leader: u8,           // 1 * 1 = 1
    
    pub _padding: [u8; 5],    // 1 * 5 = 5
}
```

Luckily for me, Pinochle is not an incredibly complicated game. By restricting 
the memory footprint I ensure that I only consider the game to contain necessary logic
and I get the added benefit of blazingly fast performance.

### Agents
The intention is for agents was to have an extremely minimal required implementation.

```rust
pub trait Agent {
    fn bid(&mut self, state: &PinochleState) -> u8;
    fn play(&mut self, state: &PinochleState) -> u8;
    fn game_over(&mut self, _final_state: &PinochleState) {}
    fn name(&self) -> &str { "Unknown Agent" }
}
```

This leaves complexity completely in the hand of the implementor. 

```
//        [               [Turn index  ] [48 Cards seperated by Suit                         ]
let game = 00000000000000_[00]           [000000000000_000000000000_000000000000_000000000000;
```