use std::{fmt, fmt::Debug, io, marker::PhantomData};

use binrw::{BinRead, BinResult, BinWrite, Endian};
use smallvec::SmallVec;

use crate::{
    format::scenario::instruction_elements::NumberSpec,
    vm::{IntoRuntimeForm, VmCtx},
};

/// A list of `T` with a length of `L`, stored in a `SmallVec` with size `N`
#[derive(Clone, PartialEq, Eq)]
pub struct SmallList<L, T, const N: usize>(pub SmallVec<T, N>, pub PhantomData<L>)
where
    L: Into<usize> + TryFrom<usize> + 'static;

pub const SMALL_LIST_SIZE: usize = 6;

pub type U8SmallList<T, const N: usize = SMALL_LIST_SIZE> = SmallList<u8, T, N>;
pub type U16SmallList<T, const N: usize = SMALL_LIST_SIZE> = SmallList<u16, T, N>;

pub type U8SmallNumberList<T = i32, const N: usize = SMALL_LIST_SIZE> =
    U8SmallList<NumberSpec<T>, N>;
pub type U16SmallNumberList<T = i32, const N: usize = SMALL_LIST_SIZE> =
    U16SmallList<NumberSpec<T>, N>;

impl<L, T, const N: usize> Debug for SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize>,
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("U8SmallList").field(&self.0).finish()
    }
}

impl<L, T, const N: usize> SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize>,
{
    pub fn from_contents<I: IntoIterator<Item = T>>(contents: I) -> Self {
        Self(contents.into_iter().collect(), PhantomData)
    }
}

impl<L, T, const N: usize> FromIterator<T> for SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize>,
    T: Clone,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
        Self(iterable.into_iter().collect(), PhantomData)
    }
}

impl<'a, L, T, const N: usize> From<&'a [T]> for SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize>,
    T: Clone,
{
    fn from(slice: &'a [T]) -> Self {
        slice.iter().cloned().collect()
    }
}

impl<L, T, const N: usize> BinRead for SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize> + for<'a> BinRead<Args<'a> = ()>,
    T: for<'a> BinRead<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let len = L::read_options(reader, endian, ())?.into();

        let mut res = SmallVec::new();
        res.reserve(len);
        for _ in 0..len {
            res.push(<_>::read_options(reader, endian, ())?);
        }

        Ok(Self(res, PhantomData {}))
    }
}

impl<L, T, const N: usize> BinWrite for SmallList<L, T, N>
where
    L: Into<usize> + TryFrom<usize> + for<'a> BinWrite<Args<'a> = ()>,
    T: for<'a> BinWrite<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: (),
    ) -> BinResult<()> {
        let pos = writer.stream_position()?;
        let len = L::try_from(self.0.len()).map_err(|_| binrw::Error::AssertFail {
            pos,
            message: format!("Failed to convert list length to the encoded representation. This is probably due to the list being too long."),
        })?;

        len.write_options(writer, endian, ())?;

        for item in &self.0 {
            item.write_options(writer, endian, ())?;
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use std::iter;

    use binrw::{io::NoSeek, BinWrite};

    use super::{SmallList, U16SmallList, U8SmallList};
    use crate::format::test_util::assert_enc_dec_pair;

    #[test]
    fn enc_dec_u8() {
        assert_enc_dec_pair(&U8SmallList::<u8>::from_contents([]), "00");
        assert_enc_dec_pair(&U8SmallList::<u8>::from_contents([1]), "0101");
        assert_enc_dec_pair(&U8SmallList::<u8>::from_contents([1, 2, 3]), "03010203");
        assert_enc_dec_pair(
            &U8SmallList::<u16>::from_contents([1, 2, 3]),
            "03010002000300",
        );

        assert_enc_dec_pair(&U8SmallList::<()>::from_contents([(), (), ()]), "03");

        assert_enc_dec_pair(
            &U8SmallList::<u8>::from_contents(iter::repeat(0xcc).take(255)),
            "ffcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        );
    }

    #[test]
    fn enc_dec_u16() {
        assert_enc_dec_pair(&U16SmallList::<u8>::from_contents([]), "0000");
        assert_enc_dec_pair(&U16SmallList::<u8>::from_contents([1]), "010001");
        assert_enc_dec_pair(&U16SmallList::<u8>::from_contents([1, 2, 3]), "0300010203");

        assert_enc_dec_pair(&U16SmallList::<()>::from_contents([(), (), ()]), "0300");

        assert_enc_dec_pair(
            &U16SmallList::<u8>::from_contents(iter::repeat(0xcc).take(256)),
            "0001cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        );
    }

    #[test]
    fn enc_too_long() {
        fn assert_length_error<
            L: Into<usize> + TryFrom<usize> + for<'a> BinWrite<Args<'a> = ()>,
            T: for<'a> BinWrite<Args<'a> = ()>,
            const N: usize,
        >(
            list: SmallList<L, T, N>,
        ) {
            match list.write_le(&mut NoSeek::new(vec![])).unwrap_err() {
                binrw::Error::AssertFail { pos, message } => {
                    assert_eq!(pos, 0);
                    assert_eq!(message, "Failed to convert list length to the encoded representation. This is probably due to the list being too long.")
                }
                e => panic!("Unexpected error: {:?}", e),
            }
        }

        assert_length_error(U8SmallList::<u8>::from_contents(
            iter::repeat(0xcc).take(256),
        ));
        assert_length_error(U16SmallList::<u8>::from_contents(
            iter::repeat(0xcc).take(65536),
        ));
    }
}
