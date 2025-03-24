use anyhow::Error;
use ffmpeg_next::{self as ffmpeg, codec::traits::Decoder};
use ffmpeg::{codec, format, decoder};

pub fn read()
{
    ffmpeg::init().unwrap();

    let path = "tests/Lucy_Video.mp4";

    match ffmpeg::format::input(&path)
    {
        Ok(input) => 
        {
            println!("Opened video: {}", path);
            println!("Format: {}", input.format().name());
            println!("Duration: {} seconds", input.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64);

            let video_stream = input.streams().best(ffmpeg::media::Type::Video).ok_or_else(|| anyhow::anyhow!("No video stream found")).unwrap();
            println!("Video Stream: Index: {}, Codec: {:?}", video_stream.index(), video_stream.parameters().id());

            let test = open_codec_context(&video_stream);
        },
        Err(e) => println!("Failed to open Video: {}", e),
    }
}

fn open_codec_context(video_stream: &ffmpeg::format::stream::Stream) -> Result<ffmpeg::codec::Context, ffmpeg::Error> 
{
    let codec_params = video_stream.parameters();

    let codec_id = codec_params.id();

    let codec = decoder::find(codec_id).ok_or(ffmpeg::Error::DecoderNotFound)?;

    // let mut codec_context = 
    
    codec_context.open()?;

    Ok(codec_context)
}