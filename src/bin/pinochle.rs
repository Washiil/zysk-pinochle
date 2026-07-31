/// Entrypoint for a pinochle simulation written using a webgpu kernel

use std::num::NonZeroU64;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MctsRolloutInput {
    // Perspective player: full hand known.
    // Others: kown (melded) cards only.
    pub p0_known: [u32; 2],
    pub p1_known: [u32; 2],
    pub p2_known: [u32; 2],
    pub p3_known: [u32; 2],

    pub current_trick_cards: [u32; 2],
    pub unseen_pool: [u32; 2], // full_deck - all known_cards - played_out_cards
    pub played_out_cards: [u32; 2], // already trick-collected before this rollout node

    pub scores: [i32; 2],
    
    pub packed_metadata: u32, // winning_card_idx|winning_player|lead_player|current_player|trump_suit|lead_suit|tricks_played
    pub packed_bids: u32, // team_a_bid | team_b_bid<<16
    pub packed_constraints: u32, // void_suits(16, 4b/player) | hand_sizes(16, 4b/player)
    pub packed_meld:     u32, // team_a_meld | team_b_meld << 16
    
    pub rng_state: [u32; 2],
}

impl MctsRolloutInput {

    /// Converts a standard Rust 64-bit bitboard to a GPU-friendly [u32; 2] pair
    /// where index 0 is lower 32 bits, and index 1 is upper 32 bits.
    #[inline(always)]
    pub fn pack_bitboard(bb: u64) -> [u32; 2] {
        [bb as u32, (bb >> 32) as u32]
    }

    /// Reconstructs a 64-bit bitboard from a [u32; 2] pair (useful for CPU debugging)
    #[inline(always)]
    pub fn unpack_bitboard(pair: [u32; 2]) -> u64 {
        (pair[0] as u64) | ((pair[1] as u64) << 32)
    }

    /// Packs trick metadata into a single u32 (21 bits used total)
    /// - `winning_card_idx`: 0..47 (use 63 for empty trick) [6 bits]
    /// - `winning_player`: 0..3                             [2 bits]
    /// - `lead_player`: 0..3                                [2 bits]
    /// - `current_player`: 0..3                             [2 bits]
    /// - `trump_suit`: 0..3 (Spades, Hearts, Diamonds, Clubs) [2 bits]
    /// - `lead_suit`: 0..3 (use 7 for empty trick)          [3 bits]
    /// - `tricks_played`: 0..12                             [4 bits]
    #[inline]
    pub fn pack_metadata(
        winning_card_idx: u8,
        winning_player: u8,
        lead_player: u8,
        current_player: u8,
        trump_suit: u8,
        lead_suit: u8,
        tricks_played: u8,
    ) -> u32 {
        let mut m = 0u32;
        m |= (winning_card_idx as u32 & 0x3F) << 0;
        m |= (winning_player as u32 & 0x03) << 6;
        m |= (lead_player as u32 & 0x03) << 8;
        m |= (current_player as u32 & 0x03) << 10;
        m |= (trump_suit as u32 & 0x03) << 12;
        m |= (lead_suit as u32 & 0x07) << 14;
        m |= (tricks_played as u32 & 0x0F) << 17;
        m
    }

    /// Packs Team A and Team B bids (16 bits each)
    #[inline]
    pub fn pack_bids(team_a_bid: u16, team_b_bid: u16) -> u32 {
        (team_a_bid as u32) | ((team_b_bid as u32) << 16)
    }

    /// Packs Team A and Team B meld points (16 bits each)
    #[inline]
    pub fn pack_meld(team_a_meld: u16, team_b_meld: u16) -> u32 {
        (team_a_meld as u32) | ((team_b_meld as u32) << 16)
    }

    /// Packs constraints for determinization:
    /// - `void_suits`: 4 bits per player (P0..P3), where each bit represents a void suit mask (0bS_H_D_C)
    /// - `hand_sizes`: 4 bits per player (P0..P3), representing how many cards they hold (0..12)
    #[inline]
    pub fn pack_constraints(void_suits: [u8; 4], hand_sizes: [u8; 4]) -> u32 {
        let mut c = 0u32;

        // Bits 0..15: Void suits (4 bits * 4 players)
        for i in 0..4 {
            c |= ((void_suits[i] as u32) & 0x0F) << (i * 4);
        }

        // Bits 16..31: Hand sizes (4 bits * 4 players)
        for i in 0..4 {
            c |= ((hand_sizes[i] as u32) & 0x0F) << (16 + i * 4);
        }

        c
    }

    /// Creates a complete, GPU-ready input struct from standard CPU parameters.
    pub fn new(
        known_hands: [u64; 4],
        current_trick: u64,
        unseen_pool: u64,
        played_out_cards: u64,
        scores: [i32; 2],
        metadata: MetadataParams,
        bids: (u16, u16),
        melds: (u16, u16),
        void_suits: [u8; 4],
        hand_sizes: [u8; 4],
        rng_seed: u64,
    ) -> Self {
        Self {
            p0_known: Self::pack_bitboard(known_hands[0]),
            p1_known: Self::pack_bitboard(known_hands[1]),
            p2_known: Self::pack_bitboard(known_hands[2]),
            p3_known: Self::pack_bitboard(known_hands[3]),

            current_trick_cards: Self::pack_bitboard(current_trick),
            unseen_pool: Self::pack_bitboard(unseen_pool),
            played_out_cards: Self::pack_bitboard(played_out_cards),

            scores,

            packed_metadata: Self::pack_metadata(
                metadata.winning_card_idx,
                metadata.winning_player,
                metadata.lead_player,
                metadata.current_player,
                metadata.trump_suit,
                metadata.lead_suit,
                metadata.tricks_played,
            ),
            packed_bids: Self::pack_bids(bids.0, bids.1),
            packed_meld: Self::pack_meld(melds.0, melds.1),
            packed_constraints: Self::pack_constraints(void_suits, hand_sizes),

            rng_state: Self::pack_bitboard(rng_seed),
        }
    }
}

/// Helper container for human-readable metadata parameters
#[derive(Clone, Copy, Debug)]
pub struct MetadataParams {
    pub winning_card_idx: u8,
    pub winning_player: u8,
    pub lead_player: u8,
    pub current_player: u8,
    pub trump_suit: u8,
    pub lead_suit: u8,
    pub tricks_played: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MctsRolloutOutput {
    pub team_0_2_score: u32,
    pub team_1_3_score: u32,
}

fn main() {
    env_logger::init();

    let input = MctsRolloutInput::new(
        [0x000000000000FFFF, 0x0, 0x0, 0x0], // known hands (u64 bitboards)
        0x0,                                // current trick empty
        0x0000FFFFFFFF0000,                 // unseen pool
        0xFFFF000000000000,                 // played out cards
        [120, 90],                          // team scores
        MetadataParams {
            winning_card_idx: 63,           // no winner yet (63 = empty flag)
            winning_player: 0,
            lead_player: 0,
            current_player: 0,
            trump_suit: 1,                  // Hearts
            lead_suit: 7,                   // None (7 = empty flag)
            tricks_played: 6,
        },
        (250, 0),                           // bids: Team A 250, Team B 0
        (60, 20),                           // meld points
        [0b0000, 0b0001, 0b0100, 0b0000],   // P1 void in Spades, P2 void in Diamonds
        [6, 6, 6, 6],                       // 6 cards remaining per hand
        1234567890123456789,                // RNG seed
    );

    // Safe zero-copy cast to bytes for WGPU buffer submission:
    let bytes: &[u8] = bytemuck::bytes_of(&input);
    assert_eq!(bytes.len(), 88); // 88 bytes continuous payload

    // // Example input: a handful of 256-bit hands to evaluate.
    // // Replace this with your real hand-generation logic.
    // let hands: Vec<MctsRolloutInput> = vec![

    // ];

    // // Load wgpu
    // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    // // We then create an `Adapter` which represents a physical gpu in the system
    // let adapter =
    //     pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    //         .expect("Failed to create adapter");

    // // Check to see if the adapter supports compute shaders
    // let downlevel_capabilities = adapter.get_downlevel_capabilities();
    // if !downlevel_capabilities
    //     .flags
    //     .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    // {
    //     panic!("Adapter does not support compute shaders");
    // }

    // // We then create a `Device` and a `Queue` from the `Adapter`.
    // let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
    //     label: None,
    //     required_features: wgpu::Features::empty(),
    //     required_limits: wgpu::Limits::downlevel_defaults(),
    //     experimental_features: wgpu::ExperimentalFeatures::disabled(),
    //     memory_hints: wgpu::MemoryHints::MemoryUsage,
    //     trace: wgpu::Trace::Off,
    // }))
    //     .expect("Failed to create device");

    // // Create a shader module from our shader code. This will parse and validate the shader.
    // let module = device.create_shader_module(wgpu::include_wgsl!("../shaders/pinochle.wgsl"));

    // // Input buffer: array<Bit256> — each element is 8 x u32 = 32 bytes.
    // let input_hands_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //     label: Some("input_hands"),
    //     contents: bytemuck::cast_slice(&hands),
    //     usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    // });

    // // Output buffer: array<u32>, one result per hand.
    // let output_size = (hands.len().max(1) * std::mem::size_of::<u32>()) as u64;
    // let output_results_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: Some("output_results"),
    //     size: output_size,
    //     usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    //     mapped_at_creation: false,
    // });

    // // CPU-readable staging buffer for the output.
    // let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: Some("download"),
    //     size: output_size,
    //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    //     mapped_at_creation: false,
    // });

    // // A bind group layout describes the types of resources that a bind group can contain
    // let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //     label: None,
    //     entries: &[
    //         // input_hands: array<Bit256>, read-only storage. Min size = one Bit256 (32 bytes).
    //         wgpu::BindGroupLayoutEntry {
    //             binding: 0,
    //             visibility: wgpu::ShaderStages::COMPUTE,
    //             ty: wgpu::BindingType::Buffer {
    //                 ty: wgpu::BufferBindingType::Storage { read_only: true },
    //                 min_binding_size: Some(NonZeroU64::new(32).unwrap()),
    //                 has_dynamic_offset: false,
    //             },
    //             count: None,
    //         },
    //         // output_results: array<u32>, read-write storage. Min size = one u32 (4 bytes).
    //         wgpu::BindGroupLayoutEntry {
    //             binding: 1,
    //             visibility: wgpu::ShaderStages::COMPUTE,
    //             ty: wgpu::BindingType::Buffer {
    //                 ty: wgpu::BufferBindingType::Storage { read_only: false },
    //                 min_binding_size: Some(NonZeroU64::new(4).unwrap()),
    //                 has_dynamic_offset: false,
    //             },
    //             count: None,
    //         },
    //     ],
    // });

    // // The bind group contains the actual resources to bind to the pipeline.
    // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //     label: None,
    //     layout: &bind_group_layout,
    //     entries: &[
    //         wgpu::BindGroupEntry {
    //             binding: 0,
    //             resource: input_hands_buffer.as_entire_binding(),
    //         },
    //         wgpu::BindGroupEntry {
    //             binding: 1,
    //             resource: output_results_buffer.as_entire_binding(),
    //         },
    //     ],
    // });

    // // The pipeline layout describes the bind groups that a pipeline expects
    // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //     label: None,
    //     bind_group_layouts: &[Some(&bind_group_layout)],
    //     immediate_size: 0,
    // });

    // // The pipeline is the ready-to-go program state for the GPU
    // let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    //     label: None,
    //     layout: Some(&pipeline_layout),
    //     module: &module,
    //     entry_point: Some("main"),
    //     compilation_options: wgpu::PipelineCompilationOptions::default(),
    //     cache: None,
    // });

    // // The command encoder allows us to record commands that we will later submit to the GPU.
    // let mut encoder =
    //     device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // // A compute pass is a single series of compute operations
    // let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
    //     label: None,
    //     timestamp_writes: None,
    // });

    // compute_pass.set_pipeline(&pipeline);
    // compute_pass.set_bind_group(0, &bind_group, &[]);

    // // Shader's workgroup_size is 64x1x1, so we ceiling-divide the number of hands by 64.
    // let workgroup_count = hands.len().div_ceil(64).max(1);
    // compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

    // drop(compute_pass);

    // // Copy the output from GPU-only storage into the CPU-mappable download buffer.
    // encoder.copy_buffer_to_buffer(
    //     &output_results_buffer,
    //     0,
    //     &download_buffer,
    //     0,
    //     output_results_buffer.size(),
    // );

    // let command_buffer = encoder.finish();
    // queue.submit([command_buffer]);

    // // Map the download buffer so we can read it.
    // let buffer_slice = download_buffer.slice(..);
    // buffer_slice.map_async(wgpu::MapMode::Read, |_| {
    //     // We wait synchronously below via device.poll, so nothing needed here.
    // });

    // device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // // Read the data back as u32 (0 = false, 1 = true per hand).
    // let data = buffer_slice.get_mapped_range().unwrap();
    // let result: Vec<u32> = bytemuck::allocation::pod_collect_to_vec(&data);
    // drop(data);
    // download_buffer.unmap();

    // for (i, (hand, passed)) in hands.iter().zip(result.iter()).enumerate() {
    //     println!("Hand {i}: {:?} -> {}", hand.data, *passed == 1);
    // }
}