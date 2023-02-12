mod ffmpeg {
    use ffmpeg_next as ffmpeg;
    use ffmpeg_next::format::Pixel;
    use ffmpeg_next::media::Type;
    use ffmpeg_next::software::scaling;
    use shin_video::ffmpeg::input_reader;
    use std::fs::File;

    pub fn main() {
        let file = File::open("BigBuckBunny.mp4").unwrap();
        let (mut ictx, _avio) = unsafe { input_reader(file).unwrap() };

        // let input = ictx.streams().best(Type::Video).unwrap();
        // let video_stream_index = input.index();

        for (i, stream) in ictx.streams().enumerate() {
            println!("stream {}: time_base={}", i, stream.time_base());
        }

        // let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        // let mut decoder = context_decoder.decoder().video()?;
        //
        // let mut scaler = scaling::Context::get(
        //     decoder.format(),
        //     decoder.width(),
        //     decoder.height(),
        //     Pixel::RGB24,
        //     decoder.width(),
        //     decoder.height(),
        //     scaling::Flags::BILINEAR,
        // )?;

        for (stream, packet) in ictx.packets() {
            let time_base: f64 = stream.time_base().into();
            println!(
                "pts={:07.2}, s={}, packet: {} bytes",
                packet.pts().unwrap_or(0) as f64 * time_base,
                stream.index(),
                packet.data().map_or(0, |d| d.len())
            );
        }
    }
}

fn main() {}
