use ffmpeg_next as ffmpeg;

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
        },
        Err(e) => println!("Failed to open Video: {}", e),
    }
}