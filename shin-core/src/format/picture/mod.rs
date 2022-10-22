use anyhow::Result;
use binrw::prelude::*;
use std::io;

#[derive(BinRead, BinWrite)]
#[br(little, magic = b"PIC4")]
struct PicHeader {
    version: u32,
    file_size: u32,
    origin_x: u16,
    origin_y: u16,
    effective_width: u16,
    effective_height: u16,
    field_20: u32,
    chunk_count: u32,
    picture_id: u32,
}

pub trait PictureBuilder {
    type Output;

    fn build(self) -> Result<Self::Output>;
}

pub struct DummyPictureBuilder;

impl PictureBuilder for DummyPictureBuilder {
    type Output = ();

    fn build(self) -> Result<Self::Output> {
        Ok(())
    }
}

pub fn read_picture<R: AsRef<[u8]>, L: PictureBuilder>(
    source: R,
    listener: L,
) -> Result<L::Output> {
    let mut source = io::Cursor::new(source.as_ref());
    let header = PicHeader::read(&mut source)?;

    if header.version != 3 {
        anyhow::bail!("Unsupported picture format version {}", header.version);
    }

    todo!()
}
