/// Entrypoint for a pinochle simulation written using a webgpu kernel

use std::num::NonZeroU64;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct MctsRolloutInput {
    // Hands 32 Bytes 
    pub p0_hand: [u32; 2],
    pub p1_hand: [u32; 2],
    pub p2_hand: [u32; 2],
    pub p3_hand: [u32; 2],

    // Mask of the 0 to 3 cards currently sitting on the table.
    pub current_trick_cards: [u32; 2],

    // CPU RNG 8 bytes
    pub rng_state: [u32; 2],

    // Packed Metadata 12 Bytes
    pub packed_metadata: u32, // Who is playing, what is trump, etc.
    pub packed_scores: u32,   // Current meld + trick points
    pub packed_bids: u32,     // Who won the bid and for what amount

    pub _padding: u32,
}

impl MctsRolloutInput {
    pub fn pack_metadata(
        winning_card_index: u32,
        winning_player: u32,
        lead_player: u32,
        current_player: u32,
        trump_suit: u32,
        lead_suit: u32,
        tricks_played: u32,
    ) -> u32 {
        let mut packed = 0u32;
        packed |= winning_card_index & 0x3F;               // 6 bits
        packed |= (winning_player & 0x03) << 6;            // 2 bits
        packed |= (lead_player & 0x03) << 8;               // 2 bits
        packed |= (current_player & 0x03) << 10;           // 2 bits
        packed |= (trump_suit & 0x03) << 12;               // 2 bits
        packed |= (lead_suit & 0x07) << 14;                // 3 bits
        packed |= (tricks_played & 0x0F) << 17;            // 4 bits
        packed
    }

    pub fn pack_scores(team_a_score: u32, team_b_score: u32) -> u32 {
        (team_a_score & 0xFFFF) | ((team_b_score & 0xFFFF) << 16)
    }

    pub fn pack_bids(team_a_bid: u32, team_b_bid: u32) -> u32 {
        (team_a_bid & 0xFFFF) | ((team_b_bid & 0xFFFF) << 16)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MctsRolloutOutput {
    pub team_0_2_score: u32,
    pub team_1_3_score: u32,
}

fn main() {
    env_logger::init();

    // Example input: a handful of 256-bit hands to evaluate.
    // Replace this with your real hand-generation logic.
    let hands: Vec<MctsRolloutInput> = vec![

    ];

    // Load wgpu
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    // We then create an `Adapter` which represents a physical gpu in the system
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Failed to create adapter");

    // Check to see if the adapter supports compute shaders
    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("Adapter does not support compute shaders");
    }

    // We then create a `Device` and a `Queue` from the `Adapter`.
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
        .expect("Failed to create device");

    // Create a shader module from our shader code. This will parse and validate the shader.
    let module = device.create_shader_module(wgpu::include_wgsl!("../shaders/pinochle.wgsl"));

    // Input buffer: array<Bit256> — each element is 8 x u32 = 32 bytes.
    let input_hands_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("input_hands"),
        contents: bytemuck::cast_slice(&hands),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    // Output buffer: array<u32>, one result per hand.
    let output_size = (hands.len().max(1) * std::mem::size_of::<u32>()) as u64;
    let output_results_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output_results"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // CPU-readable staging buffer for the output.
    let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("download"),
        size: output_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // A bind group layout describes the types of resources that a bind group can contain
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            // input_hands: array<Bit256>, read-only storage. Min size = one Bit256 (32 bytes).
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    min_binding_size: Some(NonZeroU64::new(32).unwrap()),
                    has_dynamic_offset: false,
                },
                count: None,
            },
            // output_results: array<u32>, read-write storage. Min size = one u32 (4 bytes).
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    has_dynamic_offset: false,
                },
                count: None,
            },
        ],
    });

    // The bind group contains the actual resources to bind to the pipeline.
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_hands_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_results_buffer.as_entire_binding(),
            },
        ],
    });

    // The pipeline layout describes the bind groups that a pipeline expects
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    // The pipeline is the ready-to-go program state for the GPU
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // The command encoder allows us to record commands that we will later submit to the GPU.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // A compute pass is a single series of compute operations
    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: None,
        timestamp_writes: None,
    });

    compute_pass.set_pipeline(&pipeline);
    compute_pass.set_bind_group(0, &bind_group, &[]);

    // Shader's workgroup_size is 64x1x1, so we ceiling-divide the number of hands by 64.
    let workgroup_count = hands.len().div_ceil(64).max(1);
    compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

    drop(compute_pass);

    // Copy the output from GPU-only storage into the CPU-mappable download buffer.
    encoder.copy_buffer_to_buffer(
        &output_results_buffer,
        0,
        &download_buffer,
        0,
        output_results_buffer.size(),
    );

    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    // Map the download buffer so we can read it.
    let buffer_slice = download_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {
        // We wait synchronously below via device.poll, so nothing needed here.
    });

    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    // Read the data back as u32 (0 = false, 1 = true per hand).
    let data = buffer_slice.get_mapped_range().unwrap();
    let result: Vec<u32> = bytemuck::allocation::pod_collect_to_vec(&data);
    drop(data);
    download_buffer.unmap();

    for (i, (hand, passed)) in hands.iter().zip(result.iter()).enumerate() {
        println!("Hand {i}: {:?} -> {}", hand.data, *passed == 1);
    }
}