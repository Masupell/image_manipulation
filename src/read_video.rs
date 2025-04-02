use anyhow::Error;
use ffmpeg_next::{self as ffmpeg, codec::traits::Decoder};
use ffmpeg::{codec, format, decoder};
use image::{DynamicImage, GenericImage, GenericImageView};

pub fn read()
{
    ffmpeg::init().unwrap();

    let path = "tests/Lucy_Video.mp4";

    match ffmpeg::format::input(&path)
    {
        Ok(mut input) => 
        {
            println!("Opened video: {}", path);
            println!("Format: {}", input.format().name());
            println!("Duration: {} seconds", input.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64);

            let video_stream_index = input.streams().best(ffmpeg::media::Type::Video).map(|s| s.index()).ok_or_else(|| anyhow::anyhow!("No Video-Stream found")).unwrap();

            let mut decoder = open_codec_context(&input.stream(video_stream_index).unwrap()).unwrap();

            let width = decoder.width();
            let height = decoder.height();
            println!("Width: {}, Height: {}", width, height);

            let mut test = 0;

            for (stream_index, packet) in input.packets() 
            {
                if stream_index.index() == video_stream_index 
                {
                    if decoder.send_packet(&packet).is_ok() 
                    {
                        let mut frame = ffmpeg::frame::Video::empty();

                        while decoder.receive_frame(&mut frame).is_ok()
                        {
                            test += 1;
                            println!("Decoded frame: {}x{}  Frame: {}", frame.width(), frame.height(), test);

                            println!("Frame Format: {:?}", frame.format());

                            if test == 1
                            {
                                let y_data = frame.data(0);
                                let u_data = frame.data(1);
                                let v_data = frame.data(2);

                                let buf = yuv420p_to_rgb(y_data, u_data, v_data, frame.width() as usize, frame.height() as usize, frame.stride(0), frame.stride(1));
                                let img = image::RgbImage::from_raw(frame.height(), frame.width(), buf).unwrap(); // frame.hwight and width are inverted
                                let test = image::DynamicImage::ImageRgb8(img);

                                // let input_img = image::open("tests/koala.webp").unwrap();
                                let blurred = test.blur(5.0);
                                // let gray = blurred.grayscale();
                            
                                pollster::block_on(crate::image_shader::image_shader(blurred, "src/shader/sobel_operator.wgsl"));

                                // img.save("tests/frame01.png").unwrap();
                            }
                            
                        }
                    }
                }
            }
        },
        Err(e) => println!("Failed to open Video: {}", e),
    }
}

fn open_codec_context(video_stream: &ffmpeg::format::stream::Stream) -> Result<ffmpeg::decoder::Video, ffmpeg::Error> 
{
    let codec_params = video_stream.parameters();

    let mut context = ffmpeg::codec::Context::new();
    context.set_parameters(codec_params)?;

    let decoder = context.decoder().video()?;

    Ok(decoder)
}

fn yuv420p_to_rgb(y_data: &[u8], u_data: &[u8], v_data: &[u8], width: usize, height: usize, y_stride: usize, uv_stride: usize) -> Vec<u8> 
{
    let mut rgb_buffer = vec![0u8; width * height * 3];

    for y in 0..height 
    {
        for x in 0..width 
        {
            let y_index = y * y_stride + x;
            let uv_index = (y / 2) * uv_stride + (x / 2);

            let y_value = y_data[y_index] as f32;
            let u_value = u_data[uv_index] as f32 - 128.0;
            let v_value = v_data[uv_index] as f32 - 128.0;

            let r = y_value + 1.402 * v_value;
            let g = y_value - 0.344136 * u_value -0.714136 * v_value;
            let b = y_value + 1.772 * u_value;

            // let index = (y * width + x) * 3; // Normal index
            let index = (height -1 - y + height * x) * 3; // 90 degrees clockwise rotation (because width and height of frame are swapped)
            rgb_buffer[index]     = r as u8;
            rgb_buffer[index + 1] = g as u8;
            rgb_buffer[index + 2] = b as u8;
        }
    }
    rgb_buffer
}








//Input comes from yuv420p, which has width and height flipped (so need to flip it as well when using this fn)
pub async fn shader_setup(img_width: u32, img_height: u32, shader_path: &str) -> (wgpu::Device, wgpu::Queue, wgpu::BindGroup, wgpu::Texture, wgpu::Texture, wgpu::TextureView, wgpu::TextureDescriptor, wgpu::Buffer, wgpu::RenderPipeline, wgpu::Extent3d, u32, u32)
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

    // Writing texture into the queue, but depends on input_img (so have to do it later)



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

    let shader_source = std::fs::read_to_string(shader_path).unwrap(); //No Error Handling
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor
    {
        label: Some("Shader"),
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

    (device, queue, texture_bind_group, input_texture, output_texture, output_texture_view, output_texture_desc, output_buffer, render_pipeline, img_size, padded_width, padded_height)
}

fn align_to_multiple(value: u32, alignment: u32) -> u32 
{
    (value + alignment - 1) & !(alignment - 1)
}





pub fn read_video()
{
    ffmpeg::init().unwrap();

    let path = "tests/Lucy_Video.mp4";

    match ffmpeg::format::input(&path)
    {
        Ok(mut input) => 
        {
            let video_stream_index = input.streams().best(ffmpeg::media::Type::Video).map(|s| s.index()).ok_or_else(|| anyhow::anyhow!("No Video-Stream found")).unwrap();

            let mut decoder = open_codec_context(&input.stream(video_stream_index).unwrap()).unwrap();

            let width = decoder.width();
            let height = decoder.height();

            let (device, queue, texture_bind_group, input_texture, output_texture, output_texture_view, output_texture_desc, output_buffer, render_pipeline, img_size, padded_width, padded_height) = pollster::block_on(shader_setup(width as u32, height as u32, "src/shader/sobel_operator.wgsl"));

            let mut test_vector = Vec::new();

            for (stream_index, packet) in input.packets()
            {
                if stream_index.index() == video_stream_index 
                {
                    if decoder.send_packet(&packet).is_ok() 
                    {
                        let mut frame = ffmpeg::frame::Video::empty();

                        while decoder.receive_frame(&mut frame).is_ok()
                        {
                            let y_data = frame.data(0);
                            let u_data = frame.data(1);
                            let v_data = frame.data(2);
                            
                            let buf = yuv420p_to_rgb(y_data, u_data, v_data, frame.width() as usize, frame.height() as usize, frame.stride(0), frame.stride(1));
                            let img = image::RgbImage::from_raw(frame.height(), frame.width(), buf).unwrap(); // frame.hwight and width are inverted
                            let input_img = image::DynamicImage::ImageRgb8(img);
                            
                            let (width, height, data) = pollster::block_on(send_frame(&device, &queue, &input_texture, &output_texture, &output_texture_view, &output_texture_desc, &render_pipeline, &output_buffer, padded_width, padded_height, &texture_bind_group, img_size, width, height, input_img));
                            test_vector.push((width, height, data));
                        }
                    }
                }
            }
        },
        Err(e) => println!("Failed to open Video: {}", e),
    }
}



pub async fn send_frame<'a>(device: &wgpu::Device, queue: &wgpu::Queue, input_texture: &wgpu::Texture, output_texture: &wgpu::Texture, 
    output_texture_view: &wgpu::TextureView, output_texture_desc: &wgpu::TextureDescriptor<'a>, render_pipeline: &wgpu::RenderPipeline, output_buffer: &wgpu::Buffer, padded_width: u32, padded_height: u32,
    texture_bind_group: &wgpu::BindGroup, img_size: wgpu::Extent3d, img_width: u32, img_height: u32, input_img: DynamicImage) -> (u32, u32, Vec<u8>)
{
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
        render_pass.set_bind_group(0, texture_bind_group, &[]);
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

    let data = buffer_slice.get_mapped_range();
    let image_data = data.to_vec();
    output_buffer.unmap();

    (padded_width, padded_height, image_data)
}