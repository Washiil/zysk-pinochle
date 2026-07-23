# Zysk Pinochle

High performance pinochle card game engine. Data-oriented design with bitset based
game state for cache locality and SIMD-friendly state transitions.

## Motivation

Most card game engines use object-oriented designs where cards, hands, and players
own mutable state. This scatters game data across the heap and defeats the cache.
Zysk Pinochle takes the opposite approach: the entire game state lives in a single
64-byte struct aligned to a cache line. Every operation is a pure function that
takes a `GameState` by value and returns a new one. This means millions of game
states can sit contiguously in a `Vec`, enabling MCTS and other tree-search
algorithms to run with excellent spatial locality.

## Features

- **48-card deck in a u64 bitboard**. Each card is one bit. Hands, legal moves,
  and trick cards are all bitwise operations. No heap allocations in hot paths.

- **64-byte GameState**. One cache line. Copying a full game state is a single
  SIMD move. Fields are laid out with `#[repr(C, align(64))]`.

- **Pure function state machine**. `apply_action(state, action) -> GameState`.
  No mutation through references. Every state transition produces a new state,
  making the engine trivially parallelizable.

- **Lookup-table meld evaluation**. Per-suit 12-bit patterns mapped to meld
  scores via precomputed 4096-entry tables. Zero branching during scoring.

- **Swappable players**. The `Player` trait takes `&self` (not `&mut self`).
  `RandomPlayer` is a zero-sized type. Future agents can store shared
  parameters behind `&self` for lock-free parallel simulation.

- **Rayon parallelism**. Benchmark harness runs games in parallel across all
  cores.

## Performance

Benchmarks Coming Soon

## Bit Layout

The u64 bitboard is partitioned into four 12-bit suit blocks:

```
Bit:  47 ... 36 | 35 ... 24 | 23 ... 12 | 11 ... 0
      Diamonds  |  Hearts   |  Clubs    | Spades
```

Within each 12-bit suit block, 2 bits per rank:

```
Bit:  11 10 | 9  8 | 7  6 | 5  4 | 3  2 | 1  0
      A  A  | 10 10| K  K | Q  Q | J  J | 9  9
```

Card index (0-47): `index = suit * 12 + rank * 2 + copy`

This encoding makes suit extraction (`index / 12`), rank extraction
(`(index % 12) / 2`), and mask generation (`1u64 << index`) all compile
to single instructions via strength reduction.

## Player Trait

```rust
pub trait Player {
    fn bid(&self, state: &GameState) -> u16;
    fn play(&self, state: &GameState) -> u8;
}
```

No `&mut self`. `RandomPlayer` is a zero-sized type. For tree-search agents,
the search tree lives in a flat `Vec<GameState>` external to the player,
and the player only holds shared references to learned parameters.

## Usage

```rust
use zysk_pinochle::{new_hand, apply_action, is_hand_over, Action, Phase, Player, RandomPlayer};

let players: [&dyn Player; 4] = [&RandomPlayer; 4];
let mut state = new_hand();

// Bidding
while state.phase == Phase::Bidding {
    let bid = players[state.turn as usize].bid(&state);
    state = apply_action(state, Action::Bid(bid));
}

// Trick taking
while !is_hand_over(&state) {
    let card = players[state.turn as usize].play(&state);
    state = apply_action(state, Action::Play(card));
}

println!("Scores: {:?}", state.scores);
```
