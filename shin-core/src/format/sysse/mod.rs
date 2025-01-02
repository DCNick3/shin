use std::{collections::BTreeMap, io::Read};

use anyhow::{Context, Result};
use binrw::{BinRead, BinWrite};
use bytes::Buf;

use crate::format::{
    audio::{AudioBuffer, AudioFrameSource},
    sysse::kernel::{ChannelCount, DecoderKernel as _},
};

mod kernel;

#[derive(BinRead, BinWrite)]
#[brw(magic = b"SYSE")]
struct SysseHeader {
    pub file_size: u32,
    pub entry_count: u32,
    #[br(count = entry_count)]
    pub entries: Vec<SoundEntry>,
}

#[derive(BinRead, BinWrite)]
struct SoundEntry {
    pub name: [u8; 16],
    pub offset: u32,
    pub size: u32,
}

#[derive(BinRead, BinWrite)]
#[brw(magic = b"ADP1")]
struct SoundHeader {
    pub file_size: u32,
    pub channel_count: ChannelCount,
    pub sample_rate: u16,
    pub sample_count: u32,
}

const BLOCKS_PER_FRAME: usize = 80;

pub struct Sound {
    channel_count: ChannelCount,
    sample_rate: u32,
    sample_count: u32,
    sample_data: Vec<u8>,
}

impl Sound {
    fn new(data: &[u8]) -> Result<Self> {
        let mut cur = std::io::Cursor::new(data);
        let header = SoundHeader::read_le(&mut cur)?;
        assert_eq!(header.file_size, data.len() as u32);

        let mut sample_data = Vec::with_capacity(cur.remaining());
        cur.read_to_end(&mut sample_data)?;

        Ok(Self {
            channel_count: header.channel_count,
            sample_rate: header.sample_rate as u32,
            sample_count: header.sample_count,
            sample_data,
        })
    }

    pub fn channel_count(&self) -> ChannelCount {
        self.channel_count
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn decode(&self) -> SoundDecoder {
        let kernel = kernel::EitherDecoderKernel::new(self.sample_data.clone(), self.channel_count);

        SoundDecoder {
            kernel,
            reached_eof: false,
            sample_rate: self.sample_rate,
            sample_position: 0,
            sample_count: self.sample_count,
        }
    }
}

pub struct SoundDecoder {
    kernel: kernel::EitherDecoderKernel,
    reached_eof: bool,
    sample_rate: u32,
    sample_position: u32,
    sample_count: u32,
}

impl AudioFrameSource for SoundDecoder {
    fn max_frame_size(&self) -> usize {
        BLOCKS_PER_FRAME * kernel::BLOCK_SIZE
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn pre_skip(&self) -> u32 {
        0
    }

    fn pre_roll(&self) -> u32 {
        0
    }

    fn read_frame(&mut self, destination: &mut AudioBuffer) -> bool {
        if self.reached_eof {
            return false;
        }

        let mut written_anything = false;
        for _ in 0..BLOCKS_PER_FRAME {
            let Some(block) = self.kernel.decode_block() else {
                self.reached_eof = true;
                break;
            };
            written_anything = true;

            if self.sample_position as usize + kernel::BLOCK_SIZE > self.sample_count as usize {
                let leftover = self.sample_count as usize - self.sample_position as usize;
                destination.extend(block[..leftover].iter().copied());
                self.sample_position += leftover as u32;
                self.reached_eof = true;
                break;
            } else {
                destination.extend(block);
                self.sample_position += kernel::BLOCK_SIZE as u32;
            }
        }

        written_anything
    }

    fn samples_seek(&mut self, _sample_position: u32) -> Result<u32> {
        unimplemented!("Sysse sounds do not support seeking")
    }

    fn current_sample_position(&self) -> u32 {
        self.sample_position
    }
}

pub struct SysSe {
    pub sounds: BTreeMap<String, Sound>,
}

pub fn read_sys_se(data: &[u8]) -> Result<SysSe> {
    let mut cur = std::io::Cursor::new(data);
    let header = SysseHeader::read_le(&mut cur).context("Failed to read header")?;
    assert_eq!(header.file_size, data.len() as u32);

    let mut sounds = BTreeMap::new();
    for entry in header.entries {
        let mut name = entry.name.as_ref();
        while let Some(head) = name.strip_suffix(b"\x00") {
            name = head
        }
        let name = std::str::from_utf8(name)
            .context("Failed to parse sysse name as UTF-8 string")?
            .to_string();

        let data = &data[entry.offset as usize..][..entry.size as usize];
        let sound = Sound::new(data).context("Failed to read sound")?;
        sounds.insert(name, sound);
    }

    Ok(SysSe { sounds })
}
