use crate::format::scenario::instruction_elements::NumberSpec;
use binrw::{BinRead, BinResult, BinWrite, Endian, VecArgs};
use derivative::Derivative;
use smallvec::SmallVec;
use std::fmt;
use std::io::{Read, Seek, Write};
use std::marker::PhantomData;

// TODO: make lists generic over the type of length
/// A list of `T` with a u8 length
#[derive(Debug)]
pub struct U8List<T>(pub Vec<T>);

/// A list of `T` with a u16 length
#[derive(Debug)]
pub struct U16List<T>(pub Vec<T>);

/// A list of `T` with a length of `L`, stored in a `SmallVec` with array `A`
pub struct SmallList<L: Into<usize> + TryFrom<usize> + 'static, A: smallvec::Array>(
    pub SmallVec<A>,
    pub PhantomData<L>,
);

pub type U8SmallList<A> = SmallList<u8, A>;
pub type U16SmallList<A> = SmallList<u16, A>;

pub const SMALL_LIST_SIZE: usize = 6;

pub type U8SmallNumberList<T = i32> = U8SmallList<[NumberSpec<T>; SMALL_LIST_SIZE]>;
pub type U16SmallNumberList<T = i32> = U16SmallList<[NumberSpec<T>; SMALL_LIST_SIZE]>;

/// Pad the contents to 4 bytes
///
/// (Used in [super::Instruction::jt])
#[derive(Derivative)]
#[derivative(Debug = "transparent")]
pub struct Pad4<T>(pub T);

impl<T: for<'a> BinRead<Args<'a> = ()> + 'static> BinRead for U8List<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let len = u8::read_options(reader, endian, ())?;

        Ok(Self(<_>::read_options(
            reader,
            endian,
            VecArgs {
                count: len as usize,
                inner: (),
            },
        )?))
    }
}
impl<T: for<'a> BinWrite<Args<'a> = ()>> BinWrite for U8List<T> {
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

impl<T: for<'a> BinRead<Args<'a> = ()> + 'static> BinRead for U16List<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let len = u16::read_options(reader, endian, ())?;

        Ok(Self(<_>::read_options(
            reader,
            endian,
            VecArgs {
                count: len as usize,
                inner: (),
            },
        )?))
    }
}
impl<T: for<'a> BinWrite<Args<'a> = ()>> BinWrite for U16List<T> {
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

impl<L: Into<usize> + TryFrom<usize>, A: smallvec::Array> fmt::Debug for SmallList<L, A>
where
    A::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("U8SmallList").field(&self.0).finish()
    }
}

impl<
        L: Into<usize> + TryFrom<usize> + for<'a> BinRead<Args<'a> = ()>,
        A: smallvec::Array<Item = T> + 'static,
        T: for<'a> BinRead<Args<'a> = ()>,
    > BinRead for SmallList<L, A>
{
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let len = L::read_options(reader, endian, ())?.into();

        let mut res = SmallVec::new();
        res.reserve(len);
        for _ in 0..len {
            res.push(<_>::read_options(reader, endian, ())?);
        }

        Ok(Self(res, PhantomData {}))
    }
}

impl<
        L: Into<usize> + TryFrom<usize> + for<'a> BinWrite<Args<'a> = ()>,
        A: smallvec::Array<Item = T> + 'static,
        T: for<'a> BinWrite<Args<'a> = ()>,
    > BinWrite for SmallList<L, A>
{
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

impl<T: for<'a> BinRead<Args<'a> = ()> + 'static> BinRead for Pad4<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        let res = <_>::read_options(reader, endian, args)?;
        let new_pos = reader.stream_position()?;

        assert!(new_pos - pos <= 4, "Pad4: read more than 4 bytes");

        reader.seek(std::io::SeekFrom::Start(pos + 4))?;

        Ok(Self(res))
    }
}
impl<T: for<'a> BinWrite<Args<'a> = ()>> BinWrite for Pad4<T> {
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
