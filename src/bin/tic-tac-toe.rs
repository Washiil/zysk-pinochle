use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

/// Cell encoding, 2 bits per cell, matching the WGSL shader:
///   00 = empty, 01 = X, 10 = O
const EMPTY: u32 = 0b00;
const X: u32 = 0b01;
const O: u32 = 0b10;

/// Pack a 9-cell board (row-major, cell 0..8) into a single u32 bitboard,
/// 2 bits per cell, matching `get_cell` in the WGSL shader:
///   (board >> (idx * 2)) & 0x3
fn pack_board(cells: [u32; 9]) -> u32 {
    let mut board = 0u32;
    for (idx, &cell) in cells.iter().enumerate() {
        debug_assert!(cell <= 0b11, "cell value must fit in 2 bits");
        board |= cell << (idx as u32 * 2);
    }
    board
}

fn main() {
    env_logger::init();

    #[rustfmt::skip]
    let raw_boards: [[u32; 9]; 3] = [
        // Board 0: X has middle + a corner, O has the opposite corner.
        [
            X,     EMPTY, O,
            EMPTY, X,     EMPTY,
            EMPTY, EMPTY, EMPTY,
        ],
        // Board 1: empty board. X to move.
        [
            EMPTY, EMPTY, EMPTY,
            EMPTY, EMPTY, EMPTY,
            EMPTY, EMPTY, EMPTY,
        ],
        // Board 2: X is one move from winning across the top row.
        [
            X, X,     EMPTY,
            O, O,     EMPTY,
            EMPTY, EMPTY, EMPTY,
        ],
    ];

    // Pack each 9-cell board down to a single u32 bitboard
    let boards: Vec<u32> = raw_boards.iter().map(|&cells| pack_board(cells)).collect();
    let board_count = boards.len();
    println!("Solving {board_count} board(s)");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Failed to create adapter");

    println!("Running on Adapter: {:#?}", adapter.get_info());

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("Adapter does not support compute shaders");
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))
    .expect("Failed to create device");

    let module = device.create_shader_module(wgpu::include_wgsl!("../shaders/tic-tac-toe.wgsl"));

    // Input buffer: ONE u32 per board now (packed bitboard), not 9 floats.
    let input_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&boards),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // Define what the outputs should look like
    let output_element_count = board_count;
    let output_size = (output_element_count * std::mem::size_of::<u32>()) as u64;
    let output_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Download buffer must match the OUTPUT size, not the input size.
    let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: output_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                    has_dynamic_offset: false,
                },
                count: None,
            },
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

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_data_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("solve_tic_tac_toe"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: None,
        timestamp_writes: None,
    });

    compute_pass.set_pipeline(&pipeline);
    compute_pass.set_bind_group(0, &bind_group, &[]);

    // One invocation solves one whole board (hopefully)
    let workgroup_count = board_count.div_ceil(64);
    compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

    drop(compute_pass);

    encoder.copy_buffer_to_buffer(
        &output_data_buffer,
        0,
        &download_buffer,
        0,
        output_data_buffer.size(),
    );

    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    let buffer_slice = download_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

    let data = buffer_slice.get_mapped_range().unwrap();
    let result: Vec<u32> = bytemuck::allocation::pod_collect_to_vec(&data);

    println!("{result:?}");

    // One result per board
    for (i, res) in result.iter().enumerate() {
        println!("Board {i}: {res}");
    }
}
