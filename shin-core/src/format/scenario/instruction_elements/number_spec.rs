use super::Register;
use crate::vm::{FromVmCtx, FromVmCtxDefault, VmCtx};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::fmt::Debug;
use std::io;
use std::marker::PhantomData;

/// Specifies how to get a 32-bit signed number at runtime
///
/// It can be a constant or be referencing a register.
///
/// [FromVmCtx](crate::vm::FromVmCtx) trait is used to convert it to runtime representation in command definitions (see [crate::vm::command])
#[derive(Copy, Clone)]
pub enum UntypedNumberSpec {
    Constant(i32),
    Register(Register),
}

impl Debug for UntypedNumberSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(c) => write!(f, "{}", c),
            Self::Register(r) => write!(f, "{}", r),
        }
    }
}

impl BinRead for UntypedNumberSpec {
    type Args<'a> = ();

    //noinspection SpellCheckingInspection
    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<Self> {
        let t = u8::read_options(reader, endian, ())?;
        // t=TXXXXXXX
        // T=0 => XXXXXXX is a 7-bit signed constant
        // T=1 => futher processing needed
        Ok(if t & 0x80 != 0 {
            // t=1PPPKKKK
            let p = (t & 0x70) >> 4;
            let k = t & 0x0F;
            // does the sign extension of k, using bits [0:3] (4 bit number)
            let k_sext = (k as i32) << 28 >> 28;
            // P=0 => 12-bit signed constant (KKKK denotes the upper 4 bits, lsb is read from the next byte)
            // P=1 => 20-bit signed constant (KKKK denotes the upper 4 bits, 2 lower bytes are read from the stream)
            // P=2 => 26-bit signed constante (KKKK denotes the upper 4 bits, 3 lower bytes are read from the stream)
            // P=3 => 4-bit regular register, KKKK is the index
            // P=4 => 12-bit regular register, KKKK denotes the upper 4 bits, lsb is read from the next byte
            // P=5 => 4-bit argument register, KKKK + 1 is the index
            match p {
                0 => Self::Constant(u8::read_options(reader, endian, ())? as i32 | (k_sext << 8)),
                1 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, endian, ())? as i32;
                    let b2 = u8::read_options(reader, endian, ())? as i32;
                    Self::Constant(b2 | (b1 << 8) | (k_sext << 16))
                }
                2 => {
                    // it's big endian......
                    let b1 = u8::read_options(reader, endian, ())? as i32;
                    let b2 = u8::read_options(reader, endian, ())? as i32;
                    let b3 = u8::read_options(reader, endian, ())? as i32;
                    Self::Constant(b3 | (b2 << 8) | (b1 << 16) | (k_sext << 24))
                }
                3 => Self::Register(Register::from_regular_register(k as u16)),
                4 => Self::Register(Register::from_regular_register(
                    u8::read_options(reader, endian, ())? as u16 | (k as u16) << 8,
                )),
                5 => Self::Register(Register::from_argument(k as u16)),
                _ => unreachable!("unknown number spec type: P={}", p),
            }
        } else {
            // signed 7-bit integer
            // does the sign extension of t, using bits [0:6]
            let res = (t as i32 & 0x7f) << 25 >> 25;
            Self::Constant(res)
        })
    }
}

impl BinWrite for UntypedNumberSpec {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _: (),
    ) -> BinResult<()> {
        todo!()
    }
}

#[derive(Copy, Clone, BinRead)]
pub struct NumberSpec<T = i32>(UntypedNumberSpec, PhantomData<T>);

impl<T> NumberSpec<T> {
    pub fn new(spec: UntypedNumberSpec) -> Self {
        Self(spec, PhantomData)
    }

    pub fn into_untyped(self) -> UntypedNumberSpec {
        self.0
    }
}

// See https://github.com/jam1garner/binrw/pull/230
impl<T> BinWrite for NumberSpec<T> {
    type Args<'a> = ();

    fn write_options<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.0.write_options(writer, endian, args)
    }
}

impl<T> Debug for NumberSpec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: FromNumber> FromVmCtx<NumberSpec<T>> for T {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec<T>) -> Self {
        ctx.get_number(input)
    }
}
impl<T: FromNumber> FromVmCtxDefault for NumberSpec<T> {
    type Output = T;
}

pub trait FromNumber {
    fn from_number(number: i32) -> Self;
}

impl FromNumber for bool {
    fn from_number(number: i32) -> Self {
        number != 0
    }
}

impl FromNumber for i32 {
    fn from_number(number: i32) -> Self {
        number
    }
}
