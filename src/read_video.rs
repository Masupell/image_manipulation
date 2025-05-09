use anyhow::Error;
use ffmpeg_next::{self as ffmpeg, codec::{self, traits::{Decoder, Encoder}}, device::input, encoder, format, Rational};
use image::{DynamicImage, GenericImage, GenericImageView};
use crate::ffmpeg_transcoder::video;

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

            if y == 0 && x == 0
            {
                println!("Y_Index: {}\nUV_Index: {}", ((height-1)*y_stride + width-1), ((height-1)/2)*uv_stride + ((width-1)/2));
            }

            let y_value = y_data[y_index] as f32;
            let u_value = u_data[uv_index] as f32 - 128.0;
            let v_value = v_data[uv_index] as f32 - 128.0;

            let r = y_value + 1.402 * v_value;
            let g = y_value - 0.344136 * u_value -0.714136 * v_value;
            let b = y_value + 1.772 * u_value;

            let index = (y * width + x) * 3; // Normal index
            // let index = (height -1 - y + height * x) * 3; // 90 degrees clockwise rotation (because width and height of frame are swapped)
            // Actually, just learned. it is because portrait videos from phone have metatags or so, that I can not add myself
            //So unless it is a portrait video from a phone, I should not rotate
            rgb_buffer[index]     = r as u8;
            rgb_buffer[index + 1] = g as u8;
            rgb_buffer[index + 2] = b as u8;
        }
    }
    rgb_buffer
}

fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

fn rgb_to_yuv420p(rgb_data: &[u8], width: usize, height: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>)
{
    let mut y_plane = vec![0u8; width * height];
    // let mut u_plane = vec![0u8; (width * height) / 4];
    // let mut v_plane = vec![0u8; (width * height) / 4];
    let uv_width = div_ceil(width, 2);
    let uv_height = div_ceil(height, 2);
    let uv_plane_len = uv_width * uv_height;
    let mut u_plane = vec![0u8; uv_plane_len];
    let mut v_plane = vec![0u8; uv_plane_len];

    for y in 0..height
    {
        for x in 0..width
        {
            let index = (y * width + x) * 3;
            let r = rgb_data[index] as f32;
            let g = rgb_data[index + 1] as f32;
            let b = rgb_data[index + 2] as f32;
        
            // let y_value = (0.299_f32 * r + 0.587_f32 * g + 0.114_f32 * b).round();
            let y_value = (16.0_f32 + 0.257_f32 * r + 0.504_f32 * g + 0.098_f32 * b).round();
        
            y_plane[y * width + x] = y_value.clamp(0.0, 255.0) as u8;
        }
    }

    for y in (0..height).step_by(2)
    {
        for x in (0..width).step_by(2)
        {
            let mut sum_u = 0.0;
            let mut sum_v = 0.0;

            let mut count = 0.0;

            for dy in 0..2
            {
                for dx in 0..2
                {
                    let px = x + dx;
                    let pz = y + dy;

                    if px < width && pz < height
                    {
                        let index = (pz * width + px) * 3;

                        let r = rgb_data[index] as f32;
                        let g = rgb_data[index + 1] as f32;
                        let b = rgb_data[index + 2] as f32;
    
                        // let u = -0.169 * r - 0.331 * g + 0.5 * b + 128.0;
                        // let v = 0.5 * r - 0.419 * g - 0.081 * b + 128.0;
                        let u = -0.148 * r - 0.291 * g + 0.439 * b + 128.0;
                        let v =  0.439 * r - 0.368 * g - 0.071 * b + 128.0;
                        sum_u += u;
                        sum_v += v;

                        count += 1.0;
                    }
                }
            }

            let avg_u = (sum_u / count).round().clamp(0.0, 255.0) as u8;
            let avg_v = (sum_v / count).round().clamp(0.0, 255.0) as u8;

            let uv_index = (y / 2) * (width / 2) + (x / 2);
            u_plane[uv_index] = avg_u;
            v_plane[uv_index] = avg_v;
        }
    }

    (y_plane,u_plane,v_plane)
}


pub fn test()
{
    let input_img = image::open("tests/koala.webp").unwrap();
    let test_to_yu420p = rgb_to_yuv420p(input_img.to_rgb8().as_raw(), input_img.width() as usize, input_img.height() as usize);
    println!("Size of: \nY: {}\nU: {}\nV: {}", test_to_yu420p.0.len(), test_to_yu420p.1.len(), test_to_yu420p.2.len());
    let test_to_rgb = yuv420p_to_rgb(&test_to_yu420p.0, &test_to_yu420p.1, &test_to_yu420p.2, input_img.width() as usize, input_img.height() as usize, input_img.width() as usize, input_img.width() as usize / 2);
    let output_img = image::RgbImage::from_raw(input_img.width(), input_img.height(), test_to_rgb).unwrap();

    let grayimage = image::GrayImage::from_vec(input_img.width(), input_img.height(), test_to_yu420p.0).unwrap();

    grayimage.save("tests/koala_y.png").unwrap();

    let width = input_img.width() as usize;
    let height = input_img.height() as usize;
    let mut full_res_plane = vec![0u8; width * height];

    for y in 0..height
    {
        for x in 0..width
        {
            let uv_x = x /2;
            let uv_y = y/2;
            let uv_index = (uv_y * (width / 2)) + uv_x;
            full_res_plane[y * width + x] = test_to_yu420p.1[uv_index];
        }
    }

    let grayimage_u = image::GrayImage::from_vec(width as u32, height as u32, full_res_plane).unwrap();
    grayimage_u.save("tests/koala_u.png").unwrap();

    let mut full_res_plane = vec![0u8; width * height];

    for y in 0..height
    {
        for x in 0..width
        {
            let uv_x = x /2;
            let uv_y = y/2;
            let uv_index = (uv_y * (width / 2)) + uv_x;
            full_res_plane[y * width + x] = test_to_yu420p.2[uv_index];
        }
    }

    let grayimage_v = image::GrayImage::from_vec(width as u32, height as u32, full_res_plane).unwrap();
    grayimage_v.save("tests/koala_v.png").unwrap();

    output_img.save("tests/koala2.png").unwrap();


    // ffmpeg::init().unwrap();

    // let path = "tests/Lucy_Video.mp4";

    // match ffmpeg::format::input(&path)
    // {
    //     Ok(mut input) => 
    //     {
    //         let video_stream_index = input.streams().best(ffmpeg::media::Type::Video).map(|s| s.index()).ok_or_else(|| anyhow::anyhow!("No Video-Stream found")).unwrap();

    //         let mut decoder = open_codec_context(&input.stream(video_stream_index).unwrap()).unwrap();

    //         let width = decoder.width();
    //         let height = decoder.height();

    //         let mut temp = 0;

    //         for (stream_index, packet) in input.packets()
    //         {
    //             if stream_index.index() == video_stream_index 
    //             {
    //                 if decoder.send_packet(&packet).is_ok() 
    //                 {
    //                     let mut frame = ffmpeg::frame::Video::empty();

    //                     while decoder.receive_frame(&mut frame).is_ok()
    //                     {
    //                         if temp == 0
    //                         {
    //                             let y_data = frame.data(0);
    //                             let u_data = frame.data(1);
    //                             let v_data = frame.data(2);
                                
    //                             let rgb = yuv420p_to_rgb(y_data, u_data, v_data, width as usize, height as usize, frame.stride(0), frame.stride(1));
    //                             println!("y_stride: {}, uv_stride: {}", frame.stride(0), frame.stride(1));
    //                             println!("y_data: {}, u_data: {}, v_data: {}", y_data.len(), u_data.len(), v_data.len());
    //                             let img = image::RgbImage::from_raw(frame.height(), frame.width(), rgb).unwrap();
    //                             img.save("tests/rgb_function_test.png").unwrap();
    //                         }

    //                         temp += 1;
    //                     }
    //                 }
    //             }
    //         }
    //     },
    //     Err(e) => println!("Failed to open Video: {}", e),
    // }
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
    video("tests/Drone.mp4", "tests/Drone_result.mp4");


//     ffmpeg::init().unwrap();

//     let path = "tests/Drone.mp4";
//     // let mut test_vector = Vec::new();


//     let mut input = ffmpeg::format::input(path).unwrap();
//     let mut output = ffmpeg::format::output("tests/Drone_result.mp4").unwrap();

//     let video_stream_index = input.streams().best(ffmpeg::media::Type::Video).map(|s| s.index()).ok_or_else(|| anyhow::anyhow!("No Video-Stream found")).unwrap();
//     let input_time_base = input.stream(video_stream_index).unwrap().time_base();

//     let mut decoder = ffmpeg::codec::context::Context::from_parameters(input.stream(video_stream_index).unwrap().parameters()).unwrap().decoder().video().unwrap();

//     let codec = ffmpeg::codec::encoder::find(codec::Id::H264).unwrap();

//     let global_header = output.format().flags().contains(format::Flags::GLOBAL_HEADER);
//     let mut output_stream = output.add_stream(codec).unwrap();

//     let mut encoder = ffmpeg::codec::context::Context::new_with_codec(codec).encoder().video().unwrap();

//     output_stream.set_parameters(&encoder);
//     encoder.set_height(decoder.height());
//     encoder.set_width(decoder.width());
//     encoder.set_aspect_ratio(decoder.aspect_ratio());
//     encoder.set_format(decoder.format());
//     encoder.set_frame_rate(decoder.frame_rate());
//     encoder.set_time_base(input.stream(video_stream_index).unwrap().time_base());

//     if global_header
//     {
//         encoder.set_flags(codec::Flags::GLOBAL_HEADER);
//     }

//     let mut opened_encoder = encoder.open().unwrap();
//     output_stream.set_parameters(&opened_encoder);
//     output_stream.set_time_base(opened_encoder.time_base());


//     output.set_metadata(input.metadata().to_owned());

//     println!("Input time base: {:?}", input_time_base);
// println!("Encoder time base: {:?}", opened_encoder.time_base());

//     output.write_header().unwrap();

//     let mut frames = 0;

//     for (stream, packet) in input.packets() 
//     {
//         if stream.index() == video_stream_index
//         {
//             decoder.send_packet(&packet).unwrap();

//             let mut frame = ffmpeg::frame::Video::empty();

//             while decoder.receive_frame(&mut frame).is_ok()
//             {
//                 frames += 1;
//                 let time_stamp = frame.timestamp();
//                 frame.set_pts(time_stamp);
//                 frame.set_kind(ffmpeg::picture::Type::None);
//                 opened_encoder.send_frame(&frame).unwrap();

//                 let mut received_packet = ffmpeg::Packet::empty();
//                 while opened_encoder.receive_packet(&mut received_packet).is_ok()
//                 {
//                     received_packet.set_stream(0);
//                     received_packet.rescale_ts(input_time_base, opened_encoder.time_base());
//                     received_packet.write_interleaved(&mut output).unwrap();
//                 }
//             }
//         }
//     }
//     println!("Send Frames: {}", frames);
//     frames = 0;

//     //Flush
//     decoder.send_eof().unwrap();
//     let mut frame = ffmpeg::frame::Video::empty();
//     while decoder.receive_frame(&mut frame).is_ok()
//     {
//         frames += 1;
//         let time_stamp = frame.timestamp();
//         frame.set_pts(time_stamp);
//         frame.set_kind(ffmpeg::picture::Type::None);
//         opened_encoder.send_frame(&frame).unwrap();

//         let mut received_packet = ffmpeg::Packet::empty();
//         while opened_encoder.receive_packet(&mut received_packet).is_ok()
//         {
//             received_packet.set_stream(0);
//             received_packet.rescale_ts(input_time_base, opened_encoder.time_base());
//             received_packet.write_interleaved(&mut output).unwrap();
//         }
//     }
//     println!("Received Frames: {}", frames);
//     opened_encoder.send_eof().unwrap();

//     let mut received_packet = ffmpeg::Packet::empty();
//     while opened_encoder.receive_packet(&mut received_packet).is_ok()
//     {
//         received_packet.set_stream(0);
//         received_packet.rescale_ts(input_time_base, opened_encoder.time_base());
//         received_packet.write_interleaved(&mut output).unwrap();
//     }
//     output.write_trailer().unwrap();







    // let (mut width, mut height) = (0, 0);

    // match ffmpeg::format::input(&path)
    // {
    //     Ok(mut input) => 
    //     {
    //         let video_stream_index = input.streams().best(ffmpeg::media::Type::Video).map(|s| s.index()).ok_or_else(|| anyhow::anyhow!("No Video-Stream found")).unwrap();

    //         let mut decoder = open_codec_context(&input.stream(video_stream_index).unwrap()).unwrap();

    //         // let metadata = input.stream(video_stream_index).unwrap().metadata().to_owned();
    //         // println!("////////////////////////////////////////////////////////////////////////////////////////////////");
    //         // for (key, value) in metadata.iter() {
    //         //     println!("{}: {}", key, value);
    //         // }
    //         // println!("////////////////////////////////////////////////////////////////////////////////////////////////");

    //         // let sidedata: Vec<Vec<u8>> = input.stream(video_stream_index).unwrap().side_data().map(|data| data.data().to_vec()).collect();

    //         // for data in sidedata.iter()
    //         // {
    //         //     // println!("Side Data: type: {:?}, size: {}", data.kind(), data.data().len());
    //         //     println!("{:?}", data)
    //         // }
    //         println!("////////////////////////////////////////////////////////////////////////////////////////////////");

    //         let width = decoder.width();
    //         let height = decoder.height();

    //         let (time_base, frame_rate) = (input.stream(video_stream_index).unwrap().time_base(), input.stream(video_stream_index).unwrap().rate());
    //         println!("{}\n{}", time_base.denominator(), frame_rate.numerator());
    //         // let (device, queue, texture_bind_group, input_texture, output_texture, output_texture_view, output_texture_desc, output_buffer, render_pipeline, img_size, padded_width, padded_height) = pollster::block_on(shader_setup(width as u32, height as u32, "src/shader/sobel_operator.wgsl"));

    //         let mut temp = 0;

    //         for (stream_index, packet) in input.packets()
    //         {
    //             if stream_index.index() == video_stream_index 
    //             {
    //                 if decoder.send_packet(&packet).is_ok() 
    //                 {
    //                     let mut frame = ffmpeg::frame::Video::empty();

    //                     while decoder.receive_frame(&mut frame).is_ok()
    //                     {
    //                         // let pts = temp * time_base.denominator() as i64 / frame_rate.numerator() as i64;
    //                         // frame.set_pts(Some(pts));
                            
    //                         temp += 1;
                            
    //                         let y_data = frame.data(0);
    //                         let u_data = frame.data(1);
    //                         let v_data = frame.data(2);

                            
    //                         // let buf = yuv420p_to_rgb(y_data, u_data, v_data, frame.width() as usize, frame.height() as usize, frame.stride(0), frame.stride(1));
    //                         // let img = image::RgbImage::from_raw(frame.height(), frame.width(), buf).unwrap(); // frame.hwight and width are inverted
    //                         // let input_img = image::DynamicImage::ImageRgb8(img);
                            
    //                         // let img = pollster::block_on(crate::image_shader::image_shader(input_img, "src/shader/sobel_operator.wgsl"));
    //                         // test.save("image_test.png").unwrap();

    //                         // let (width, height, data) = pollster::block_on(send_frame(&device, &queue, &input_texture, &output_texture, &output_texture_view, &output_texture_desc, &render_pipeline, &output_buffer, padded_width, padded_height, &texture_bind_group, img_size, width, height, input_img));
    //                         test_vector.push(frame.clone());
                            
    //                     }
    //                 }
    //             }
    //         }
    //         println!("Amount input frames: {}", temp);



    //         let mut output = ffmpeg::format::output("tests/video_result.mp4").unwrap();

    //         let codec = ffmpeg::codec::encoder::find(codec::Id::H264).unwrap();

    //         let mut stream = output.add_stream(codec).unwrap();

    //         let stream_index = stream.index();

    //         let mut encoder_context = ffmpeg::codec::context::Context::new_with_codec(codec).encoder().video().unwrap();

    //         encoder_context.set_width(width);
    //         encoder_context.set_height(height);
    //         encoder_context.set_format(ffmpeg::format::Pixel::YUV420P);
    //         encoder_context.set_time_base(time_base);
    //         encoder_context.set_frame_rate(Some(frame_rate));

    //         stream.set_parameters(&encoder_context);
    //         // stream.set_metadata(metadata);


    //         output.write_header().unwrap();

    //         let mut encoder = encoder_context.open().unwrap();

    //         encoder.set_max_b_frames(0);
    //         encoder.set_gop(1);

    //         let pts_step = time_base.denominator() / frame_rate.numerator(); // Should maybe be float?

    //         println!("Amount of Frames: {}", test_vector.len());

    //         // println!("clone  -  Frame");
    //         for (i, frame) in test_vector.iter().enumerate()
    //         {
    //             let mut cloned = frame.clone();
    //             cloned.set_pts(Some(i as i64 * pts_step as i64));
    //             // println!("{:?}  -  {:?}", cloned.pts(), frame.pts());
    //             encoder.send_frame(&cloned).unwrap();

    //             let mut packet = ffmpeg::packet::Packet::empty();
    //             while encoder.receive_packet(&mut packet).is_ok()
    //             {
    //                 if unsafe { packet.is_empty() }
    //                 {
    //                     continue;
    //                 }
                    
    //                 packet.set_stream(stream_index);
    //                 packet.rescale_ts(encoder.time_base(), time_base);
    //                 packet.write_interleaved(&mut output).unwrap();
    //             }
    //         }

    //         encoder.send_eof().unwrap();

    //         let mut packet = ffmpeg::packet::Packet::empty();
    //         let mut received = 0;
    //         while encoder.receive_packet(&mut packet).is_ok()
    //         {
    //             if unsafe { packet.is_empty() }
    //             {
    //                 continue;
    //             }

    //             packet.set_stream(stream_index);
    //             packet.rescale_ts(encoder.time_base(), time_base);
    //             packet.write_interleaved(&mut output).unwrap();
    //             received += 1;
    //             println!("Packet {} received", received);
    //         }
    //         println!("Recieved Frames: {}", received);

    //         output.write_trailer().unwrap();

    //     },
    //     Err(e) => println!("Failed to open Video: {}", e),
    // }
    
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