use crate::format::text;
use binrw::{BinRead, BinResult, BinWrite, Endian};
use smallvec::SmallVec;
use std::fmt::Debug;
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

pub trait StringLengthDesc:
    for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + Sized + 'static
{
    /// Should return the length of the string, in bytes, including the null terminator.
    fn get_length(&self) -> Option<usize>;
}

impl StringLengthDesc for u8 {
    fn get_length(&self) -> Option<usize> {
        Some(*self as usize)
    }
}

impl StringLengthDesc for u16 {
    fn get_length(&self) -> Option<usize> {
        Some(*self as usize)
    }
}

impl StringLengthDesc for () {
    fn get_length(&self) -> Option<usize> {
        None
    }
}

/// A string that is encoded in Shift-JIS when written to a file.
///
/// `L` specifies how the length will be encoded in the file.
/// `F` describes the fixup to be applied to the string (an additional transform that is applied to the string before it is written to the file).
///     Entergram uses it to convert hiragana to half-width katakana in some places, probably saving a bit of space.
pub struct SJisString<L: StringLengthDesc, F: StringFixup + 'static = NoFixup>(
    pub String,
    pub PhantomData<(L, F)>,
);

#[derive(Debug)]
pub struct StringArray(pub SmallVec<[String; 4]>);

impl<L: StringLengthDesc, F: StringFixup> BinRead for SJisString<L, F> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let len = L::read_options(reader, endian, ())?;
        // "- 1" to strip the null terminator

        let res = Self(
            // TODO: extra allocation in case of fixup
            // this will consume the null terminator
            F::decode(text::read_sjis_string(reader, len.get_length())?),
            PhantomData,
        );

        // read the null terminator
        // let _ = u8::read_options(reader, options, ())?;

        Ok(res)
    }
}
impl<L: StringLengthDesc, F: StringFixup> BinWrite for SJisString<L, F> {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}
impl<L: StringLengthDesc, F: StringFixup> AsRef<str> for SJisString<L, F> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl<L: StringLengthDesc, F: StringFixup> Debug for SJisString<L, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl<L: StringLengthDesc, F: StringFixup> std::fmt::Display for SJisString<L, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl<L: StringLengthDesc, F: StringFixup> SJisString<L, F> {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BinRead for StringArray {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let size = u16::read_options(reader, endian, ())?;
        let pos = reader.stream_position()?;
        let mut res = SmallVec::new();
        loop {
            let s = text::read_sjis_string(reader, None)?;

            res.push(s);

            let v = u8::read_options(reader, endian, ())?;
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
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}
