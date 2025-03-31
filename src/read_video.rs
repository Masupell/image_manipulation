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

            for (stream_index, packet) in input.packets() 
            {
                if stream_index.index() == video_stream_index 
                {
                    if decoder.send_packet(&packet).is_ok() 
                    {
                        let mut frame = ffmpeg::frame::Video::empty();

                        while decoder.receive_frame(&mut frame).is_ok() 
                        {
                            println!("Decoded frame: {}x{}", frame.width(), frame.height());
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