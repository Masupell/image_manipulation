use wgpu::util::DeviceExt;

// small: data with <= 8388480 elements
// big: max elemenst depends on type (number elements * element size) and max buffersize (256MB for my GPU)/ in this case the workgroup size, which is 128
// so here for f32 (4bytes per element) -> 33554432 (33M)
// Could make it bigger, by using multiple buffers, but not worth it for me right now

// Shader Path for .wgsl file
// Would also need to have input + output bindings with same variable type as T, like so:
// @group(0) @binding(0) var<storage, read> input_buffer: array<T>;
// @group(0) @binding(1) var<storage, read_write> output_buffer: array<T>;
// T being T (but not written as generic (f32, u32, i32,...))
// Also main always has to look like that:
// @compute @workgroup_size(128) -> workgroup_size same as here
// fn main(@builtin(global_invocation_id) id: vec3<u32>) // always u32, because that is just an index
// To properly index, an index variable has to be declared (atleast for 2D workgroups):
// let index = id.x; -> would be for 1D, but therefore not really necessary
// let index = id.x + id.y * 65535 * 128; -> for 2D
// If the element_size is a multiple of workgroup_size, the following is not needed, but otherwise add this, before doing anything else:
// if (index < arrayLength(&input_buffer)) -> makes sure index is not out of bounds *then write any calculations in the if block
// output like this: output_buffer[index] = output;

pub async fn compute_shader<T: bytemuck::Pod>(input: &[T], shader_path: &str, small: bool) -> Vec<T>
{
    let input_data = input;
    
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor 
    {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions 
    {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&Default::default(), None).await.unwrap();

    let workgroup_size = 128;
    let num_values = input_data.len() as u32;

    let buffer_size = (input_data.len() * std::mem::size_of::<T>()) as wgpu::BufferAddress;

    // Input Buffer (Read Only Storage)
    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor
    {   
        label: Some("Input Buffer"),
        contents: bytemuck::cast_slice(&input_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    // Output Buffer (Read and Write Storage)
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor 
    {
        label: Some("Output Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // To send the data back to the CPU
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor 
    {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor 
    {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()), //No error handling
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
    {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry //Input
        {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true }, //Input-data read only
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry //Output
        {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false }, //Output-data read and write
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor 
    {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry //Input
        {
            binding: 0,
            resource: input_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry //Output
        {
            binding: 1,
            resource: output_buffer.as_entire_binding(),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
    {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor 
    {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module, 
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
    {
        label: Some("Compute Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor 
        {
            label: Some("Compute Pass"),
            timestamp_writes: None  
        });
    
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
    
        if small
        {
            let workgroups = (num_values + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(workgroups, 1, 1); // 1D
        }
        else 
        {
            let workgroups_x = 65535;
            let workgroups_y = (num_values + workgroup_size - 1) / workgroups_x;
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1); // 2D
        }
    }

    encoder.copy_buffer_to_buffer(&output_buffer, 0, &readback_buffer, 0, buffer_size);

    queue.submit(Some(encoder.finish()));
    let buffer_slice = readback_buffer.slice(..);
    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| 
    {
        tx.send(result).unwrap();
    });

    device.poll(wgpu::Maintain::Wait);

    rx.receive().await.unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: &[T] = bytemuck::cast_slice(&data);
    result.to_vec()
}