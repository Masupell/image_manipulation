use image::{DynamicImage, GenericImage, GenericImageView};

// Assuming GrayScale
pub fn sobel(img: &mut DynamicImage)
{
    let (width, height) = img.dimensions();
    let img_buffer = img.to_luma8().as_raw().clone();
    
    let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    let mut gradient: Vec<(f32, f32)> = vec![(0.0, 0.0); (width * height) as usize]; // Gradient, direction

    for y in 0..height
    {
        for x in 0..width
        {
            if x > 0 && y > 0 && x < width - 1 && y < height - 1
            {
                let img_matrix = 
                [
                    [img_buffer[((y-1)*width+x-1) as usize], img_buffer[((y-1)*width+x) as usize], img_buffer[((y-1)*width+x+1) as usize]],
                    [img_buffer[(y*width+x-1) as usize], img_buffer[(y*width+x) as usize], img_buffer[(y*width+x+1) as usize]],
                    [img_buffer[((y+1)*width+x-1) as usize], img_buffer[((y+1)*width+x) as usize], img_buffer[((y+1)*width+x+1) as usize]]
                ];
                
                let result_x = convolution(gx, img_matrix) as f32;
                let result_y = convolution(gy, img_matrix) as f32;
                let result = ((result_x.powf(2.0) + result_y.powf(2.0)) as f32).sqrt();
                let dir = result_y.atan2(result_x);

                
                gradient[(y * width + x) as usize] = (result, dir);
            }
            else 
            {
                gradient[(y * width + x) as usize] = (0.0, 0.0);
            }
        }
    }

    for y in 0..height as usize
    {
        for x in 0..width as usize
        {
            if x > 0 && y > 0 && (x as u32) < width - 1 && (y as u32) < height - 1
            {
                let result = gradient[y * width as usize + x].0;
                let dir = gradient[y * width as usize + x].1;

                let degrees = dir.to_degrees();
                let simple_degrees = if degrees >= -22.5 && degrees < 22.5 || degrees < -157.5 && degrees >= 157.5
                {
                    0
                } 
                else if degrees >= 22.5 && degrees < 67.5 || degrees < -112.5 && degrees >= -157.5
                {
                    45
                } 
                else if degrees >= 67.5 && degrees < 112.5 || degrees < -67.5 && degrees >= -112.5
                {
                    90
                } 
                else if degrees >= 112.5 && degrees < 157.5 || degrees < -22.5 && degrees >= -67.5
                {
                    135
                } 
                else 
                {
                    0
                };
                

                match simple_degrees
                {
                    0 =>
                    {
                        if result > gradient[y * width as usize + x -1].0 && result > gradient[y * width as usize + x + 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    45 =>
                    {
                        if result > gradient[(y-1) * width as usize + x - 1].0 && result > gradient[(y+1) * width as usize + x + 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    90 =>
                    {
                        if result > gradient[(y-1) * width as usize + x].0 && result > gradient[(y+1) * width as usize + x].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    135 =>
                    {
                        if result > gradient[(y-1) * width as usize + x + 1].0 && result > gradient[(y+1) * width as usize + x - 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

// fn map_to_direction(degrees)

fn convolution(kernel: [[i32; 3]; 3], matrix: [[u8; 3]; 3]) -> i32
{
    let mut sum = 0;
    for y in 0..matrix.len()
    {
        for x in 0..matrix[y].len()
        {
            sum += kernel[y][x] * matrix[y][x] as i32;
        }
    }
    sum
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    pub fn run()
    {
        // let input_img = image::open("tests/swan.jpg").unwrap();

        // let blurred = input_img.blur(5.0);
        // let mut gray = blurred.grayscale();
        // sobel(&mut gray);

        // gray.save("tests/result.png").unwrap();

        // pollster::block_on(sobel_on_gpu());
        sobel_gpu();
    }
}

pub fn sobel_cpu()
{
    let input_img = image::open("tests/swan.jpg").unwrap();

    let blurred = input_img.blur(5.0);
    let mut gray = blurred.grayscale();
    sobel(&mut gray);
    gray.save("/tests/result.png").unwrap();
}
pub fn sobel_gpu()
{
    let input_img = image::open("tests/swan.jpg").unwrap();
    let blurred = input_img.blur(5.0);
    let mut gray = blurred.grayscale();

    pollster::block_on(sobel_on_gpu("tests/swan.jpg"));
}



async fn sobel_on_gpu(path: &str) 
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
    None,).await.unwrap();

    // Input Texture
    /////////////////////////////////////////////////////////////////////////////
    let input_img = image::open(path).unwrap();
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
    img_size,);

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


    let texture_size = 512u32;
    let output_texture_desc = wgpu::TextureDescriptor 
    {
        size: wgpu::Extent3d 
        {
            width: texture_size,
            height: texture_size,
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
        size: (4 * texture_size * texture_size) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    });

    let shader_source = include_str!("sobel_shader.wgsl");
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
                bytes_per_row: Some(texture_size * 4),
                rows_per_image: Some(texture_size),
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
    use image::{ImageBuffer, Rgba};
    ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size, texture_size, data).unwrap().save("tests/sobel_gpu.png").unwrap();
    output_buffer.unmap();
}