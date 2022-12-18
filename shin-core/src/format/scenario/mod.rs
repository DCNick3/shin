//! Support for SNR file format, storing the game scenario.
//!
//! See also [crate::vm] for the VM that runs the scenario.

pub mod instructions;
mod string;

use crate::format::scenario::instructions::{CodeAddress, Instruction, NumberSpec};
use anyhow::{Context, Result};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, VecArgs, WriteOptions};
use bytes::Bytes;
use smallvec::SmallVec;
use std::fmt;
use std::io::{Cursor, Read, Seek, Write};
use std::marker::PhantomData;

// TODO: make lists generic over the type of length
#[derive(Debug)]
pub struct U8List<T>(pub Vec<T>);
#[derive(Debug)]
pub struct U16List<T>(pub Vec<T>);

pub use string::{SJisString, StringArray};

pub type U8String = SJisString<u8>;
pub type U16String = SJisString<u16>;
pub type U8FixupString = SJisString<u8, string::WithFixup>;
pub type U16FixupString = SJisString<u16, string::WithFixup>;

pub struct SmallList<L: Into<usize> + TryFrom<usize> + 'static, A: smallvec::Array>(
    pub SmallVec<A>,
    pub PhantomData<L>,
);

pub type U8SmallList<A> = SmallList<u8, A>;
pub type U16SmallList<A> = SmallList<u16, A>;

pub type U8SmallNumberList<A = [NumberSpec; 6]> = U8SmallList<A>;
pub type U16SmallNumberList<A = [NumberSpec; 6]> = U16SmallList<A>;

impl<T: BinRead<Args = ()>> BinRead for U8List<T> {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let len = u8::read_options(reader, options, ())?;

        Ok(Self(<_>::read_options(
            reader,
            options,
            VecArgs {
                count: len as usize,
                inner: (),
            },
        )?))
    }
}
impl<T: BinWrite<Args = ()>> BinWrite for U8List<T> {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}
impl<T: BinRead<Args = ()>> BinRead for U16List<T> {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let len = u16::read_options(reader, options, ())?;

        Ok(Self(<_>::read_options(
            reader,
            options,
            VecArgs {
                count: len as usize,
                inner: (),
            },
        )?))
    }
}
impl<T: BinWrite<Args = ()>> BinWrite for U16List<T> {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

impl<L: Into<usize> + TryFrom<usize>, A: smallvec::Array> fmt::Debug for SmallList<L, A>
where
    A::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("U8SmallList").field(&self.0).finish()
    }
}

impl<
        L: Into<usize> + TryFrom<usize> + BinRead<Args = ()>,
        A: smallvec::Array<Item = T> + 'static,
        T: BinRead<Args = ()>,
    > BinRead for SmallList<L, A>
{
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let len = L::read_options(reader, options, ())?.into();

        let mut res = SmallVec::new();
        res.reserve(len);
        for _ in 0..len {
            res.push(<_>::read_options(reader, options, ())?);
        }

        Ok(Self(res, PhantomData {}))
    }
}
impl<
        L: Into<usize> + TryFrom<usize> + BinWrite<Args = ()>,
        A: smallvec::Array<Item = T> + 'static,
        T: BinWrite<Args = ()>,
    > BinWrite for SmallList<L, A>
{
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _: (),
    ) -> BinResult<()> {
        todo!()
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

    let mut res = Vec::new();
    for _ in 0..header.element_count {
        res.push(T::read_le(cur).context("Parsing scenario section element")?);
    }

    Ok(res)
}

fn parse_simple_section<T: BinRead<Args = ()>, C: AsRef<[u8]>>(
    cur: &mut Cursor<C>,
) -> Result<Vec<T>> {
    let count = u32::read_le(cur).context("Parsing scenario section element count")?;

    let mut res = Vec::new();
    for _ in 0..count {
        res.push(T::read_le(cur).context("Parsing scenario section element")?);
    }

    Ok(res)
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
    pub code_offset: u32,
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

#[allow(unused)]
pub struct Scenario {
    mask_data: Vec<U16String>,
    pic_data: Vec<(U16String, i16)>,
    bup_data: Vec<(U16String, U16String, u16)>,
    bgm_data: Vec<(U16String, U16String, u16)>,
    se_data: Vec<U16String>,
    movie_data: Vec<(U16String, u16, u16, u16)>,
    voice_data: Vec<(U16String, U8List<u8>)>,
    section_64: Vec<(U16String, U16List<u16>)>,
    section_68: Vec<(u16, u16, u16)>,
    tips_data: Vec<(u8, u16, U16String, U16String)>,
    entrypoint_address: CodeAddress,
    raw_data: Bytes,
}

impl Scenario {
    pub fn new(data: Bytes) -> Result<Self> {
        let mut cur = Cursor::new(&data);
        let header = ScenarioHeader::read(&mut cur)?;

        // looks like mask names
        cur.set_position(header.offset_36 as u64);
        let mask_data = parse_section(&mut cur)?;

        // looks like CG names
        // not sure when the piggypacked number means
        cur.set_position(header.offset_40 as u64);
        let pic_data = parse_section(&mut cur)?;

        // these are names of character sprites
        // the first string is the bup filename
        // the second is the emotion name
        // the third number is the character index to be used for lip sync
        cur.set_position(header.offset_44 as u64);
        let bup_data = parse_section::<(U16String, U16String, u16), _>(&mut cur)?;

        // these are names of BGMs
        // the first string is the BGM filename
        // the second string is the BGM display name
        // the third value is... probably a BGM index but i dunno really
        cur.set_position(header.offset_48 as u64);
        let bgm_data = parse_section::<(U16String, U16String, u16), _>(&mut cur)?;

        // these are filenames for SEs
        cur.set_position(header.offset_52 as u64);
        let se_data = parse_section::<U16String, _>(&mut cur)?;

        // these are movie names
        // don't know what the numbers mean
        cur.set_position(header.offset_56 as u64);
        let movie_data = parse_section::<(U16String, u16, u16, u16), _>(&mut cur)?;

        // these are used to map voice names to a list of character indices
        // the first string is the voice name, possibly including a wildcard (e.g. "57/*")
        cur.set_position(header.offset_60 as u64);
        let voice_data = parse_section::<(U16String, U8List<u8>), _>(&mut cur)?;

        // not sure what is this for
        // the strings are pic names
        cur.set_position(header.offset_64 as u64);
        let section_64 = parse_simple_section::<(U16String, U16List<u16>), _>(&mut cur)?;

        // not sure what is this for
        cur.set_position(header.offset_68 as u64);
        let section_68 = parse_simple_section::<(u16, u16, u16), _>(&mut cur)?;

        // not sure what is this for
        // don't know how to even parse this
        // cur.set_position(header.offset_72 as u64);
        // let section_72 = parse_simple_section::<(u16, u16, u16), _>(&mut cur)?;

        // looks like info for the characters screen
        // don't know how to parse the leading data (maybe some lists?? it varies in length but seems to be very similar between entries...)
        // there is a parser impl at https://gitlab.com/Neurochitin/kaleido/-/blob/saku/snr-reader/read_scenario.rb#L1886
        //cur.set_position(header.offset_76 as u64);
        //let section_76 = parse_section::<(U8List<u8>, U8List<u8>, U16String, U16String, u8, U16String, U16String), _>(&mut cur)?;

        // some numbers, idk...
        // cur.set_position(header.offset_80 as u64);
        // let section_80 = parse_section::<(U16String, U16String, u16), _>(&mut cur)?;

        // TIPS data
        // the first number appears to be the episode index
        // the second number appears to be the tip index in the episode
        cur.set_position(header.offset_84 as u64);
        let tips_data = parse_section::<(u8, u16, U16String, U16String), _>(&mut cur)?;

        Ok(Self {
            mask_data,
            pic_data,
            bup_data,
            bgm_data,
            se_data,
            movie_data,
            voice_data,
            section_64,
            section_68,
            tips_data,
            entrypoint_address: CodeAddress(header.code_offset),
            raw_data: data,
        })
    }

    pub fn get_picture_data(&self, pic_id: i32) -> (&str, i16) {
        let (pic_name, pic_index) = &self.pic_data[pic_id as usize];
        (pic_name.as_str(), *pic_index)
    }

    pub fn get_bustup_data(&self, bup_id: i32) -> (&str, &str, u16) {
        let (bup_name, bup_emotion, bup_index) = &self.bup_data[bup_id as usize];
        (bup_name.as_str(), bup_emotion.as_str(), *bup_index)
    }

    pub fn raw(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn entrypoint_address(&self) -> CodeAddress {
        self.entrypoint_address
    }

    pub fn instruction_reader(&self, offset: CodeAddress) -> InstructionReader {
        InstructionReader::new(self.raw_data.clone(), offset)
    }
}

pub struct InstructionReader {
    cur: Cursor<Bytes>,
}

impl InstructionReader {
    pub fn new(data: Bytes, offset: CodeAddress) -> Self {
        let mut cur = Cursor::new(data);
        cur.set_position(offset.0 as u64);
        Self { cur }
    }

    #[inline]
    pub fn read(&mut self) -> Result<Instruction> {
        let instruction = Instruction::read(&mut self.cur)?;
        Ok(instruction)
    }

    #[inline]
    pub fn position(&self) -> CodeAddress {
        CodeAddress(self.cur.position().try_into().unwrap())
    }

    pub fn set_position(&mut self, offset: CodeAddress) {
        assert!(offset.0 as u64 <= self.cur.get_ref().len() as u64);
        self.cur.set_position(offset.0 as u64);
    }
}
