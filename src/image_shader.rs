use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

// let input_img = image::open(path).unwrap();
// Needs to crop image better (changed to be more fitting, but still to big (padded image))
// shader_path: Path to .wgsl file (with vs_main and vs_frag)
pub async fn image_shader(input_img: DynamicImage, shader_path: &str) -> image::ImageBuffer<Rgba<u8>, Vec<u8>>
{
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor 
    {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions 
    {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }).await.unwrap();

    let required_features = wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::POLYGON_MODE_POINT;

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor 
    {
        label: None,
        required_features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
    },
    None).await.unwrap();

    // Input Texture
    /////////////////////////////////////////////////////////////////////////////
    let (img_width, img_height) = input_img.dimensions();

    let img_size = wgpu::Extent3d
    {
        width: img_width,
        height: img_height,
        depth_or_array_layers: 1,
    };

    let input_texture_desc = wgpu::TextureDescriptor
    {
        label: None,
        size: img_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[]
    };

    let input_texture = device.create_texture(&input_texture_desc);
    let input_texture_view = input_texture.create_view(&Default::default());

    queue.write_texture(wgpu::TexelCopyTextureInfo
    {
        texture: &input_texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    },
    input_img.to_rgba8().as_raw(),
    wgpu::TexelCopyBufferLayout
    {
        offset: 0,
        bytes_per_row: Some(4 * img_width),
        rows_per_image: Some(img_height),
    },
    img_size);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor
    {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries: 
        &[
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture 
                {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ]
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
    {
        label: None,
        layout: &texture_bind_group_layout,
        entries:
        &[
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&input_texture_view)
            },
            wgpu::BindGroupEntry
            {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler)
            }
        ]
    });
    /////////////////////////////////////////////////////////////////////////////
    
    let padded_size = align_to_multiple(img_width.max(img_height), 64);
    let padded_width = align_to_multiple(img_width, 64);
    let padded_height = align_to_multiple(img_height, 64);

    let output_texture_desc = wgpu::TextureDescriptor 
    {
        size: wgpu::Extent3d 
        {
            width: padded_width,//padded_size,
            height: padded_height, //padded_size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };
    let output_texture = device.create_texture(&output_texture_desc);
    let output_texture_view = output_texture.create_view(&Default::default());

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor 
    {
        size: (4 * padded_size * padded_size) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    });

    // let shader_source = include_str!("sobel_shader.wgsl");
    let shader_source = std::fs::read_to_string(shader_path).unwrap(); //No Error Handling
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor
    {
        label: Some("Sobel Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
    {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor 
    {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState 
        {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState 
        {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState 
            {
                format: output_texture_desc.format,
                blend: Some(wgpu::BlendState 
                {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState 
        {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState 
        {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor 
        {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment 
            {
                view: &output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations 
                {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.0, b: 1.0, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(0, &texture_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo
        {
            aspect: wgpu::TextureAspect::All,
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::TexelCopyBufferInfo 
        {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout 
            {
                offset: 0,
                bytes_per_row: Some(padded_width * 4),
                rows_per_image: Some(padded_height),
            },
        },
        output_texture_desc.size,
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| { tx.send(result).unwrap(); });
    device.poll(wgpu::Maintain::Wait);
    rx.receive().await.unwrap().unwrap();

    let data = 
    {
        let mapped_range = buffer_slice.get_mapped_range();
        let vec = mapped_range.to_vec();
        vec
    };
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(padded_width, padded_height, data).unwrap();
    output_buffer.unmap();
    image
}

fn align_to_multiple(value: u32, alignment: u32) -> u32
{
    (value + alignment - 1) & !(alignment - 1)
}