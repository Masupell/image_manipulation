use anyhow::Error;
use ffmpeg_next::{self as ffmpeg, codec::traits::Decoder};
use ffmpeg::{codec, format, decoder};

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

                                // // let test = debug_yuv_data(frame.width(), frame.height(), y_data, u_data, v_data);
                                // // let rgba_image = convert_yuv420p_to_rgba(frame.width(), frame.height(), y_data, u_data, v_data);
                                // // rgba_image.save("tests/frame01.png").unwrap();

                                // println!("Frame dimensions: {}x{}", frame.width(), frame.height());
                                // println!("Y Plane size: {}", y_data.len());
                                // println!("U Plane size: {}", u_data.len());
                                // println!("V Plane size: {}", v_data.len());

                                // // Print first few values for each plane
                                // for i in 0..10 {
                                //     println!(
                                //         "Y[{}]: {}, U[{}]: {}, V[{}]: {}",
                                //         i, y_data[i], i, u_data[i], i, v_data[i]
                                //     );
                                // }

                                let test = yuv420p_to_rgb(y_data, u_data, v_data, frame.width() as usize, frame.height() as usize, frame.stride(0), frame.stride(1));

                                let img = image::RgbImage::from_raw(frame.width(), frame.height(), test).unwrap();
                                img.save("tests/frame01.png").unwrap();
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

// fn yuv420p_to_rgb(width: u32, height: u32, frame_data: &[u8])
// {
//     let total_size = width * height;

//     // let y = frame_data[]
// }

use image::{ImageBuffer, Rgba};

// Function to convert YUV420P to RGBA
fn yuv420p_to_rgb(y_data: &[u8], u_data: &[u8], v_data: &[u8], width: usize, height: usize, y_stride: usize, uv_stride: usize) -> Vec<u8> {
    let mut rgb_data = vec![0u8; width * height * 3];

    let mut rgb_buffer = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let y_index = y * y_stride + x;
            let uv_index = (y / 2) * uv_stride + (x / 2);

            let y_value = y_data[y_index] as f32;
            let u_value = u_data[uv_index] as f32 - 128.0;
            let v_value = v_data[uv_index] as f32 - 128.0;

            // let r = y_value + 1.402 * (v_value - 128.0);
            // let g = y_value - 0.344136 * (u_value - 128.0) -0.714136 * (v_value - 128.0);
            // let b = y_value + 1.772 * (u_value - 128.0);

            let r = (y_value + 1.402 * v_value).clamp(0.0, 255.0) as u8;
            let g = (y_value - 0.344 * u_value - 0.714 * v_value).clamp(0.0, 255.0) as u8;
            let b = (y_value + 1.772 * u_value).clamp(0.0, 255.0) as u8;

            rgb_buffer.push(r as u8);
            rgb_buffer.push(g as u8);
            rgb_buffer.push(b as u8);
            // rgba_buffer.push(255); // Full alpha
        }
    }

    rgb_buffer
}

fn debug_yuv_data(width: u32, height: u32, y_data: &[u8], u_data: &[u8], v_data: &[u8]) 
{
    for y in 0..height {
        for x in 0..width {
            let y_idx = (y * width + x) as usize;
            let u_idx = ((y / 2) * (width / 2) + (x / 2)) as usize;
            let v_idx = ((y / 2) * (width / 2) + (x / 2)) as usize;

            let y_val = y_data[y_idx];
            let u_val = u_data[u_idx];
            let v_val = v_data[v_idx];

            println!("Y: {} U: {} V: {}", y_val, u_val, v_val);
        }
    }
}