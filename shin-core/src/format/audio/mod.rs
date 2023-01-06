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

#[derive(BinRead, BinWrite, Debug)]
pub struct AudioInfo {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub frame_size: u16,
    pub frame_samples: u16,
    pub pre_skip: u16,
    pub num_samples: u32,
    pub loop_start: u32,
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
    position: usize,
    pre_skip: usize,
    buffer: Box<[f32]>,
    decoder: opus::Decoder,
}

impl<F: AsRef<AudioFile>> AudioDecoder<F> {
    pub fn new(file: F) -> Result<Self> {
        let info = &file.as_ref().info;
        assert_eq!(info.channel_count, 2);
        let decoder = opus::Decoder::new(info.sample_rate, Channels::Stereo)?;
        let buffer =
            vec![0.0; info.frame_samples as usize * info.channel_count as usize].into_boxed_slice();
        let pre_skip = info.pre_skip as usize;
        Ok(Self {
            file,
            position: 0,
            pre_skip,
            buffer,
            decoder,
        })
    }

    pub fn info(&self) -> &AudioInfo {
        &self.file.as_ref().info
    }

    pub fn samples_seek(&mut self, samples_position: usize) {
        let frames_position = samples_position / self.frame_samples();
        let bytes_position = frames_position * self.frame_size();
        let in_frame_position = samples_position % self.info().frame_samples as usize;

        // TODO: the decoder needs some time to settle, we probably should seek a little bit before

        self.position = bytes_position;
        self.pre_skip += in_frame_position;
        self.decoder.reset_state().unwrap();
    }

    pub fn samples_position(&self) -> usize {
        self.position / self.frame_size() * self.frame_samples() + self.pre_skip
    }

    fn frame_size(&self) -> usize {
        self.info().frame_size as usize
    }

    fn frame_samples(&self) -> usize {
        self.info().frame_samples as usize
    }

    pub fn decode_frame(&mut self) -> Option<&[f32]> {
        // the loop is here to handle pre-skips larger than one frame
        loop {
            let data = &self.file.as_ref().data;
            if self.position >= data.len() {
                return None;
            }

            let data = &data[self.position..][..self.frame_size()];

            assert_eq!(
                self.decoder.get_nb_samples(data).unwrap(),
                self.frame_samples()
            );

            let decoded = self
                .decoder
                .decode_float(data, &mut self.buffer, false)
                .unwrap();

            assert_eq!(decoded, self.frame_samples());

            self.position += self.frame_size();

            if self.pre_skip > self.frame_samples() {
                self.pre_skip -= self.frame_samples();
            } else {
                self.pre_skip = 0;
                break Some(&self.buffer[self.pre_skip..]);
            }
        }
    }
}

pub fn read_audio(data: &[u8]) -> Result<AudioFile> {
    let mut cur = std::io::Cursor::new(data);
    let header = NxaHeader::read_le(&mut cur)?;

    assert_eq!(header.file_size, data.len() as u32);
    assert_eq!(header.info.channel_count, 2);
    // how are we supposed to loop when the loop end is not in the end of the file?
    assert_eq!(header.info.loop_end, header.info.num_samples);

    let mut data = Vec::new();
    cur.read_to_end(&mut data)?;

    Ok(AudioFile {
        info: header.info,
        data,
    })
}
