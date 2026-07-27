/// Entrypoint for a pinochle simulation written using a webgpu kernel

use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Bit256 {
    data: [u32; 8],
}

fn main() {
    env_logger::init();

    // Example input: a handful of 256-bit hands to evaluate.
    // Replace this with your real hand-generation logic.
    let hands: Vec<Bit256> = vec![
        Bit256 { data: [1, 0, 1, 0, 0, 0, 0, 0] }, // bit 0 and bit 64 set -> rule_a true
        Bit256 { data: [0xFF00FF00, 0, 0, 0, 0, 0, 0, 0] }, // rule_b true
        Bit256 { data: [0, 0, 0, 0, 0, 0, 0, 0] }, // neither rule -> false
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