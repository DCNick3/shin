use std::{
    fmt,
    io::{Read, Seek, Write},
    marker::PhantomData,
};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use derivative::Derivative;
use smallvec::SmallVec;

use crate::{
    format::scenario::instruction_elements::NumberSpec,
    vm::{IntoRuntimeForm, VmCtx},
};

/// A list of `T` with a length of `L`, stored in a `SmallVec` with size `N`
#[derive(Clone, PartialEq, Eq)]
pub struct SmallList<L: Into<usize> + TryFrom<usize> + 'static, T, const N: usize>(
    pub SmallVec<T, N>,
    pub PhantomData<L>,
);

pub const SMALL_LIST_SIZE: usize = 6;

pub type U8SmallList<T, const N: usize = SMALL_LIST_SIZE> = SmallList<u8, T, N>;
pub type U16SmallList<T, const N: usize = SMALL_LIST_SIZE> = SmallList<u16, T, N>;

pub type U8SmallNumberList<T = i32, const N: usize = SMALL_LIST_SIZE> =
    U8SmallList<NumberSpec<T>, N>;
pub type U16SmallNumberList<T = i32, const N: usize = SMALL_LIST_SIZE> =
    U16SmallList<NumberSpec<T>, N>;

/// Pad the contents to 4 bytes
///
/// (Used in [super::Instruction::gt])
#[derive(Derivative, PartialEq, Eq, Copy, Clone)]
#[derivative(Debug = "transparent")]
pub struct Pad4<T>(pub T);

impl<L: Into<usize> + TryFrom<usize>, T, const N: usize> fmt::Debug for SmallList<L, T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("U8SmallList").field(&self.0).finish()
    }
}

impl<
        L: Into<usize> + TryFrom<usize> + for<'a> BinRead<Args<'a> = ()>,
        T: for<'a> BinRead<Args<'a> = ()>,
        const N: usize,
    > BinRead for SmallList<L, T, N>
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
        T: for<'a> BinWrite<Args<'a> = ()>,
        const N: usize,
    > BinWrite for SmallList<L, T, N>
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

impl<L, Ts, Td, const N: usize> IntoRuntimeForm for SmallList<L, Ts, N>
where
    L: Into<usize> + TryFrom<usize>,
    Ts: IntoRuntimeForm<Output = Td>,
{
    type Output = SmallVec<Ts::Output, N>;
    fn into_runtime_form(self, ctx: &VmCtx) -> Self::Output {
        self.0
            .into_iter()
            .map(|ts| ts.into_runtime_form(ctx))
            .collect()
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
