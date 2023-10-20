use super::UntypedNumberSpec;
use crate::format::scenario::instruction_elements::{FromNumber, NumberSpec};
use crate::vm::{IntoRuntimeForm, VmCtx};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::io;

/// An 8-typle of i32
pub type UntypedNumberArray = (i32, i32, i32, i32, i32, i32, i32, i32);

/// Represents 8 numbers, each of which may or may not be present.
///
/// If the number is not present, it is treated as `NumberSpec::Constant(0)`.
#[derive(Debug, Copy, Clone)]
pub struct BitmaskNumberArray<
    T1 = i32,
    T2 = i32,
    T3 = i32,
    T4 = i32,
    T5 = i32,
    T6 = i32,
    T7 = i32,
    T8 = i32,
>(
    NumberSpec<T1>,
    NumberSpec<T2>,
    NumberSpec<T3>,
    NumberSpec<T4>,
    NumberSpec<T5>,
    NumberSpec<T6>,
    NumberSpec<T7>,
    NumberSpec<T8>,
);

impl<T1, T2, T3, T4, T5, T6, T7, T8> BinRead
    for BitmaskNumberArray<T1, T2, T3, T4, T5, T6, T7, T8>
{
    type Args<'a> = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,

        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let mut untyped = [UntypedNumberSpec::Constant(0); 8];
        let mut mask = u8::read_options(reader, endian, ())?;
        for res in untyped.iter_mut() {
            if mask & 1 != 0 {
                *res = UntypedNumberSpec::read_options(reader, endian, ())?;
            }
            mask >>= 1;
        }

        Ok(Self(
            NumberSpec::new(untyped[0]),
            NumberSpec::new(untyped[1]),
            NumberSpec::new(untyped[2]),
            NumberSpec::new(untyped[3]),
            NumberSpec::new(untyped[4]),
            NumberSpec::new(untyped[5]),
            NumberSpec::new(untyped[6]),
            NumberSpec::new(untyped[7]),
        ))
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> BinWrite
    for BitmaskNumberArray<T1, T2, T3, T4, T5, T6, T7, T8>
{
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _: &mut W,
        _: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        todo!()
    }
}

impl<
        T1: FromNumber,
        T2: FromNumber,
        T3: FromNumber,
        T4: FromNumber,
        T5: FromNumber,
        T6: FromNumber,
        T7: FromNumber,
        T8: FromNumber,
    > IntoRuntimeForm for BitmaskNumberArray<T1, T2, T3, T4, T5, T6, T7, T8>
{
    type Output = (T1, T2, T3, T4, T5, T6, T7, T8);

    fn into_runtime_form(self, ctx: &VmCtx) -> Self::Output {
        (
            self.0.into_runtime_form(ctx),
            self.1.into_runtime_form(ctx),
            self.2.into_runtime_form(ctx),
            self.3.into_runtime_form(ctx),
            self.4.into_runtime_form(ctx),
            self.5.into_runtime_form(ctx),
            self.6.into_runtime_form(ctx),
            self.7.into_runtime_form(ctx),
        )
    }
}
