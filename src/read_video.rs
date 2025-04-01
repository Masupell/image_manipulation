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

                                let buf = yuv420p_to_rgb(y_data, u_data, v_data, frame.width() as usize, frame.height() as usize, frame.stride(0), frame.stride(1));
                                let img = image::RgbImage::from_raw(frame.height(), frame.width(), buf).unwrap(); // frame.hwight and width are inverted
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