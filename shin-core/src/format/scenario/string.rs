use crate::format::text;
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use smallvec::SmallVec;
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;

pub trait StringFixup {
    fn encode(string: String) -> String;
    fn decode(string: String) -> String;
}

#[derive(Debug)]
pub struct NoFixup;
impl StringFixup for NoFixup {
    fn encode(string: String) -> String {
        string
    }
    fn decode(string: String) -> String {
        string
    }
}

#[derive(Debug)]
pub struct WithFixup;
impl StringFixup for WithFixup {
    fn encode(string: String) -> String {
        text::encode_string_fixup(&string)
    }

    fn decode(string: String) -> String {
        text::decode_string_fixup(&string)
    }
}

#[derive(Debug)]
pub struct SJisString<L: Into<usize> + TryFrom<usize> + 'static, F: StringFixup + 'static = NoFixup>(
    pub String,
    pub PhantomData<(L, F)>,
);

#[derive(Debug)]
pub struct StringArray(pub SmallVec<[String; 4]>);

impl<L: Into<usize> + TryFrom<usize> + 'static, F: StringFixup> BinRead for SJisString<L, F> {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let len = u16::read_options(reader, options, ())?;
        // "- 1" to strip the null terminator

        let res = Self(
            // TODO: extra allocation in case of fixup
            F::decode(text::read_sjis_string(reader, Some((len - 1) as usize))?),
            PhantomData,
        );

        // read the null terminator
        let _ = u8::read_options(reader, options, ())?;

        Ok(res)
    }
}
impl<L: Into<usize> + TryFrom<usize>, F: StringFixup> BinWrite for SJisString<L, F> {
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
impl<L: Into<usize> + TryFrom<usize>, F: StringFixup> AsRef<str> for SJisString<L, F> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl<L: Into<usize> + TryFrom<usize>, F: StringFixup> SJisString<L, F> {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BinRead for StringArray {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: (),
    ) -> BinResult<Self> {
        let size = u16::read_options(reader, options, ())?;
        let pos = reader.stream_position()?;
        let mut res = SmallVec::new();
        loop {
            let s = text::read_sjis_string(reader, None)?;

            res.push(s);

            let v = u8::read_options(reader, options, ())?;
            if v == 0x00 {
                break;
            } else {
                reader.seek(SeekFrom::Current(-1))?;
            }
        }
        let end = reader.stream_position()?;
        let diff = end - pos;
        assert_eq!(diff, size as u64);
        Ok(Self(res))
    }
}

impl BinWrite for StringArray {
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
