use std::{
    borrow::Cow,
    fs::File,
    io,
    io::{BufWriter, Cursor},
};

use anyhow::Context;
use binrw::BinWrite;
use hound::WavSpec;
use ogg::PacketWriteEndInfo;
use shin_core::format::audio::{AudioInfo, AudioSource};

use crate::AudioCommand;

#[derive(BinWrite)]
#[brw(magic(b"OpusHead"))]
struct OpusIdHeader {
    pub version: u8,
    pub channel_count: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: u16,
    pub mapping_family: u8,
}

struct OpusOggWriter<'writer, W: io::Write> {
    inner: ogg::writing::PacketWriter<'writer, W>,
    frame_idx: usize,
    frame_samples: usize,
}

impl<'writer, W: io::Write> OpusOggWriter<'writer, W> {
    const SERIAL: u32 = 42;

    pub fn new(inner: W, audio_info: &AudioInfo) -> io::Result<Self> {
        let mut inner = ogg::writing::PacketWriter::new(inner);

        // write the Opus Identification Header
        assert!(matches!(audio_info.channel_count, 1 | 2));

        let opus_id_header = OpusIdHeader {
            version: 1,
            channel_count: audio_info.channel_count as _,
            pre_skip: audio_info.pre_skip,
            input_sample_rate: audio_info.sample_rate,
            output_gain: 0,
            mapping_family: 0,
        };
        let mut opus_id_header_bytes = [0; 19];
        opus_id_header
            .write_le(&mut Cursor::new(opus_id_header_bytes.as_mut_slice()))
            .unwrap();

        inner.write_packet(
            Cow::Owned(Vec::from(&opus_id_header_bytes)),
            Self::SERIAL,
            PacketWriteEndInfo::EndPage,
            0,
        )?;

        // write the Opus Comment Header
        // vendor string length + vendor string + user comment count (always 0)
        let opus_comment_header_bytes = b"OpusTags\x11\x00\x00\x00sdu remuxer\x00\x00\x00\x00";

        inner.write_packet(
            opus_comment_header_bytes,
            Self::SERIAL,
            PacketWriteEndInfo::EndPage,
            0,
        )?;

        Ok(Self {
            inner,
            frame_idx: 0,
            frame_samples: audio_info.frame_samples as _,
        })
    }

    // accepting Vec<u8> here to make life easier
    pub fn write_frame(&mut self, frame: Vec<u8>, is_last_frame: bool) -> io::Result<()> {
        self.frame_idx += 1;
        let samples_count = self.frame_idx as u64 * self.frame_samples as u64;

        self.inner.write_packet(
            frame,
            Self::SERIAL,
            if is_last_frame {
                PacketWriteEndInfo::EndPage
            } else {
                PacketWriteEndInfo::NormalPacket
            },
            samples_count,
        )
    }
}

pub fn audio_command(command: AudioCommand) -> anyhow::Result<()> {
    match command {
        AudioCommand::Decode {
            audio_path,
            output_path,
        } => {
            let audio = std::fs::read(audio_path).context("Reading input file")?;
            let audio = shin_core::format::audio::read_audio(&audio)?;

            let info = audio.info().clone();

            let writer = File::create(output_path).context("Creating output file")?;
            let writer = BufWriter::new(writer);
            let mut writer = hound::WavWriter::new(
                writer,
                WavSpec {
                    channels: info.channel_count,
                    sample_rate: info.sample_rate,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )
            .context("Creating WAV writer")?;

            let mut audio_source = AudioSource::new(audio.decode().context("Creating decoder")?);

            while let Some((left, right)) = audio_source.read_sample() {
                writer.write_sample(left).context("Writing sample")?;
                writer.write_sample(right).context("Writing sample")?;
            }

            writer.finalize().context("Finalizing the WAV file")?;

            Ok(())
        }
        AudioCommand::Remux {
            audio_path,
            output_path,
        } => {
            let audio = std::fs::read(audio_path).context("Reading input file")?;
            let audio = shin_core::format::audio::read_audio(&audio)?;

            let info = audio.info().clone();

            let mut frame_reader = audio.read_frames();

            let writer = File::create(output_path).context("Creating output file")?;
            let writer = BufWriter::new(writer);
            let mut writer = OpusOggWriter::new(writer, &info).context("Creating OGG writer")?;

            while let Some(frame) = frame_reader.get_next_frame() {
                // allocating here to make life easier
                writer
                    .write_frame(Vec::from(frame), !frame_reader.has_next_frame())
                    .context("Writing frame")?;
            }

            Ok(())
        }
    }
}
