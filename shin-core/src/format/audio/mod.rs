//! Support for NXA format, storing opus audio. (The format seems to be specific for Nintendo Switch?)

use anyhow::Result;
use binrw::{BinRead, BinWrite};
use opus::Channels;
use std::io::Read;

#[derive(BinRead, BinWrite, Debug)]
#[brw(little, magic = b"NXA1")]
#[br(assert(version == 2))]
struct NxaHeader {
    version: u32,
    file_size: u32,
    #[brw(align_after = 0x10)]
    info: AudioInfo,
}

/// Provides some metadata about the audio
///
/// Stuff that would usually go into opus headers
#[derive(BinRead, BinWrite, Debug, Clone)]
pub struct AudioInfo {
    /// Sample rate, in Hz.
    pub sample_rate: u32,
    /// Number of channels (usually 1 or 2).
    pub channel_count: u16,
    /// Size of frame in bytes
    pub frame_size: u16,
    /// Amount of samples in one frame
    pub frame_samples: u16,
    /// Amount of samples to skip, needed because of the way opus works
    pub pre_skip: u16,
    /// Amount of samples in the whole file
    pub num_samples: u32,
    /// Where to start playing after looping in samples.
    pub loop_start: u32,
    /// Where to loop from in samples (usually happens at the end of the file).
    pub loop_end: u32,
}

pub struct AudioFile {
    info: AudioInfo,
    data: Vec<u8>,
}

impl AudioFile {
    pub fn info(&self) -> &AudioInfo {
        &self.info
    }

    pub fn decode(self) -> Result<AudioDecoder<Self>> {
        AudioDecoder::new(self)
    }
}

impl AsRef<AudioFile> for AudioFile {
    fn as_ref(&self) -> &AudioFile {
        self
    }
}

pub struct AudioDecoder<F: AsRef<AudioFile>> {
    file: F,
    bytes_position: usize,
    /// Amount of samples that should be dropped when decoding next frame
    /// Used to implement pre-skip & seeking
    skip_samples: u64,
    buffer: Box<[f32]>,
    decoder: opus::Decoder,
}

impl<F: AsRef<AudioFile>> AudioDecoder<F> {
    pub fn new(file: F) -> Result<Self> {
        let info = &file.as_ref().info;
        let decoder = opus::Decoder::new(
            info.sample_rate,
            match info.channel_count {
                1 => Channels::Mono,
                2 => Channels::Stereo,
                _ => panic!("Unsupported channel count"),
            },
        )?;
        let buffer =
            vec![0.0; info.frame_samples as usize * info.channel_count as usize].into_boxed_slice();
        let skip_samples = info.pre_skip as u64;
        Ok(Self {
            file,
            bytes_position: 0,
            skip_samples,
            buffer,
            decoder,
        })
    }

    pub fn info(&self) -> &AudioInfo {
        &self.file.as_ref().info
    }

    pub fn samples_seek(&mut self, samples_position: u64) {
        assert!(samples_position <= self.info().num_samples as u64);

        // the decoder needs some time to converge, we probably should seek a little bit before and skip some samples
        // this is called "pre-roll" by the RFC7845 and recommends to use 3840 samples / 80 ms
        const PRE_ROLL: u64 = 3840;

        let samples_position = samples_position + self.pre_skip();

        // handle the case when samples_position is < PRE_ROLL
        let pre_roll = std::cmp::min(PRE_ROLL, samples_position);

        let samples_position = samples_position - pre_roll;
        let frames_position = samples_position / self.frame_samples();
        let bytes_position = frames_position * self.frame_size();
        let in_frame_position = samples_position % self.frame_samples();

        self.bytes_position = bytes_position.try_into().unwrap();
        self.skip_samples += in_frame_position + pre_roll;
        self.decoder.reset_state().unwrap();
    }

    /// Returns the current position of the decoder in samples
    ///
    /// This handles the pre-skip, so the position is relative to the actual start of the audio
    pub fn samples_position(&self) -> u64 {
        (self.bytes_position as u64 / self.frame_size()) * self.frame_samples()
            // skip samples are not accounted in the "bytes_position", so we need to add them
            + self.skip_samples
            // pre-skip is counted in the bytes_position and we want to hide it, so subtract it
            - self.pre_skip()
        // Note that at the start skip_samples will be equal to pre_skip, so we will return 0
    }

    fn frame_size(&self) -> u64 {
        self.info().frame_size as u64
    }

    fn frame_samples(&self) -> u64 {
        self.info().frame_samples as u64
    }

    fn pre_skip(&self) -> u64 {
        self.info().pre_skip as u64
    }

    /// Decodes one opus frame
    ///
    /// Returns the offset in the buffer to start reading from
    pub fn decode_frame(&mut self) -> Option<usize> {
        // the loop is here to handle pre-skips larger than one frame
        loop {
            let data = &self.file.as_ref().data;
            if self.bytes_position >= data.len() {
                return None;
            }

            let data = &data[self.bytes_position..][..self.frame_size() as usize];

            assert_eq!(
                self.decoder.get_nb_samples(data).unwrap() as u64,
                self.frame_samples()
            );

            let decoded = self
                .decoder
                .decode_float(data, &mut self.buffer, false)
                .unwrap();

            assert_eq!(decoded as u64, self.frame_samples());

            self.bytes_position += self.frame_size() as usize;

            if self.skip_samples > self.frame_samples() {
                self.skip_samples -= self.frame_samples();
            } else {
                self.skip_samples = 0;
                break Some(self.skip_samples as usize);
            }
        }
    }

    pub fn buffer(&self) -> &[f32] {
        &self.buffer
    }
}

/// Wrapper around the `AudioDecoder` that implements `Iterator` over individual samples
pub struct AudioDecoderIterator<F: AsRef<AudioFile>> {
    decoder: AudioDecoder<F>,
    buffer_position: usize,
}

impl<F: AsRef<AudioFile>> AudioDecoderIterator<F> {
    pub fn new(mut decoder: AudioDecoder<F>) -> Self {
        let buffer_position = match decoder.decode_frame() {
            None => decoder.buffer().len(),
            Some(pos) => pos,
        };

        Self {
            decoder,
            buffer_position,
        }
    }

    pub fn info(&self) -> &AudioInfo {
        self.decoder.info()
    }

    pub fn seek(&mut self, samples_position: u64) {
        self.decoder.samples_seek(samples_position);
        self.buffer_position = match self.decoder.decode_frame() {
            None => {
                // end of file, put the buffer position at the end to return `None` from the `next()`
                self.decoder.buffer().len()
            }
            Some(pos) => pos,
        }
    }

    pub fn position(&self) -> u64 {
        self.decoder.samples_position() + self.buffer_position as u64
    }
}

impl<F: AsRef<AudioFile>> Iterator for AudioDecoderIterator<F> {
    type Item = (f32, f32);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = self.decoder.buffer();

        if self.buffer_position >= buffer.len() {
            self.buffer_position = self.decoder.decode_frame()?;

            buffer = self.decoder.buffer();
        }

        match self.decoder.info().channel_count {
            1 => {
                let sample = buffer[self.buffer_position];
                self.buffer_position += 1;
                Some((sample, sample))
            }
            2 => {
                let sample = (
                    buffer[self.buffer_position],
                    buffer[self.buffer_position + 1],
                );
                self.buffer_position += 2;
                Some(sample)
            }
            _ => panic!(
                "Unsupported channel count {}",
                self.decoder.info().channel_count
            ),
        }
    }
}

pub fn read_audio(data: &[u8]) -> Result<AudioFile> {
    let mut cur = std::io::Cursor::new(data);
    let header = NxaHeader::read_le(&mut cur)?;

    assert_eq!(header.file_size, data.len() as u32);
    // how are we supposed to loop when the loop end is not in the end of the file?
    assert_eq!(header.info.loop_end, header.info.num_samples);

    let mut data = Vec::new();
    cur.read_to_end(&mut data)?;

    Ok(AudioFile {
        info: header.info,
        data,
    })
}
