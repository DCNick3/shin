use crate::format::text;
use anyhow::{Context, Result};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions};
use std::io::{Cursor, Read, Seek};

struct ShortString(String);
struct LongString(String);

impl BinRead for ShortString {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let len = u8::read_options(reader, options, ())?;
        // "- 1" to strip the null terminator

        let res = Self(text::read_sjis_string(reader, (len - 1) as usize)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e).into())?);

        // read the null terminator
        let _ = u8::read_options(reader, options, ())?;

        Ok(res)
    }
}

#[derive(BinRead, BinWrite)]
#[br(little)]
struct ScenarioSectionHeader {
    byte_size: u32,
    element_count: u32,
}

fn parse_section<T: BinRead<Args = ()>, C: AsRef<[u8]>>(cur: &mut Cursor<C>) -> Result<Vec<T>> {
    let header = ScenarioSectionHeader::read(cur).context("Parsing scenario section header")?;

    for _ in 0..header.element_count {
        let _ = T::read_le(cur).context("Parsing scenario section element")?;
    }
    todo!()
}

#[derive(BinRead, BinWrite)]
#[br(little, magic = b"SNR ")]
struct ScenarioHeader {
    pub size: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub commands_offset: u32,
    pub offset_36: u32,
    pub offset_40: u32,
    pub offset_44: u32,
    pub offset_48: u32,
    pub offset_52: u32,
    pub offset_56: u32,
    pub offset_60: u32,
    pub offset_64: u32,
    pub offset_68: u32,
    pub offset_72: u32,
    pub offset_76: u32,
    pub offset_80: u32,
    pub offset_84: u32,
}

pub struct ScenarioReader {}

impl ScenarioReader {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut cur = Cursor::new(&data);
        let header = ScenarioHeader::read(&mut cur)?;

        cur.set_position(header.offset_36 as u64);
        parse_section::<ShortString, _>(&mut cur)?;

        todo!()
    }
}
